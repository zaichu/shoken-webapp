import { defineConfig, devices } from '@playwright/test';

/**
 * Receipts挙動検証用の実験設定（PoC専用に新規作成したspecを実行する）
 *
 * - testDir は frontend-leptos/e2e を指す
 * - リクエスト回数・順序でキャッシュ挙動を検証するため API モックは各spec内で行う
 */
export default defineConfig({
  testDir: './e2e',
  testMatch: ['**/*.spec.ts'],
  outputDir: '/tmp/leptos-receipts-e2e-out',
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
