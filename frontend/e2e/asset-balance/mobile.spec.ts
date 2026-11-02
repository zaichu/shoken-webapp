import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';
import { csvRow } from '../support/csv-page';

const SYN_AUTO = {
  id: 'asset-balance-1',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '9904',
  security_name: '架空自動車',
  shares: 100,
  average_purchase_price: 2500,
  total_purchase_amount: 250000,
  current_price: 2600,
  daily_change: 50,
};

const SYN_ELECTRONICS = {
  id: 'asset-balance-2',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '9914',
  security_name: '架空エレクトロニクス',
  shares: 50,
  average_purchase_price: 3000,
  total_purchase_amount: 150000,
  current_price: 3200,
  daily_change: -20,
};

const SYN_TELECOM = {
  id: 'asset-balance-3',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '9910',
  security_name: '架空電信電話',
  shares: 200,
  average_purchase_price: 150,
  total_purchase_amount: 30000,
  current_price: 160,
  daily_change: 1,
};

async function setupAssetBalanceMocks(page: Page) {
  await mockSession(page);

  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.abort();
    }
    const data = [SYN_AUTO, SYN_ELECTRONICS, SYN_TELECOM];
    return route.fulfill(
      json({
        data,
        total: data.length,
        page: 1,
        per_page: 1000,
      }),
    );
  });

  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

async function gotoAssetBalance(page: Page) {
  await page.goto('/assetbalance');
  await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
}

test.beforeEach(async ({ page }) => {
  await setupAssetBalanceMocks(page);
});

test('390px では検索入力とCSV管理へのリンクにそのまま届く', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);

  // CSV 操作は専用ページに移ったため、一覧にはドロワーがない
  await expect(page.getByTestId('asset-utility-toggle')).toHaveCount(0);
  // 検索・並べ替えは紺帯の下の入口1行にまとめ、操作はシートで行う
  const entry = page.getByTestId('asset-search-sheet-entry');
  await expect(entry).toBeVisible();
  await expect(entry).toContainText('検索・並べ替え');
  const main = page.getByTestId('assetbalance-main-stage');
  // 検索欄は入口を開いたダイアログの中にある。並べ替えは対象と順序の選択
  await entry.click();
  const sheet = page.getByTestId('asset-search-sheet');
  await expect(sheet).toBeVisible();
  const searchInput = page.getByTestId('asset-search-input');
  await expect(searchInput).toBeVisible();
  await expect(searchInput).toBeEditable();
  await expect(sheet.getByTestId('asset-dialog-sort-target')).toHaveValue('market_value');
  await expect(sheet.getByTestId('asset-dialog-sort-order')).toHaveValue('desc');
  // 背景を押すと閉じて入口にフォーカスが戻る
  await page.getByTestId('search-dialog-backdrop').click({ position: { x: 10, y: 10 } });
  await expect(sheet).toBeHidden();
  await expect(entry).toBeFocused();

  // main 内は集計情報 → 保有内訳
  const summary = page.getByTestId('asset-portfolio-summary');
  const firstRow = page.getByTestId('asset-compact-row').first();
  const summaryBox = await summary.boundingBox();
  const rowBox = await firstRow.boundingBox();
  expect(summaryBox).not.toBeNull();
  expect(rowBox).not.toBeNull();
  expect(summaryBox!.y).toBeLessThan(rowBox!.y);

  await expect(summary).toBeVisible();
  await expect(page.getByTestId('portfolio-kpi-grid')).toBeVisible();
  await expect(firstRow).toBeVisible();
  // モバイルはコンパクト行、PC 表は出さない
  await expect(page.getByTestId('asset-holdings-table')).toBeHidden();

  // CSV 管理ページにはヘッダーのリンクから入る
  await page.getByRole('link', { name: 'CSV管理' }).click();
  await expect(page).toHaveURL('/csv');
  await expect(csvRow(page, 'assetbalance')).toBeVisible();
  await page.getByRole('link', { name: '資産管理に戻る' }).click();
  await expect(page).toHaveURL('/assetbalance');
  await expect(main).toBeVisible();
});

test('保有行を押すと中央ダイアログで詳細が出る', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);
  const row = page.getByTestId('asset-compact-row').first();
  // 行には評価額と含み損益(符号付き)が出る
  await expect(row).toContainText('架空自動車');
  await expect(row).toContainText('¥260,000');
  await expect(row).toContainText('+¥10,000');
  const rowButton = row.getByTestId('asset-row-button');
  await expect(rowButton).toHaveAttribute('aria-haspopup', 'dialog');
  await rowButton.click();
  const dialog = page.getByTestId('asset-holding-dialog');
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText('架空自動車');
  await expect(dialog.locator('dt')).toHaveText([
    '現在値',
    '取得単価',
    '取得額利回り',
    '1株配当',
    '数量',
    '取得総額',
    '評価額',
    '年間配当',
    '価格基準日',
  ]);
  await dialog.getByTestId('detail-dialog-close').click();
  await expect(dialog).toBeHidden();
  await expect(rowButton).toBeFocused();
  await expect(page).toHaveURL('/assetbalance');
});

