import { defineConfig, devices } from '@playwright/test';

/**
 * Leptos PoC 用の実験設定（本番の playwright.ci.config.ts は変更しない）
 *
 * - 対象は frontend/e2e の各 spec を無改修で実行する
 * - baseURL は Trunk dev サーバ（frontend-leptos）を指す
 */
const port = process.env.LEPTOS_E2E_PORT ?? '8081';
const baseURL = `http://127.0.0.1:${port}`;
// 固定パスや receipts との入れ子だと、実行開始時の掃除で互いの成果物を消し合う
const outputDir = `${process.env.LEPTOS_E2E_OUTPUT_DIR || 'test-results'}/leptos`;

export default defineConfig({
  testDir: '../frontend/e2e',
  testMatch: [
    '**/search-flow.spec.ts',
    '**/receipt-flow.spec.ts',
    '**/auth-flow.spec.ts',
    '**/error-scenarios.spec.ts',
    '**/assetbalance-flow.spec.ts',
    '**/mobile-screenshots.spec.ts',
    '**/a11y.spec.ts',
  ],
  outputDir,
  fullyParallel: false,
  forbidOnly: true,
  retries: 0,
  workers: 1,
  reporter: [['list']],
  use: {
    baseURL,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'leptos-e2e',
      testDir: './e2e',
      testMatch: ['**/*.spec.ts'],
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: `trunk serve --port ${port} --no-autoreload`,
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
