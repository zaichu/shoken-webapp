import { test } from '@playwright/test';
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
 * 1. npm run ui:save-auth でログイン状態を保存
 * 2. npm run ui:screenshot:auth で実行
 *
 * 取得するスクリーンショット:
 * - receipts-dividend.png: 配当金タブの初期表示
 * - receipts-dividend-search.png: 配当金 - 銘柄検索結果（配当シミュレーション表示）
 * - receipts-domestic-stock.png: 国内株式タブの初期表示
 * - receipts-mutualfund.png: 投資信託タブの初期表示
 */

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const SCREENSHOT_DIR = path.join(__dirname, '../../.playwright-mcp');

// スクショ保存ディレクトリを作成し、既存画像を削除
test.beforeAll(async () => {
  if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  }
  // 既存のスクリーンショットを削除
  const files = fs.readdirSync(SCREENSHOT_DIR);
  for (const file of files) {
    if (file.endsWith('.png')) {
      fs.unlinkSync(path.join(SCREENSHOT_DIR, file));
    }
  }
});

test.describe('受取金ページ - スクリーンショット', () => {
  test('配当金タブの初期表示', async ({ page }) => {
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'receipts-dividend.png'),
      fullPage: true,
    });
  });

  test('配当金 - 銘柄検索（配当シミュレーション表示）', async ({ page }) => {
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
      path: path.join(SCREENSHOT_DIR, 'receipts-dividend-search.png'),
      fullPage: true,
    });
  });

  test('国内株式タブの初期表示', async ({ page }) => {
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');

    // 国内株式タブをクリック
    await page.getByRole('tab', { name: '国内株式' }).click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'receipts-domestic-stock.png'),
      fullPage: true,
    });
  });

  test('投資信託タブの初期表示', async ({ page }) => {
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');

    // 投資信託タブをクリック
    await page.getByRole('tab', { name: '投資信託' }).click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'receipts-mutualfund.png'),
      fullPage: true,
    });
  });
});
