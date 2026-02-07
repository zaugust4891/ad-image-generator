# Development sandbox for ad-image-generator
# Using Rust nightly because image 0.25.x depends on moxcms which requires edition2024
FROM rust:1.84-slim-bookworm

# Install dependencies (slim image needs more base packages)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    git \
    pkg-config \
    libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Rust nightly (required for moxcms crate which uses edition2024)
RUN rustup toolchain install nightly \
    && rustup default nightly

# Install Node.js 20.x (LTS)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Default command: keep container running for interactive use
CMD ["bash", "-c", "echo 'Dev sandbox ready. Run: cargo run -- serve' && echo 'In another terminal: cd adgen-ui && npm install && npm run dev --host' && tail -f /dev/null"]

