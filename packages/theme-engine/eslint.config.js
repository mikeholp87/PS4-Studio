import js from '@eslint/js'
import ts from 'typescript-eslint'
export default ts.config(
  { ignores: ['node_modules', 'src/validator.generated.js'] },
  js.configs.recommended,
  ...ts.configs.recommended,
  { files: ['scripts/**'], languageOptions: { globals: { console: 'readonly', process: 'readonly', URL: 'readonly' } } },
)
