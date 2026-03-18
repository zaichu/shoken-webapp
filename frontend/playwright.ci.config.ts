import { defineConfig, devices } from '@playwright/test';

/**
 * CI 用 Playwright 設定
 *
 * - Vite dev server を自動起動
 * - API モック前提の E2E spec を実行
 * - 失敗時に trace / screenshot / HTML レポートを保存
 */
export default defineConfig({
  testDir: './e2e',
  testMatch: [
    '**/error-scenarios.spec.ts',
    '**/a11y.spec.ts',
    '**/auth-flow.spec.ts',
    '**/receipt-flow.spec.ts',
    '**/search-flow.spec.ts',
  ],
  fullyParallel: false,
  forbidOnly: true,
  retries: 1,
  workers: 1,
  reporter: [['html', { open: 'never', outputFolder: 'playwright-report' }], ['list']],
  use: {
    baseURL: 'http://127.0.0.1:8080',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'npx vite --host 127.0.0.1 --port 8080 --strictPort',
    url: 'http://127.0.0.1:8080',
    reuseExistingServer: false,
    timeout: 60_000,
    env: {
      // バックエンドは page.route() でモックするため実際に接続しない
      VITE_SHOKEN_WEBAPI_API_URL: 'http://localhost:3001',
      PLAYWRIGHT_TEST: '1',
      // React Compiler を無効化（VITEST フラグを流用）
      VITEST: '1',
    },
  },
});
