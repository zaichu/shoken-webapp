import { defineConfig, devices } from '@playwright/test';

/**
 * Leptos PoC 用の実験設定（本番の playwright.ci.config.ts は変更しない）
 *
 * - 対象は frontend/e2e の各 spec を無改修で実行する
 * - error-scenarios は CSV ケース（CSV画面は #884/#885 の範囲）を除く
 * - assetbalance-flow は EmptyState と検索・クリアのみ（CSV・削除は #918/#919 の範囲）
 * - baseURL は Trunk dev サーバ（frontend-leptos）を指す
 */
const port = process.env.LEPTOS_E2E_PORT ?? '8081';
const baseURL = `http://127.0.0.1:${port}`;

export default defineConfig({
  testDir: '../frontend/e2e',
  testMatch: [
    '**/search-flow.spec.ts',
    '**/receipt-flow.spec.ts',
    '**/auth-flow.spec.ts',
    '**/error-scenarios.spec.ts',
    '**/assetbalance-flow.spec.ts',
  ],
  grepInvert: /CSV プレビュー|保存ボタンが活性化|再取得されて銘柄データ|サマリーと検索オプション|全件削除で EmptyState/,
  outputDir: '/tmp/leptos-e2e-out',
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
  ],
  webServer: {
    command: `trunk serve --port ${port} --no-autoreload`,
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
