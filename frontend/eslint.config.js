import eslint from "@eslint/js";
import astro from "eslint-plugin-astro";
import svelte from "eslint-plugin-svelte";

export default [
  eslint.configs.recommended,
  ...astro.configs.recommended,
  ...svelte.configs["flat/recommended"],
  { 
	  ignores: ["dist/", ".astro/", "src/api/generated/"]
  }, {
	  files: ["**/*.astro"],
	  rules: {},
},
];
