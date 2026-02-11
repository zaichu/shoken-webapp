import { test, type Page } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

/**
 * UIレビュー用スクリーンショット取得
 *
 * 使用方法:
 * 1. 開発サーバーを起動: npm run dev
 * 2. スクショ取得: npm run ui:screenshot
 *
 * ログインが必要な場合:
 * 1. 手動でブラウザを開きログイン
 * 2. npm run ui:save-auth でログイン状態を保存
 * 3. npm run ui:screenshot:auth で実行
 */

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const SCREENSHOT_DIR = path.join(__dirname, '../../.playwright-mcp');
const TABS = [
  { name: 'dividend', label: '配当金', index: 0 },
  { name: 'domestic-stock', label: '国内株式', index: 1 },
  { name: 'mutualfund', label: '投資信託', index: 2 },
] as const;

async function waitForPageReady(page: Page) {
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(1000);
}

async function selectFirstNonDefaultOption(
  page: Page,
  selector: string
) {
  const select = page.locator(selector);
  if (!(await select.isVisible())) {
    return;
  }

  const options = await select.locator('option').allTextContents();
  const option = options.find(opt => opt !== '全て表示' && opt !== '');
  if (!option) {
    return;
  }

  await select.selectOption({ label: option });
  await page.waitForTimeout(1500);
}

// スクショ保存ディレクトリを作成
test.beforeAll(async () => {
  if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  }
});

test.describe('主要ページ', () => {
  test('ホームページ - 初期表示', async ({ page }) => {
    await page.goto('/');
    await waitForPageReady(page);

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'home-initial.png'),
      fullPage: true,
    });
  });

  test('銘柄検索 - 任天堂検索', async ({ page }) => {
    await page.goto('/search');
    await waitForPageReady(page);

    const codeInput = page.getByLabel('銘柄コード');
    await codeInput.fill('任天堂');
    await page.getByRole('button', { name: '検索' }).click();
    await waitForPageReady(page);

    const hasNintendoResult = await page
      .locator('text=任天堂')
      .first()
      .isVisible({ timeout: 2000 })
      .catch(() => false);

    // API側が銘柄名検索に対応していない環境でも、任天堂の検索結果を取得するために銘柄コードで再検索
    if (!hasNintendoResult) {
      await codeInput.fill('7974');
      await page.getByRole('button', { name: '検索' }).click();
      await waitForPageReady(page);
      await page.waitForTimeout(1000);
    }

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'search-nintendo-result.png'),
      fullPage: true,
    });
  });

  test('資産管理 - 初期表示', async ({ page }) => {
    await page.goto('/assetbalance');
    await waitForPageReady(page);
    await page.waitForTimeout(2000);

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'assetbalance-initial.png'),
      fullPage: true,
    });
  });

  test('資産管理 - 銘柄検索', async ({ page }) => {
    await page.goto('/assetbalance');
    await waitForPageReady(page);
    await page.waitForTimeout(2000);

    await selectFirstNonDefaultOption(page, '#securities-search');

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'assetbalance-search-security.png'),
      fullPage: true,
    });
  });
});

// 各タブの初期表示をスクショ
test.describe('取引明細ページ - タブ初期表示', () => {
  for (const tab of TABS) {
    test(`${tab.label}タブの初期表示`, async ({ page }) => {
      await page.goto('/receipts');
      await waitForPageReady(page);

      // タブをクリック
      if (tab.index > 0) {
        const tabButton = page.getByRole('tab', { name: tab.label });
        await tabButton.click();
        await waitForPageReady(page);
      }

      // データ読み込み待機（最大5秒）
      await page.waitForTimeout(1000);

      // スクショ取得
      await page.screenshot({
        path: path.join(SCREENSHOT_DIR, `receipts-${tab.name}-initial.png`),
        fullPage: true,
      });
    });
  }
});

// 各タブの検索結果をスクショ
test.describe('取引明細ページ - 検索結果', () => {
  test('配当金 - 西暦検索', async ({ page }) => {
    await page.goto('/receipts');
    await waitForPageReady(page);

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
    await waitForPageReady(page);

    // 銘柄セレクトが表示されていれば選択
    await selectFirstNonDefaultOption(page, '#securities-search');

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'receipts-dividend-search-security.png'),
      fullPage: true,
    });
  });

  test('国内株式 - 西暦検索', async ({ page }) => {
    await page.goto('/receipts');
    await waitForPageReady(page);

    // 国内株式タブをクリック
    await page.getByRole('tab', { name: '国内株式' }).click();
    await waitForPageReady(page);

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
    await waitForPageReady(page);

    // 国内株式タブをクリック
    await page.getByRole('tab', { name: '国内株式' }).click();
    await waitForPageReady(page);

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
    await waitForPageReady(page);

    // 投資信託タブをクリック
    await page.getByRole('tab', { name: '投資信託' }).click();
    await waitForPageReady(page);

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
    await waitForPageReady(page);

    // 投資信託タブをクリック
    await page.getByRole('tab', { name: '投資信託' }).click();
    await waitForPageReady(page);

    // ファンドセレクトが表示されていれば選択
    await selectFirstNonDefaultOption(page, '#securities-search');

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, 'receipts-mutualfund-search-fund.png'),
      fullPage: true,
    });
  });
});
