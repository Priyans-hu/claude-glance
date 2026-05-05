// Minimal ESLint config — kept light intentionally; expand as the codebase grows.
import js from "@eslint/js";

export default [
  js.configs.recommended,
  {
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
      globals: {
        window: "readonly",
        document: "readonly",
        console: "readonly",
        process: "readonly",
      },
    },
    rules: {
      "no-unused-vars": ["warn", { argsIgnorePattern: "^_" }],
    },
  },
  {
    ignores: [
      "node_modules/",
      "build/",
      ".svelte-kit/",
      "src-tauri/target/",
      "src-tauri/gen/",
      "static/",
      "**/*.svelte",
      "**/*.ts",
    ],
  },
];
