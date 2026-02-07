use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{future::Future, path::PathBuf, pin::Pin, sync::Arc};
use tokio::{fs, io::{AsyncBufReadExt, AsyncWriteExt}, sync::Mutex};

pub trait PromptRewriter: Send + Sync {
    fn rewrite<'a>(
        &'a self,
        original: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;
    fn name(&self) -> &'static str;
}

pub struct NoopRewriter;
impl PromptRewriter for NoopRewriter {
    fn rewrite<'a>(
        &'a self,
        original: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move { Ok(original.to_string()) })
    }

    fn name(&self) -> &'static str { "noop" }
}

pub struct OpenAIRewriter{ client: reqwest::Client, api_key: String, model: String, system: String, max_tokens: u32 }
impl OpenAIRewriter{
    pub fn new(api_key:String, model:String, system:String, max_tokens:u32)->Self{
        Self{ client:reqwest::Client::new(), api_key, model, system, max_tokens }
    }
}
#[derive(Serialize)] struct ChatReq<'a>{ model:&'a str, messages:Vec<Msg<'a>>, max_tokens:u32 }
#[derive(Serialize)] struct Msg<'a>{ role:&'a str, content:&'a str }
#[derive(Deserialize)] struct ChatResp{ choices:Vec<Choice> }
#[derive(Deserialize)] struct Choice{ message: MsgOwned }
#[derive(Deserialize)] struct MsgOwned{ #[allow(unused)] role:String, content:String }

impl PromptRewriter for OpenAIRewriter {
    fn rewrite<'a>(
        &'a self,
        original: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            let req = ChatReq{
                model:&self.model,
                max_tokens:self.max_tokens,
                messages:vec![
                    Msg{role:"system", content:&self.system},
                    Msg{role:"user", content:original},
                ],
            };
            let resp = self.client.post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(&self.api_key)
                .json(&req).send().await?.error_for_status()?.json::<ChatResp>().await?;
            Ok(resp.choices.get(0).map(|c| c.message.content.clone()).unwrap_or_else(|| original.to_string()))
        })
    }

    fn name(&self) -> &'static str { "openai-rewriter" }
}

pub struct RewriteCache{ path: PathBuf, map: Arc<Mutex<std::collections::HashMap<String,String>>> }
impl RewriteCache{
    pub async fn load(path: PathBuf) -> Result<Self> {
        let mut map = std::collections::HashMap::new();
        if let Ok(f) = fs::File::open(&path).await {
            let mut lines = tokio::io::BufReader::new(f).lines();
            while let Some(line) = lines.next_line().await? {
                if let Ok((k,v)) = serde_json::from_str::<(String,String)>(&line) { map.insert(k,v); }
            }
        }
        Ok(Self{ path, map: Arc::new(Mutex::new(map)) })
    }
    pub async fn get(&self, key:&str)->Option<String>{ self.map.lock().await.get(key).cloned() }
    pub async fn put(&self, key:&str, val:&str)->Result<()>{
        {
            self.map.lock().await.insert(key.to_string(), val.to_string());
        }
        let mut f = fs::OpenOptions::new().create(true).append(true).open(&self.path).await?;
        let line = serde_json::to_string(&(key, val))?;
        f.write_all(line.as_bytes()).await?;
        f.write_all(b"\n").await?;
        Ok(())
    }
}

pub fn cache_key(original:&str, rewriter_name:&str, model:&str, system:&str)->String{
    let mut h = Sha256::new();
    h.update(rewriter_name.as_bytes());
    h.update(model.as_bytes());
    h.update(system.as_bytes());
    h.update(b"\x1f");
    h.update(original.as_bytes());
    format!("{:x}", h.finalize())
}
