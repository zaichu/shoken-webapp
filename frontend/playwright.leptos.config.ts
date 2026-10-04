import { defineConfig, devices } from '@playwright/test';

const port = process.env.LEPTOS_E2E_PORT ?? '8081';
const baseURL = `http://127.0.0.1:${port}`;
// 固定パスや他 suite との入れ子だと、実行開始時の掃除で互いの成果物を消し合う
const outputBase = process.env.LEPTOS_E2E_OUTPUT_DIR || 'test-results';
const outputDir = `${outputBase}/leptos`;
// 並行して serve を立てたとき既定の dist/ を共有すると成果物が混ざるので、実行ごとに分ける
const distDir = `${outputBase}/dist-leptos`;
// CI ではビルド済み dist を静的配信し、wasm のビルドを1回にする。API origin は空に固定して
// 同一オリジン化し、モックに当たらない呼び出しが backend に出ないようにする。未指定なら trunk serve
const prebuiltDist = process.env.LEPTOS_E2E_DIST_DIR;

const screenTests = [
  '**/stock-search/flow.spec.ts',
  '**/receipts/flow.spec.ts',
  '**/session/auth-flow.spec.ts',
  '**/csv/error-scenarios.spec.ts',
  '**/asset-balance/flow.spec.ts',
  '**/accessibility/a11y.spec.ts',
];

export default defineConfig({
  testDir: './e2e',
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
      testMatch: screenTests,
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'leptos-e2e',
      testMatch: ['**/*.spec.ts'],
      testIgnore: [...screenTests, '**/deploy/**'],
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: prebuiltDist
      ? `node scripts/prepare-vercel-dist.mjs ${prebuiltDist} '' && node scripts/serve-dist.mjs ${prebuiltDist} ${port}`
      // テスト成果物や spec の変更で再ビルドが走ると dist 差し替えでページ読み込みが落ちるので、watch はアプリの入力だけに絞る
      : `trunk serve --port ${port} --dist ${distDir} --no-autoreload --enable-cooldown --watch src --watch index.html --watch style/input.css --watch Trunk.toml --watch Cargo.toml --watch Cargo.lock`,
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    timeout: prebuiltDist ? 60_000 : 300_000,
  },
});
