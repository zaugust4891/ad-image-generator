/**
 * API configuration
 * Uses VITE_API_BASE_URL environment variable if set, otherwise defaults to localhost:8787
 */
export const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL || "http://127.0.0.1:8787";
