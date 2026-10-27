import type { Page } from '@playwright/test';
import { expect, test, paginated, mockSession } from '../support/test';
import {
  csvImportRow,
  csvRow,
  mockCsvKindStats,
  openCsvDelete,
  openCsvImport,
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
  account: 'SBI証券',
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

// スマホ下部の常設検索欄。PC の検索バーは同名の入力を持つので scoped で取る
function mobileSearch(page: Page) {
  return page
    .getByRole('search', { name: '検索', exact: true })
    .getByPlaceholder('銘柄名・コード');
}

test('スマホ幅では明細が2行の行表示になりテーブルは隠れる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await expect(cardList).toBeVisible();
  await expect(page.getByRole('table')).toBeHidden();
  await expect(cardList.getByTestId('receipt-card')).toHaveCount(3);
  await expect(cardList.getByTestId('receipt-card-group')).toHaveCount(2);

  // 1行目は銘柄名と税引後、2行目は日付・コード・口座・数量を並べる
  // 先頭は入金日順で架空電信電話
  const card = cardList.getByTestId('receipt-card').first();
  await expect(card.getByRole('button', { name: /架空電信電話\(9910\) をコピー/ })).toBeVisible();
  await expect(card.locator('.receipt-compact-net')).toHaveText('¥1,594');
  const facts = card.locator('.receipt-compact-facts');
  await expect(facts).toContainText('2024/03/15');
  await expect(facts).toContainText('9910');
  await expect(facts).toContainText('楽天証券');
  await expect(facts).toContainText('200株');
  await expect(card.locator('.receipt-compact-primary')).toContainText('配当 ¥2,000');
  // 行を押すと残りの項目(税額など)が開く
  const toggle = card.locator('.receipt-compact-toggle');
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(card.locator('.receipt-compact-details')).toContainText('税額');
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
    '税引後 ¥3,985',
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

test('国内株式タブでも銘柄リンクとコピーが使える', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  await page.getByRole('tab', { name: '国内株式' }).click();
  const card = page
    .getByTestId('receipt-card-list')
    .getByTestId('receipt-card')
    .first();
  await expect(card.getByRole('link', { name: '9905' })).toHaveAttribute(
    'href',
    '/search?code=9905',
  );
  await expect(
    card.getByRole('button', { name: '架空ゲーム(9905) をコピー' }),
  ).toBeVisible();
  await expect(card.locator('.receipt-compact-facts')).toContainText('10株');
});

test('投資信託タブでもファンド名をコピーできる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  await page.getByRole('tab', { name: '投資信託' }).click();
  const card = page
    .getByTestId('receipt-card-list')
    .getByTestId('receipt-card')
    .first();
  await expect(
    card.getByRole('button', {
      name: '架空インデックス 米国株式 をコピー',
    }),
  ).toBeVisible();
});

test('検索欄で銘柄コードで絞り込んでも残る行は同じ書式', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const cardList = page.getByTestId('receipt-card-list');
  await mobileSearch(page).fill('9910');
  await expect(cardList.getByTestId('receipt-card')).toHaveCount(1);
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');

  const card = cardList.getByTestId('receipt-card').first();
  await expect(card.locator('.receipt-compact-facts')).toContainText('9910');
  await expect(card.locator('.receipt-compact-facts')).toContainText('楽天証券');
});

test('下部ナビは検索欄より下にありリンクで遷移できる', async ({ page }) => {
  await mockApi(page);
  await page.goto('/receipts');

  const nav = page.locator('.app-bottom-nav');
  await expect(nav).toBeVisible();
  const search = mobileSearch(page);
  const navBox = (await nav.boundingBox())!;
  const searchBox = (await search.boundingBox())!;
  expect(navBox.y).toBeGreaterThanOrEqual(searchBox.y);

  await nav.getByRole('link', { name: '銘柄検索' }).click();
  await expect(page).toHaveURL('/search');
});

test('CSV管理ページは一覧のリンクから開き、5種類の行が出る', async ({ page }) => {
  await mockApi(page);
  await mockCsvKindStats(page, { counts: () => ({}) });
  await page.goto('/receipts');
  // CSV の操作は一覧のレールではなく専用ページに移った
  await expect(page.getByTestId('csv-file-input')).toHaveCount(0);

  await page.getByRole('link', { name: 'CSV管理' }).click();
  await expect(page).toHaveURL('/csv');
  // ダイアログは初期表示では開かない
  await expect(csvImportRow(page)).toBeHidden();

  for (const kind of [
    'assetbalance',
    'fundbalance',
    'dividend',
    'domesticstock',
    'mutualfund',
  ] as const) {
    await expect(csvRow(page, kind)).toHaveCount(1);
  }
  // `?kind=` を付けてもダイアログは開かない
  await page.goto('/csv?kind=stock');
  await expect(csvRow(page, 'domesticstock')).toContainText('取引明細(国内株式)');
  await expect(csvImportRow(page)).toBeHidden();
  // ⋯から開く
  await openCsvImport(page, 'domesticstock');
  await expect(csvImportRow(page)).toContainText('取引明細(国内株式) へファイルから取り込み');

  // ダイアログを閉じてからパンくずで戻る
  await page.keyboard.press('Escape');
  await expect(csvImportRow(page)).toBeHidden();
  await page
    .getByRole('navigation', { name: 'パンくず' })
    .getByRole('link', { name: '資産管理' })
    .click();
  await expect(page).toHaveURL('/assetbalance');
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

test('CSV取込の保存後はダイアログが閉じる', async ({ page }) => {
  const imported = { ...DIVIDENDS[0], id: '00000000-0000-0000-0000-000000000009' };
  const rows: unknown[] = [...DIVIDENDS];
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
      body: JSON.stringify(paginated(rows)),
    }),
  );
  await page.route(/\/api\/v1\/dividend-import-validations$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: 1,
        valid_rows: 1,
        errors: [],
        rows: [imported],
      }),
    }),
  );
  await page.route(/\/api\/v1\/dividend-imports$/, (route) => {
    rows.push(imported);
    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ inserted: 1, skipped: 0, errors: [] }),
    });
  });
  await page.goto('/csv?kind=dividend');
  await expect(page.getByTestId('csv-import-dialog')).toBeHidden();
  await openCsvImport(page, 'dividend');

  const fileInput = page.getByTestId('csv-file-input');
  // バックグラウンドの他タブ取得中は input が disabled で、change イベントが捨てられる。
  // setInputFiles は enabled を待たないので、先に有効化を待つ
  await expect(fileInput).toBeEnabled();
  await fileInput.setInputFiles(
      path.resolve(
        test.info().project.testDir,
        'fixtures/csv/dividend-base.csv',
      ),
    );
  await page.getByRole('button', { name: '1件を保存する' }).click();

  // 成功したらダイアログが閉じる
  await expect(page.getByTestId('csv-import-dialog')).toBeHidden();
});
