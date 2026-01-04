import { defineConfig } from "@kubb/core";
import { pluginOas } from "@kubb/plugin-oas";
import { pluginTs } from "@kubb/plugin-ts";
import { pluginClient } from "@kubb/plugin-client";

export default defineConfig({
  input: {
    path: "./openapi.json",
  },
  output: {
    path: "./src/kubb",
    clean: true,
  },
  plugins: [
    pluginOas(),
    pluginTs({
      output: { path: "types" },
    }),
    pluginClient({
      output: { path: "hooks" },
      transformers: {
        name: (name) => `use${name.charAt(0).toUpperCase()}${name.slice(1)}`,
      },
      baseURL: "/api",
    }),
  ],
});
