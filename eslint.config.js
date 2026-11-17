import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";

export default tseslint.config(
  { ignores: ["dist", "node_modules", "src-tauri", "coverage", ".stryker-tmp", "stryker-tmp", "reports"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      globals: { ...globals.browser, ...globals.es2021 },
    },
    plugins: {
      "react-hooks": reactHooks,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
    },
  },
  {
    // Vitest/Node files for tests and tooling
    files: ["**/*.{test,spec}.{ts,tsx}", "src/test/**/*", "vitest.config.ts", "vite.config.ts"],
    languageOptions: {
      globals: { ...globals.node },
    },
  },
);
