import { defineConfig, devices } from '@playwright/test';

const port = process.env.VERCEL_E2E_PORT ?? '8190';
const dist = process.env.VERCEL_DIST_DIR ?? 'dist-vercel';
const baseURL = `http://127.0.0.1:${port}`;

export default defineConfig({
  testDir: './e2e-vercel',
  outputDir: `${process.env.LEPTOS_E2E_OUTPUT_DIR || 'test-results'}/vercel`,
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
    command: `node scripts/prepare-vercel-dist.mjs ${dist} && node scripts/serve-dist.mjs ${dist} ${port}`,
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
  },
});
