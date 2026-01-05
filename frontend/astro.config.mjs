import { defineConfig } from "astro/config";
import svelte from "@astrojs/svelte";
import tailwind from "@astrojs/tailwind";

export default defineConfig({
  integrations: [
    svelte({
      compilerOptions: {
        runes: false,
      },
    }),
    tailwind(),
  ],
  output: "server",
  server: {
    port: 3000,
  },
  vite: {
    server: {
      proxy: {
        "/api": {
          target: "http://localhost:3001",
          changeOrigin: true,
          rewrite: (path) => path.replace(/^\/api/, ""),
        },
      },
    },
    optimizeDeps: {
      include: ["axios", "@kubb/plugin-client/clients/axios"],
    },
  },
});
