import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

/**
 * 受取金ページのUIレビュー用スクリーンショット取得
 *
 * 使用方法:
 * 1. 開発サーバーを起動: npm run dev
 * 2. スクショ取得: npm run ui:screenshot
 *
 * ログインが必要な場合:
 * 1. 手動でブラウザを開きログイン
 * 2. npm run ui:save-auth でログイン状態を保存
 * 3. npm run ui:screenshot で実行
 */

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const SCREENSHOT_DIR = path.join(__dirname, '../../.playwright-mcp');
const TABS = [
  { name: 'dividend', label: '配当金', index: 0 },
  { name: 'domestic-stock', label: '国内株式', index: 1 },
  { name: 'mutualfund', label: '投資信託', index: 2 },
] as const;

// スクショ保存ディレクトリを作成
test.beforeAll(async () => {
  if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  }
});

// 各タブの初期表示をスクショ
test.describe('受取金ページ - タブ初期表示', () => {
  for (const tab of TABS) {
    test(`${tab.label}タブの初期表示`, async ({ page }) => {
      await page.goto('/receipts');
      await page.waitForLoadState('networkidle');

      // タブをクリック
      if (tab.index > 0) {
        const tabButton = page.getByRole('tab', { name: tab.label });
        await tabButton.click();
        await page.waitForLoadState('networkidle');
      }

      // データ読み込み待機（最大5秒）
      await page.waitForTimeout(2000);

      // スクショ取得
      await page.screenshot({
        path: path.join(SCREENSHOT_DIR, `receipts-${tab.name}-initial.png`),
        fullPage: true,
      });
    });
  }
});

// 各タブの検索結果をスクショ
test.describe('受取金ページ - 検索結果', () => {
  test('配当金 - 西暦検索', async ({ page }) => {
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // 検索オプションが表示されるまで待機
    const yearSelect = page.locator('#years-search');
    if (await yearSelect.isVisible()) {
      // 最初の年度オプションを選択（全て表示以外）
      const options = await yearSelect.locator('option').allTextContents();
      const yearOption = options.find(opt => /^\d{4}年$/.test(opt));
      if (yearOption) {
        await yearSelect.selectOption({ label: yearOption });
        await page.waitForTimeout(1500);
      }
    }

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'receipts-dividend-search-year.png'),
      fullPage: true,
    });
  });

  test('配当金 - 銘柄検索', async ({ page }) => {
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // 銘柄セレクトが表示されていれば選択
    const securitySelect = page.locator('#securities-search');
    if (await securitySelect.isVisible()) {
      const options = await securitySelect.locator('option').allTextContents();
      // 「全て表示」以外の最初のオプションを選択
      const securityOption = options.find(opt => opt !== '全て表示' && opt !== '');
      if (securityOption) {
        await securitySelect.selectOption({ label: securityOption });
        await page.waitForTimeout(1500);
      }
    }

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'receipts-dividend-search-security.png'),
      fullPage: true,
    });
  });

  test('国内株式 - 西暦検索', async ({ page }) => {
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');

    // 国内株式タブをクリック
    await page.getByRole('tab', { name: '国内株式' }).click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // 西暦検索
    const yearSelect = page.locator('#years-search');
    if (await yearSelect.isVisible()) {
      const options = await yearSelect.locator('option').allTextContents();
      const yearOption = options.find(opt => /^\d{4}年$/.test(opt));
      if (yearOption) {
        await yearSelect.selectOption({ label: yearOption });
        await page.waitForTimeout(1500);
      }
    }

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'receipts-domestic-stock-search-year.png'),
      fullPage: true,
    });
  });

  test('国内株式 - 口座検索', async ({ page }) => {
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');

    // 国内株式タブをクリック
    await page.getByRole('tab', { name: '国内株式' }).click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // 口座ボタンが表示されていればクリック
    const accountButtons = page.locator('button:has-text("特定")');
    if (await accountButtons.first().isVisible()) {
      await accountButtons.first().click();
      await page.waitForTimeout(1500);
    }

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'receipts-domestic-stock-search-account.png'),
      fullPage: true,
    });
  });

  test('投資信託 - 西暦検索', async ({ page }) => {
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');

    // 投資信託タブをクリック
    await page.getByRole('tab', { name: '投資信託' }).click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // 西暦検索
    const yearSelect = page.locator('#years-search');
    if (await yearSelect.isVisible()) {
      const options = await yearSelect.locator('option').allTextContents();
      const yearOption = options.find(opt => /^\d{4}年$/.test(opt));
      if (yearOption) {
        await yearSelect.selectOption({ label: yearOption });
        await page.waitForTimeout(1500);
      }
    }

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'receipts-mutualfund-search-year.png'),
      fullPage: true,
    });
  });

  test('投資信託 - ファンド検索', async ({ page }) => {
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');

    // 投資信託タブをクリック
    await page.getByRole('tab', { name: '投資信託' }).click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // ファンドセレクトが表示されていれば選択
    const securitySelect = page.locator('#securities-search');
    if (await securitySelect.isVisible()) {
      const options = await securitySelect.locator('option').allTextContents();
      const fundOption = options.find(opt => opt !== '全て表示' && opt !== '');
      if (fundOption) {
        await securitySelect.selectOption({ label: fundOption });
        await page.waitForTimeout(1500);
      }
    }

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'receipts-mutualfund-search-fund.png'),
      fullPage: true,
    });
  });
});
