module.exports = {
  plugins: ['unused-imports'],
  rules: {
    '@typescript-eslint/consistent-type-definitions': ['error', 'type'],
    'arrow-body-style': ['error', 'as-needed'],
    'no-undef': 'off',
    'unused-imports/no-unused-imports': 'error',
  },
  extends: ['alloy', 'alloy/typescript'],
}
