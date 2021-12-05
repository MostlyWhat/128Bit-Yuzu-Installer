module.exports = {
  root: true,
  env: {
    node: true
  },
  extends: [
    'plugin:vue/essential',
    '@vue/standard'
  ],
  rules: {
    'no-console': 'off',
    'no-debugger': process.env.NODE_ENV === 'production' ? 'error' : 'off',
    'no-redeclare': 'off',
    camelcase: 'off'
  },
  parserOptions: {
    parser: 'babel-eslint'
  }
}
