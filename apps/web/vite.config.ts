import contentCollections from "@content-collections/vite";
import tailwindcss from "@tailwindcss/vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite";

// We removed vite-tsconfig-paths from the imports

export default defineConfig({
  plugins: [nitro(), tailwindcss(), tanstackStart(), viteReact(), contentCollections()],
  resolve: {
    // This tells Vite 8 to natively handle your tsconfig paths,
    // which works much better across monorepo workspace boundaries.
    tsconfigPaths: true,
  },
});
