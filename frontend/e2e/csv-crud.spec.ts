/**
 * CSV CRUD E2E テスト
 *
 * 使用方法:
 *   npm run ui:csv-crud:auth
 *
 * 前提:
 *   - ローカル環境が起動中 (DB → backend → frontend)
 *   - ログイン状態が .auth/storage-state.json に保存済み
 *   - 起動: ./scripts/start-local.sh
 */
import { test, expect, type Page } from '@playwright/test';
import * as path from 'path';
import * as fs from 'fs';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const FIXTURE_DIR = path.join(__dirname, '__fixtures__/csv');
const SCREENSHOT_DIR = path.join(__dirname, '../../.playwright-mcp');

test.beforeAll(() => {
  if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  }
});

async function waitForPageReady(page: Page) {
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500);
}

/**
 * 全件削除ボタンが表示されていれば削除してクリア状態にする
 */
async function deleteAllIfExists(page: Page) {
  const deleteBtn = page.getByRole('button', { name: /全件削除/ });
  if (await deleteBtn.isVisible({ timeout: 3000 }).catch(() => false)) {
    await deleteBtn.click();
    await page.getByRole('button', { name: '削除する' }).click();
    // 削除完了を確認（全件削除ボタンが消える = 0件）
    await expect(deleteBtn).not.toBeVisible({ timeout: 10000 });
    await waitForPageReady(page);
  }
}

/**
 * CSV をアップロードし、プレビュー完了（保存ボタン活性化）を待つ
 */
async function uploadCsv(page: Page, filename: string) {
  const csvPath = path.join(FIXTURE_DIR, filename);
  await page.locator('#csv-file-input').setInputFiles(csvPath);
  // 保存ボタンが "解析中..." から "N件 〇〇で保存" に変わるまで待機
  const saveBtn = page.getByRole('button', { name: /保存/ });
  await saveBtn.waitFor({ state: 'visible', timeout: 15000 });
  await expect(saveBtn).toBeEnabled({ timeout: 10000 });
}

/**
 * 保存ボタンをクリックし、rawFile がクリアされる（ボタン消滅）まで待つ
 * receipts: その後 importResult に "件登録" が表示される
 */
async function clickSave(page: Page) {
  const saveBtn = page.getByRole('button', { name: /保存/ });
  await saveBtn.click();
  // rawFile = null になりボタンがアンマウントされるまで待機
  await expect(saveBtn).not.toBeVisible({ timeout: 15000 });
}

// =====================================================================
// 国内株式 - 追加と削除
// =====================================================================
test('国内株式 - CSVで3件追加 + 10件追加（商船三井8件新規重複含む）+ 全件削除', async ({ page }) => {
  await page.goto('/receipts');
  await waitForPageReady(page);

  // 国内株式タブへ切り替え
  await page.getByRole('tab', { name: '国内株式' }).click();
  await waitForPageReady(page);

  // 既存データをクリア
  await deleteAllIfExists(page);

  // ── base CSV (3件) をアップロード・保存 ──
  await uploadCsv(page, 'domesticstock-base.csv');
  await clickSave(page);
  await expect(page.getByText(/3件登録/)).toBeVisible({ timeout: 10000 });

  await page.screenshot({
    path: path.join(SCREENSHOT_DIR, 'csv-domesticstock-after-base-import.png'),
    fullPage: true,
  });

  // ── additional CSV (9x商船三井 + 九州電力 + 三菱電機) をアップロード・保存 ──
  // 国内株式は occurrence_index で同一内容の複数行を許容する仕様。
  // base 後の DB には商船三井が1件（index=1）。additional の商船三井9行のうち
  // index=1 は既存とみなしてスキップ、index=2〜9（8件）が新規挿入される。
  await uploadCsv(page, 'domesticstock-additional.csv');
  await clickSave(page);
  await expect(page.getByText(/10件登録/)).toBeVisible({ timeout: 10000 });

  await page.screenshot({
    path: path.join(SCREENSHOT_DIR, 'csv-domesticstock-after-additional-import.png'),
    fullPage: true,
  });

  // ── 全件削除 ──
  await page.getByRole('button', { name: /全件削除/ }).click();
  await page.getByRole('button', { name: '削除する' }).click();
  await expect(page.getByRole('button', { name: /全件削除/ })).not.toBeVisible({ timeout: 10000 });
  await waitForPageReady(page);

  await page.screenshot({
    path: path.join(SCREENSHOT_DIR, 'csv-domesticstock-after-delete.png'),
    fullPage: true,
  });
});

