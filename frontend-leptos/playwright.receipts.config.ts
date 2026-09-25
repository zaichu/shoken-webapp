import { defineConfig, devices } from '@playwright/test';

/**
 * Receipts挙動検証用の実験設定（PoC専用に新規作成したspecを実行する）
 *
 * - testDir は frontend-leptos/e2e を指す
 * - リクエスト回数・順序でキャッシュ挙動を検証するため API モックは各spec内で行う
 */
const port = process.env.LEPTOS_E2E_PORT ?? '8081';
const baseURL = `http://127.0.0.1:${port}`;
// /tmp 固定だと worktree 間で共有され、並行実行時に互いの trace を消し合って落ちる
const outputDir = process.env.LEPTOS_E2E_OUTPUT_DIR ?? 'test-results/receipts';

export default defineConfig({
  testDir: './e2e',
  testMatch: ['**/*.spec.ts'],
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
  ],
  webServer: {
    command: `trunk serve --port ${port} --no-autoreload`,
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