test('一覧に無い文字列を入れるとフィルタ済み空状態と解除ボタンが出る', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);

  await page.getByTestId('asset-search-sheet-entry').click();
  const sheet = page.getByTestId('asset-search-sheet');
  await sheet.getByTestId('asset-search-input').fill('9999');
  await sheet.getByTestId('search-dialog-search').click();
  await expect(sheet).toBeHidden();
  const heading = page.getByRole('heading', { name: '条件に合う保有がありません' });
  await expect(heading).toBeVisible();
  const clearButton = page.getByRole('button', { name: '検索条件を解除' });
  await expect(clearButton).toBeVisible();

  // 解除すると絞り込みも消えて一覧が戻る
  await clearButton.click();
  await expect(heading).toHaveCount(0);
  await expect(page.getByTestId('asset-compact-row').first()).toBeVisible();
});

test('Enter で検索して閉じる。IME 変換中は閉じない', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);

  await page.getByTestId('asset-search-sheet-entry').click();
  const sheet = page.getByTestId('asset-search-sheet');
  const input = sheet.getByTestId('asset-search-input');
  await input.fill('9999');
  await input.press('Enter');
  await expect(sheet).toBeHidden();
  await expect(
    page.getByRole('heading', { name: '条件に合う保有がありません' }),
  ).toBeVisible();
  await page.getByRole('button', { name: '検索条件を解除' }).click();

  // IME 変換中の Enter では閉じない
  await page.getByTestId('asset-search-sheet-entry').click();
  const reopened = page.getByTestId('asset-search-sheet');
  const reopenedInput = reopened.getByTestId('asset-search-input');
  await reopenedInput.fill('架空');
  await page.evaluate(() => {
    const input = document.querySelector('[data-testid="asset-search-input"]') as HTMLInputElement;
    const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true });
    Object.defineProperty(event, 'isComposing', { value: true });
    input.dispatchEvent(event);
  });
  await expect(reopened).toBeVisible();
  await page.keyboard.press('Escape');
});

test('閉じるは変更を捨て、クリアは下書きだけ消す', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);

  // 閉じるは捨てる
  await page.getByTestId('asset-search-sheet-entry').click();
  const sheet = page.getByTestId('asset-search-sheet');
  await sheet.getByTestId('asset-search-input').fill('9999');
  await sheet.getByTestId('search-dialog-close').click();
  await expect(sheet).toBeHidden();
  await expect(page.getByTestId('asset-compact-row').first()).toBeVisible();

  // クリアは下書きだけ消し、検索で反映する
  await page.getByTestId('asset-search-sheet-entry').click();
  const dialog = page.getByTestId('asset-search-sheet');
  await dialog.getByTestId('asset-search-input').fill('9999');
  await dialog.getByTestId('search-dialog-clear').click();
  await expect(dialog.getByTestId('asset-search-input')).toHaveValue('');
  await dialog.getByTestId('search-dialog-search').click();
  await expect(dialog).toBeHidden();
  await expect(page.getByTestId('asset-compact-row').first()).toBeVisible();
});

test('1023px ではモバイル表示、1024px でPC表示に切り替わる', async ({ page }) => {
  await page.setViewportSize({ width: 1023, height: 844 });
  await gotoAssetBalance(page);

  await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
  await expect(page.getByTestId('portfolio-kpi-grid')).toBeVisible();
  await expect(page.getByTestId('asset-compact-row').first()).toBeVisible();
  await expect(page.getByTestId('asset-holdings-table')).toBeHidden();

  await page.setViewportSize({ width: 1024, height: 844 });

  await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible();
  await expect(page.getByTestId('portfolio-kpi-grid')).toBeVisible();
  await expect(page.getByTestId('asset-compact-row').first()).toBeHidden();
  await expect(page.getByTestId('asset-holdings-table')).toBeVisible();
});

test('1920px でも一覧は1カラムでページをはみ出さない', async ({ page }) => {
  await page.setViewportSize({ width: 1920, height: 1080 });
  await gotoAssetBalance(page);

  // CSV 操作は専用ページに移ったためユーティリティレールは無い
  await expect(page.getByTestId('assetbalance-utility-rail')).toHaveCount(0);
  const main = page.getByTestId('assetbalance-main-stage');
  await expect(main).toBeVisible();
  await expect(page.getByTestId('asset-search-input-pc')).toBeVisible();
  const width = await page.evaluate(() => document.documentElement.scrollWidth);
  expect(width).toBeLessThanOrEqual(1920);
});
