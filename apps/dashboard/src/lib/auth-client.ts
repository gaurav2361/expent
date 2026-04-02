import { createAuthClient } from "better-auth/react";
import { passkeyClient } from "@better-auth/passkey/client";
import { usernameClient } from "better-auth/client/plugins";

/**
 * Client-side authentication client.
 * Pointing to the Rust server's /api/auth endpoints.
 */
export const authClient = createAuthClient({
  baseURL: (process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8080") + "/api/auth",
  plugins: [passkeyClient(), usernameClient()],
  fetchOptions: {
    onError: async (context) => {
      const { response } = context;
      if (response.status === 429) {
        const retryAfter = response.headers.get("X-Retry-After");
        console.log(`Rate limit exceeded. Retry after ${retryAfter} seconds`);
      }
    },
  },
});

export const { signIn, signUp, useSession, signOut } = authClient;
