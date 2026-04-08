import { env } from "@/env";

const API_BASE_URL = env.NEXT_PUBLIC_API_BASE_URL;

export interface ApiClientOptions extends RequestInit {
  retries?: number;
  retryDelay?: number;
}

/**
 * Shared API client for the dashboard.
 * Handles base URL, credentials, basic error parsing, and resilient features like retries and cancellation.
 */
export async function apiClient<T>(
  endpoint: string,
  { retries = 2, retryDelay = 1000, ...options }: ApiClientOptions = {},
): Promise<T> {
  const url = endpoint.startsWith("http")
    ? endpoint
    : `${API_BASE_URL}${endpoint.startsWith("/") ? "" : "/"}${endpoint}`;

  let lastError: Error | null = null;

  for (let attempt = 0; attempt <= retries; attempt++) {
    try {
      const response = await fetch(url, {
        ...options,
        headers: {
          "Content-Type": "application/json",
          ...options.headers,
        },
        credentials: "include",
      });

      if (!response.ok) {
        // Only retry on transient errors (5xx, 429)
        if (attempt < retries && (response.status >= 500 || response.status === 429)) {
          const delay = retryDelay * Math.pow(2, attempt);
          await new Promise((resolve) => setTimeout(resolve, delay));
          continue;
        }

        const errorBody = await response.text().catch(() => "Unknown error");
        throw new Error(errorBody || `API Error: ${response.status} ${response.statusText}`);
      }

      if (response.status === 204) {
        return {} as T;
      }

      return response.json();
    } catch (error: any) {
      lastError = error;

      // Don't retry if the request was aborted by the user
      if (error.name === "AbortError") {
        throw error;
      }

      // Retry on network errors
      if (attempt < retries) {
        const delay = retryDelay * Math.pow(2, attempt);
        await new Promise((resolve) => setTimeout(resolve, delay));
        continue;
      }

      throw error;
    }
  }

  throw lastError;
}
