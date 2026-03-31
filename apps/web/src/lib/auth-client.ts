import { createAuthClient } from "better-auth/react";

/**
 * Client-side authentication client.
 * This is used in your React components and hooks.
 */
export const authClient = createAuthClient({
  baseURL: import.meta.env.VITE_AUTH_BASE_URL || "http://localhost:3000",
});

export const { signIn, signUp, useSession, signOut } = authClient;
