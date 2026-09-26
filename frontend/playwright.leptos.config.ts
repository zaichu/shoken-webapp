import { defineConfig, devices } from '@playwright/test';

const port = process.env.LEPTOS_E2E_PORT ?? '8081';
const baseURL = `http://127.0.0.1:${port}`;
// 固定パスや他 suite との入れ子だと、実行開始時の掃除で互いの成果物を消し合う
const outputBase = process.env.LEPTOS_E2E_OUTPUT_DIR || 'test-results';
const outputDir = `${outputBase}/leptos`;
// 並行して serve を立てたとき既定の dist/ を共有すると成果物が混ざるので、実行ごとに分ける
const distDir = `${outputBase}/dist-leptos`;

export default defineConfig({
  testDir: './e2e/migrated',
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
  // wasm の取得・コンパイル・hydrate が初回 goto 後のアサーションに乗るため、CI の低負荷時も見て長めに取る
  expect: { timeout: 15_000 },
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
      testIgnore: ['**/migrated/**'],
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    // テスト成果物や spec の変更で再ビルドが走ると dist 差し替えでページ読み込みが落ちるので、watch はアプリの入力だけに絞る
    command: `trunk serve --port ${port} --dist ${distDir} --no-autoreload --enable-cooldown --watch src --watch index.html --watch style/input.css --watch Trunk.toml --watch Cargo.toml --watch Cargo.lock`,
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
