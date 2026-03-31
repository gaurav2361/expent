import { betterAuth } from "better-auth";
import { createAuthClient } from "better-auth/react";
import { tanstackStartCookies } from "better-auth/tanstack-start";
import Database from "better-sqlite3";

/**
 * Server-side authentication instance.
 * This is used in your API routes (e.g., /api/auth/*).
 */
export const auth = betterAuth({
  database: new Database(process.env.DATABASE_URL?.replace("sqlite:", "") || "expent.db"),
  // Fix for "Base URL could not be determined" warning
  baseURL: process.env.BETTER_AUTH_URL || "http://localhost:3000",
  emailAndPassword: {
    enabled: true,
  },
  socialProviders: {
    // Only enable github if environment variables are present to avoid warnings
    ...(process.env.GITHUB_CLIENT_ID && process.env.GITHUB_CLIENT_SECRET
      ? {
          github: {
            clientId: process.env.GITHUB_CLIENT_ID,
            clientSecret: process.env.GITHUB_CLIENT_SECRET,
          },
        }
      : {}),
  },
  plugins: [tanstackStartCookies()],
});

/**
 * Client-side authentication client.
 * This is used in your React components and hooks.
 */
export const authClient = createAuthClient({
  baseURL: import.meta.env.VITE_AUTH_BASE_URL || "http://localhost:3000",
});

// Export hooks and methods directly for convenience
export const { signIn, signUp, useSession, signOut } = authClient;
