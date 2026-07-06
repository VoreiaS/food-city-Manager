// ESLint 9 flat config for Food City frontend.
// Run: `npm run lint`
import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";

export default tseslint.config(
  // Base JS recommended rules
  js.configs.recommended,

  // TypeScript recommended (strict)
  ...tseslint.configs.recommended,
  ...tseslint.configs.strict,

  // Global ignores
  {
    ignores: ["dist/", "node_modules/", "vite.config.ts", "*.config.js"],
  },

  // Main config for all TS/TSX files
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2022,
      globals: {
        ...globals.browser,
        ...globals.es2021,
      },
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      // React Hooks — required for correct React behavior
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",

      // React Refresh — only warn on constant exports (HMR issue)
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],

      // TypeScript — relax some strict rules that conflict with our patterns
      "@typescript-eslint/no-explicit-any": "warn", // warn, not error (WS handlers use any)
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
      "@typescript-eslint/consistent-type-imports": [
        "warn",
        { prefer: "type-imports" },
      ],

      // General code quality
      "no-console": ["warn", { allow: ["warn", "error", "info", "debug"] }],
      "no-debugger": "error",
      "prefer-const": "error",
      "no-var": "error",
    },
  },

  // Test files — allow console and relax some rules
  {
    files: ["src/**/*.test.{ts,tsx}", "src/**/*.spec.{ts,tsx}"],
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
      "no-console": "off",
    },
  },
);
