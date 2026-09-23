import { defineConfig, devices } from '@playwright/test';

/**
 * Leptos PoC 用の実験設定（本番の playwright.ci.config.ts は変更しない）
 *
 * - 対象は frontend/e2e の各 spec を無改修で実行する
 * - error-scenarios は CSV ケース（CSV画面は #884/#885 の範囲）を除く
 * - baseURL は Trunk dev サーバ（frontend-leptos）を指す
 */
export default defineConfig({
  testDir: '../frontend/e2e',
  testMatch: [
    '**/search-flow.spec.ts',
    '**/receipt-flow.spec.ts',
    '**/auth-flow.spec.ts',
    '**/error-scenarios.spec.ts',
  ],
  grepInvert: /CSV プレビュー/,
  outputDir: '/tmp/leptos-e2e-out',
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
    command: 'trunk serve --port 8081 --no-autoreload',
    url: 'http://127.0.0.1:8081',
    reuseExistingServer: true,
    timeout: 300_000,
  },
});
