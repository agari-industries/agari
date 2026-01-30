import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  build: {
    target: 'esnext',
  },
  optimizeDeps: {
    exclude: ['agari-wasm'],
  },
  server: {
    fs: {
      // Allow serving files from the wasm output directory
      allow: ['..'],
    },
  },
});
