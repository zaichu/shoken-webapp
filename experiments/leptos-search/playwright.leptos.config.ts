import { defineConfig, devices } from '@playwright/test';

/**
 * Leptos PoC 用の実験設定（本番の playwright.ci.config.ts は変更しない）
 *
 * - 対象は frontend/e2e/search-flow.spec.ts を無改修で実行する
 * - baseURL は Trunk dev サーバ（experiments/leptos-search）を指す
 */
export default defineConfig({
  testDir: '../../frontend/e2e',
  testMatch: ['**/search-flow.spec.ts'],
  fullyParallel: false,
  forbidOnly: true,
  retries: 0,
  workers: 1,
  reporter: [['list']],
  use: {
    baseURL: 'http://127.0.0.1:8081',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'trunk serve --port 8081',
    url: 'http://127.0.0.1:8081',
    reuseExistingServer: true,
    timeout: 300_000,
  },
});
