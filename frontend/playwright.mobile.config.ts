import { defineConfig, devices } from '@playwright/test';

/**
 * スマホ表示レビュー用 Playwright 設定
 *
 * - API は page.route() でモックするためバックエンド・ログイン不要
 * - Vite dev server を自動起動
 * - iPhone 14 相当（390x844）で `.playwright-mcp/mobile-*.png` を保存
 */
export default defineConfig({
  testDir: './e2e',
  testMatch: ['**/mobile-screenshots.spec.ts'],
  fullyParallel: false,
  retries: 0,
  workers: 1,
  reporter: 'list',
  use: {
    // devices['iPhone 14'] は webkit 前提のため、viewport だけ借りて chromium で動かす
    ...devices['Desktop Chrome'],
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: 3,
    isMobile: true,
    hasTouch: true,
    baseURL: 'http://127.0.0.1:8080',
    trace: 'off',
    screenshot: 'off',
  },
  webServer: {
    command: 'npx vite --host 127.0.0.1 --port 8080 --strictPort',
    url: 'http://127.0.0.1:8080',
    // 別 worktree で起動中の dev server を再利用すると、そのブランチのコードを
    // 撮ってしまい修正前後の比較が無意味になる。毎回起動する。
    reuseExistingServer: false,
    timeout: 60_000,
    env: {
      VITE_SHOKEN_WEBAPI_API_URL: 'http://localhost:3001',
      PLAYWRIGHT_TEST: '1',
      VITEST: '1',
    },
  },
});
