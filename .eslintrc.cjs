module.exports = {
  root: true,
  env: { browser: true, es2021: true, node: true },
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module',
    extraFileExtensions: ['.svelte'],
    project: './tsconfig.json'
  },
  plugins: ['@typescript-eslint', 'svelte'],
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:svelte/recommended',
    'prettier'
  ],
  rules: {
    '@typescript-eslint/no-explicit-any': 'warn',
    'no-console': 'off'
  },
  overrides: [
    {
      files: ['*.svelte'],
      parser: 'svelte-eslint-parser',
      parserOptions: { parser: '@typescript-eslint/parser' }
    }
  ]
};
