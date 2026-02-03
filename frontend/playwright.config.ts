import { defineConfig, devices } from '@playwright/test';

/**
 * UIレビュー用スクリーンショット取得のためのPlaywright設定
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: 'list',
  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:8081',
    trace: 'off',
    screenshot: 'off',
    // ログイン状態を保持するためのstorageState
    storageState: process.env.STORAGE_STATE || undefined,
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  // 開発サーバーは手動で起動する前提
  webServer: undefined,
});
