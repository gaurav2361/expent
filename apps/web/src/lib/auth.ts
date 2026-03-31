import { betterAuth } from "better-auth";
import { tanstackStartCookies } from "better-auth/tanstack-start";
import Database from "better-sqlite3";

/**
 * Server-side authentication instance.
 * This is used in your API routes (e.g., /api/auth/*).
 * 
 * IMPORTANT: This file should ONLY be imported in server-side code
 * to avoid issues with native bindings (better-sqlite3) in the browser.
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
