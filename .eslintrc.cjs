module.exports = {
    root: true,
    parser: "@typescript-eslint/parser",
    extends: [
        "plugin:@typescript-eslint/recommended",
        "prettier",
        "plugin:svelte/recommended",
        "plugin:svelte/prettier"
    ],
    plugins: ["@typescript-eslint"],
    ignorePatterns: ["*.cjs"],
    overrides: [
        {
            files: ["*.svelte"],
            parser: "svelte-eslint-parser",
            parserOptions: {
                parser: "@typescript-eslint/parser"
            }
        }
    ],
    parserOptions: {
        sourceType: "module",
        ecmaVersion: 2020,
        project: "./tsconfig.json",
        extraFileExtensions: [".svelte"]
    },
    rules: {
        semi: ["error", "always"],
        "@typescript-eslint/no-explicit-any": "off"
    },
    env: {
        browser: true,
        es2017: true,
        node: true
    }
};
