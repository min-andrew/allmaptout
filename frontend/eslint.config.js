import eslint from "@eslint/js";
import astro from "eslint-plugin-astro";
import svelte from "eslint-plugin-svelte";

export default [
  eslint.configs.recommended,
  ...astro.configs.recommended,
  ...svelte.configs["flat/recommended"],
  {
    ignores: ["dist/", ".astro/", "src/kubb/"],
  },
  {
    files: ["**/*.svelte"],
    languageOptions: {
      globals: {
        window: "readonly",
        document: "readonly",
      },
    },
  },
  {
    files: ["**/*.astro"],
    rules: {},
  },
];
