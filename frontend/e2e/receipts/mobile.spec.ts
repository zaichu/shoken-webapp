import type { Page } from '@playwright/test';
import { expect, test, paginated, mockSession } from '../support/test';
import {
  csvRow,
  mockCsvKindStats,
  openCsvDelete,
  savedCount,
} from '../support/csv-page';
import * as path from 'path';

test.use({ viewport: { width: 390, height: 844 } });

const DIVIDENDS = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    settlement_date: '2024-03-01',
    product: '特定口座',
    account: 'SBI証券',
    security_code: '9904',
    security_name: '架空自動車',
    unit_price: 30.0,
    shares: 100,
    dividends_before_tax: 3000,
    taxes: 609,
    net_amount_received: 2391,
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  },
  {
    id: '00000000-0000-0000-0000-000000000002',
    settlement_date: '2024-03-15',
    product: '特定口座',
    account: '楽天証券',
    security_code: '9910',
    security_name: '架空電信電話',
    unit_price: 10.0,
    shares: 200,
    dividends_before_tax: 2000,
    taxes: 406,
    net_amount_received: 1594,
    created_at: '2024-03-15T00:00:00Z',
    updated_at: '2024-03-15T00:00:00Z',
  },
  {
    id: '00000000-0000-0000-0000-000000000003',
    settlement_date: '2024-02-10',
    product: 'NISA口座',
    account: 'SBI証券',
    security_code: '9914',
    security_name: '架空エレクトロニクス',
    unit_price: 20.0,
    shares: 50,
    dividends_before_tax: 1000,
    taxes: 203,
    net_amount_received: 797,
    created_at: '2024-02-10T00:00:00Z',
    updated_at: '2024-02-10T00:00:00Z',
  },
];

const DOMESTIC = {
  id: '00000000-0000-0000-0000-000000000004',
  trade_date: '2024-02-01',
  settlement_date: '2024-02-03',
  account: '特定口座',
  security_code: '9905',
  security_name: '架空ゲーム',
  shares: 10,
  asked_price: 5000,
  proceeds: 55000,
  purchase_price: 5000,
  realized_profit_and_loss: 5000,
  taxes: 1015,
  realized_profit_and_loss_after_tax: 3985,
  created_at: '2024-02-01T00:00:00Z',
  updated_at: '2024-02-01T00:00:00Z',
};

const FUND = {
  id: '00000000-0000-0000-0000-000000000005',
  trade_date: '2024-02-01',
  settlement_date: '2024-02-03',
  fund_name: '架空インデックス 米国株式',
  account: 'NISA',
  shares: 1000,
  exchange_rate: 1,
  cancellation_unit_price_yen: 20000,
  cancellation_amount_yen: 20000,
  average_acquisition_price_yen: 18000,
  realized_profit_and_loss: 2000,
  taxes: 406,
  realized_profit_and_loss_after_tax: 1594,
  created_at: '2024-02-01T00:00:00Z',
  updated_at: '2024-02-01T00:00:00Z',
};

async function mockApi(page: Page) {
  await page.route(/\/api\/v1\//, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([])),
    }),
  );
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated(DIVIDENDS)),
    }),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([DOMESTIC])),
    }),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([FUND])),
    }),
  );
}