// =====================================================================
// 配当金 - 追加
// =====================================================================
test('配当金 - CSVで3件追加 + 3件追加（KDDI重複2件はスキップ）', async ({ page }) => {
  await page.goto('/receipts');
  await waitForPageReady(page);
  // 配当金タブはデフォルト選択済み

  // 既存データをクリア
  await deleteAllIfExists(page);

  // ── base CSV (3件) ──
  await uploadCsv(page, 'dividend-base.csv');
  await clickSave(page);
  await expect(page.getByText(/3件登録/)).toBeVisible({ timeout: 10000 });

  await page.screenshot({
    path: path.join(SCREENSHOT_DIR, 'csv-dividend-after-base-import.png'),
    fullPage: true,
  });

  // ── additional CSV (3x KDDI + 日本電信電話 + トヨタ) ──
  // 配当金は ON CONFLICT DO NOTHING のため、KDDI 重複2件はスキップ → 3件登録 / 2件スキップ
  await uploadCsv(page, 'dividend-additional.csv');
  await clickSave(page);
  await expect(page.getByText(/3件登録/)).toBeVisible({ timeout: 10000 });

  await page.screenshot({
    path: path.join(SCREENSHOT_DIR, 'csv-dividend-after-additional-import.png'),
    fullPage: true,
  });
});

// =====================================================================
// 資産管理 - 差分更新（全件置換）
// =====================================================================
test('資産管理 - base(2銘柄)→updated(3銘柄)の差分更新', async ({ page }) => {
  await page.goto('/assetbalance');
  await waitForPageReady(page);
  await page.waitForTimeout(1500);

  // 既存データをクリア
  await deleteAllIfExists(page);

  // ── base CSV (INPEX + 任天堂 = 2件) ──
  await uploadCsv(page, 'assetbalance-base.csv');
  // 資産管理は保存後に importResult がないため、ボタン消滅後に全件削除ボタンの件数で確認
  await clickSave(page);
  await expect(page.getByRole('button', { name: /全件削除 \(2件\)/ })).toBeVisible({ timeout: 10000 });

  await page.screenshot({
    path: path.join(SCREENSHOT_DIR, 'csv-assetbalance-after-base-import.png'),
    fullPage: true,
  });

  // ── updated CSV (INPEX + 三菱UFJ + KDDI = 3件) ──
  await uploadCsv(page, 'assetbalance-updated.csv');
  await clickSave(page);
  // 3銘柄に差し替えられたことを確認（保有銘柄数の更新）
  await expect(page.getByRole('button', { name: /全件削除 \(3件\)/ })).toBeVisible({ timeout: 10000 });

  await page.screenshot({
    path: path.join(SCREENSHOT_DIR, 'csv-assetbalance-after-updated-import.png'),
    fullPage: true,
  });
});

// =====================================================================
// 投資信託 - 追加
// =====================================================================
test('投資信託 - CSVで2件追加 + 2件追加（eMAXIS重複1件はスキップ）', async ({ page }) => {
  await page.goto('/receipts');
  await waitForPageReady(page);

  // 投資信託タブへ切り替え
  await page.getByRole('tab', { name: '投資信託' }).click();
  await waitForPageReady(page);

  // 既存データをクリア
  await deleteAllIfExists(page);

  // ── base CSV (2件) ──
  await uploadCsv(page, 'mutualfund-base.csv');
  await clickSave(page);
  await expect(page.getByText(/2件登録/)).toBeVisible({ timeout: 10000 });

  await page.screenshot({
    path: path.join(SCREENSHOT_DIR, 'csv-mutualfund-after-base-import.png'),
    fullPage: true,
  });

  // ── additional CSV (2x eMAXIS S&P500 + eMAXIS 全世界) ──
  // 投資信託は ON CONFLICT DO NOTHING のため、eMAXIS Slim S&P500 重複1件はスキップ → 2件登録 / 1件スキップ
  await uploadCsv(page, 'mutualfund-additional.csv');
  await clickSave(page);
  await expect(page.getByText(/2件登録/)).toBeVisible({ timeout: 10000 });

  await page.screenshot({
    path: path.join(SCREENSHOT_DIR, 'csv-mutualfund-after-additional-import.png'),
    fullPage: true,
  });
});
