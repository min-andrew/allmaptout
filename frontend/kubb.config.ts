import { defineConfig } from "@kubb/core";
import { pluginOas } from "@kubb/plugin-oas";
import { pluginTs } from "@kubb/plugin-ts";
import { pluginSvelteQuery } from "@kubb/plugin-svelte-query";

export default defineConfig({
  input: {
    path: "./openapi.json",
  },
  output: {
    path: "./src/api/generated",
    clean: true,
  },
  plugins: [
    pluginOas(),
    pluginTs({
      output: { path: "types" },
    }),
    pluginSvelteQuery({
      output: { path: "hooks" },
      client: {
        importPath: "../client",
      },
    }),
  ],
});
