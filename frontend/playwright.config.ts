import { defineConfig, devices } from '@playwright/test';

/**
 * UIレビュー用スクリーンショット取得のためのPlaywright設定
 */
const FHD_VIEWPORT = { width: 1920, height: 1080 } as const;
const UHD_4K_VIEWPORT = { width: 3840, height: 2160 } as const;

function resolveViewport() {
  const mode = (process.env.UI_REVIEW_VIEWPORT || 'fhd').toLowerCase();
  return mode === '4k' || mode === 'uhd' ? UHD_4K_VIEWPORT : FHD_VIEWPORT;
}

const viewport = resolveViewport();

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: 'list',
  use: {
    baseURL: process.env.BASE_URL || 'http://127.0.0.1:8080',
    trace: 'off',
    screenshot: 'off',
    // ログイン状態を保持するためのstorageState
    storageState: process.env.STORAGE_STATE || undefined,
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'], viewport },
    },
  ],
  // 開発サーバーは手動で起動する前提
  webServer: undefined,
});