test('スマホ幅では明細が2行の行表示になりテーブルは隠れる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await expect(cardList).toBeVisible();
  await expect(page.getByRole('table')).toBeHidden();
  await expect(cardList.getByTestId('receipt-card')).toHaveCount(3);
  await expect(cardList.getByTestId('receipt-card-group')).toHaveCount(2);

  // 1行目は銘柄名と受取額(税引後)、2行目は日付・コード・口座だけ出す(補助の金額は出さない)
  // 先頭は入金日順で架空電信電話
  const card = cardList.getByTestId('receipt-card').first();
  await expect(card.locator('.list-row-name')).toHaveText('架空電信電話');
  await expect(card.locator('.list-row-primary')).toHaveText('+¥1,594');
  const meta = card.locator('.list-row-meta');
  await expect(meta).toContainText('3/15');
  await expect(meta).toContainText('9910');
  await expect(meta).toContainText('楽天証券');
  await expect(card.locator('.list-row-secondary')).toHaveCount(0);
  // 行を押すと詳細ダイアログが開く。閉じると押した行に戻る
  const rowButton = card.getByTestId('receipt-row-button');
  await expect(rowButton).toHaveAttribute('aria-haspopup', 'dialog');
  await rowButton.click();
  const dialog = page.getByTestId('receipt-detail-dialog');
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText('9910');
  await expect(dialog).toContainText('+¥1,594');
  await expect(dialog).toContainText('税引前');
  await expect(dialog).toContainText('¥2,000');
  const items = dialog.locator('dt');
  await expect(items).toHaveText(['入金日', '数量', '1株配当', '税引前', '税額', '税引後']);
  await dialog.getByTestId('detail-dialog-close').click();
  await expect(dialog).toBeHidden();
  await expect(rowButton).toBeFocused();
});

test('グループ見出しは件数と小計を常に表示する', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await expect(cardList.getByRole('button', { name: /2024年3月 · 2件/ })).toHaveCount(0);
  await expect(cardList.getByTestId('receipt-card-group').first()).toContainText(
    '2024年3月 · 2件',
  );
  await expect(cardList.getByTestId('receipt-card-group').first()).toContainText(
    '受取 +¥3,985',
  );
});

test('タブは1行のまま並ぶ', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');
  await expect(page.getByRole('tab')).toHaveCount(3);

  const tops = await page
    .getByRole('tab')
    .evaluateAll((tabs) => tabs.map((tab) => tab.getBoundingClientRect().top));
  expect(new Set(tops).size).toBe(1);
});

test('矢印キーとHome/Endでタブを移動しフォーカスも追従する', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const dividend = page.getByRole('tab', { name: /配当金/ });
  const domestic = page.getByRole('tab', { name: /国内株式/ });
  const fund = page.getByRole('tab', { name: /投資信託/ });
  await expect(dividend).toHaveAttribute('aria-selected', 'true');

  await dividend.press('ArrowRight');
  await expect(domestic).toHaveAttribute('aria-selected', 'true');
  await expect(domestic).toBeFocused();
  await expect(domestic).toHaveAttribute('tabindex', '0');
  await expect(dividend).toHaveAttribute('tabindex', '-1');

  await domestic.press('ArrowRight');
  await expect(fund).toHaveAttribute('aria-selected', 'true');
  await fund.press('ArrowRight');
  await expect(dividend).toHaveAttribute('aria-selected', 'true');
  await expect(dividend).toBeFocused();

  await dividend.press('End');
  await expect(fund).toHaveAttribute('aria-selected', 'true');
  await fund.press('Home');
  await expect(dividend).toHaveAttribute('aria-selected', 'true');
});

test('国内株式タブでも2行の行にコード・口座・金額が出る', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  await page.getByRole('tab', { name: '国内株式' }).click();
  const card = page
    .getByTestId('receipt-card-list')
    .getByTestId('receipt-card')
    .first();
  await expect(card.locator('.list-row-meta')).toContainText('9905');
  await expect(card.locator('.list-row-meta')).toContainText('特定');
  await expect(card.locator('.list-row-primary')).toContainText('+¥3,985');
  await expect(card.locator('.list-row-secondary')).toHaveCount(0);
  // 補助の売却額は行に出さず詳細ダイアログに出す
  await card.getByTestId('receipt-row-button').click();
  const dialog = page.getByTestId('receipt-detail-dialog');
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText('売却額');
  await expect(dialog).toContainText('¥55,000');
  await dialog.getByTestId('detail-dialog-close').click();
  // 1件だけのとき帯・見出し・行の税引後が一致する
  await expect(page.getByTestId('receipt-summary-strip')).toContainText('+¥3,985');
});

