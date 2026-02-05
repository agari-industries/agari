import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// https://vite.dev/config/
export default defineConfig(() => ({
  // Use root base path for custom domain (agari.org)
  base: "/",
  plugins: [svelte()],
  build: {
    target: "esnext",
  },
  optimizeDeps: {
    exclude: ["agari-wasm"],
  },
  server: {
    fs: {
      // Allow serving files from the wasm output directory
      allow: [".."],
    },
  },
}));
