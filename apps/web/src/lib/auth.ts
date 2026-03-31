import { betterAuth } from "better-auth";
import { createAuthClient } from "better-auth/react";
import { tanstackStartCookies } from "better-auth/tanstack-start";
import { Pool } from "pg";

/**
 * Server-side authentication instance.
 * This is used in your API routes (e.g., /api/auth/*).
 */
export const auth = betterAuth({
  database: new Pool({
    connectionString: process.env.DATABASE_URL,
  }),
  emailAndPassword: {
    enabled: true,
  },
  socialProviders: {
    github: {
      clientId: process.env.GITHUB_CLIENT_ID as string,
      clientSecret: process.env.GITHUB_CLIENT_SECRET as string,
    },
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

export const { signIn, signUp, useSession, signOut } = authClient;