test('投資信託タブでもファンド名が出る', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  await page.getByRole('tab', { name: '投資信託' }).click();
  const card = page
    .getByTestId('receipt-card-list')
    .getByTestId('receipt-card')
    .first();
  await expect(card.locator('.list-row-name')).toContainText('架空インデックス');
  await expect(card.locator('.list-row-secondary')).toHaveCount(0);
  // 補助の解約額は行に出さず詳細ダイアログに出す
  await card.getByTestId('receipt-row-button').click();
  const dialog = page.getByTestId('receipt-detail-dialog');
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText('解約額');
  await dialog.getByTestId('detail-dialog-close').click();
});

test('検索欄で銘柄コードで絞り込んでも残る行は同じ書式', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await page.getByTestId('receipts-search-sheet-entry').click();
  const sheet = page.getByTestId('receipts-search-sheet');
  await sheet.getByPlaceholder('銘柄名・コード').fill('9910');
  await sheet.getByTestId('search-dialog-search').click();
  await expect(sheet).toBeHidden();
  await expect(cardList.getByTestId('receipt-card')).toHaveCount(1);
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');

  const card = cardList.getByTestId('receipt-card').first();
  await expect(card.locator('.list-row-meta')).toContainText('9910');
  await expect(card.locator('.list-row-meta')).toContainText('楽天証券');
});

test('下部ナビは検索欄より下にありリンクで遷移できる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const nav = page.locator('.app-bottom-nav');
  await expect(nav).toBeVisible();
  // 検索の入口は下部ナビの上にある
  const search = page.getByTestId('receipts-search-sheet-entry');
  const navBox = (await nav.boundingBox())!;
  const searchBox = (await search.boundingBox())!;
  expect(navBox.y).toBeGreaterThanOrEqual(searchBox.y);

  await nav.getByRole('link', { name: '銘柄検索' }).click();
  await expect(page).toHaveURL('/search');
});



test('1024px境界で2行の行表示と表が切り替わる', async ({ page }) => {
  await mockApi(page);
  await page.setViewportSize({ width: 1023, height: 844 });
  await page.goto('/receipts');
  await expect(page.getByTestId('receipt-card-list')).toBeVisible();
  await expect(page.getByRole('table')).toBeHidden();
  await expect(page.locator('.app-bottom-nav')).toBeVisible();

  await page.setViewportSize({ width: 1024, height: 844 });
  await expect(page.getByRole('table')).toBeVisible();
  await expect(page.getByTestId('receipt-card-list')).toBeHidden();
  await expect(page.locator('.app-bottom-nav')).toBeHidden();
});

test('全件削除はカード内の「すべて削除…」ボタンから確認モーダルで行う', async ({ page }) => {
  let rows: unknown[] = [...DIVIDENDS];
  let deleteCount = 0;
  await page.route(/\/api\/v1\//, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated([])),
    }),
  );
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => {
    if (route.request().method() === 'DELETE') {
      deleteCount += 1;
      rows = [];
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: '{}',
      });
    }
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(paginated(rows)),
    });
  });
  await mockCsvKindStats(page, {
    counts: () => ({ dividend: rows.length }),
  });
  await page.goto('/csv');

  // 配当金の行の⋯メニューから削除を開く
  await openCsvDelete(page, 'dividend');

  // 確認モーダルに種類と件数が出る
  const modal = page.getByRole('dialog');
  await expect(modal).toBeVisible();
  await expect(modal).toContainText('取引明細(配当金)を削除しますか?');
  await expect(modal).toContainText('元に戻せません');
  await expect(modal).toContainText('3件');

  // 初期フォーカスは「キャンセル」にある
  const cancelButton = modal.getByRole('button', { name: 'キャンセル' });
  await expect(cancelButton).toBeFocused();

  // キャンセルでモーダルが閉じ、DELETE は送られない
  await cancelButton.click();
  await expect(modal).toBeHidden();
  expect(deleteCount).toBe(0);

  // 再度開いて確定する
  await openCsvDelete(page, 'dividend');
  await modal.getByRole('button', { name: '3件を削除する' }).click();
  expect(deleteCount).toBe(1);
  await expect(modal).toBeHidden();
  // 削除後は行の件数が「0件」になる
  await expect(savedCount(page, 'dividend')).toContainText('0件');
});


