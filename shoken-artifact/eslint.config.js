import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'
import { defineConfig, globalIgnores } from 'eslint/config'

export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      reactHooks.configs.flat.recommended,
      reactRefresh.configs.vite,
    ],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
  },
  {
    // shadcn/ui コンポーネントは CVA 等の非コンポーネント値も export するため
    // react-refresh のルールを除外する
    files: ['src/components/ui/**/*.{ts,tsx}', 'src/hooks/use-toast.ts'],
    rules: {
      'react-refresh/only-export-components': 'off',
      '@typescript-eslint/no-unused-vars': 'off',
    },
  },
])
