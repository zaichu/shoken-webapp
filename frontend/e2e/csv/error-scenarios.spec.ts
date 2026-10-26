/**
 * エラーシナリオ E2E テスト（API モック使用）
 *
 * page.route() でバックエンド API をモックし、
 * 失敗系・境界系の UI 挙動を検証する。
 * Leptos の開発サーバーで実行する。
 */
import type { Page } from '@playwright/test';
import { expect, test } from '../support/test';
import * as path from 'path';
const FIXTURE_DIR = path.resolve(process.cwd(), 'e2e/fixtures/csv');

const MOCK_USER = { id: '1', email: 'test@example.com', name: 'テストユーザー' };

// ホスト非依存のパターン（VITE_SHOKEN_WEBAPI_API_URL の値に関わらず一致する）
const ROUTES = {
  authMe: /\/api\/v1\/session$/,
  dividends: /\/api\/v1\/dividends(?:\?.*)?$/,
  dividendsCsvPreview: /\/api\/v1\/dividend-import-validations$/,
  domesticStocks: /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
  mutualfunds: /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
  assetBalances: /\/api\/v1\/asset-balances(?:\?.*)?$/,
};

async function setupCommonMocks(page: Page) {
  await page.route(ROUTES.authMe, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(MOCK_USER) }),
  );
  await page.route(ROUTES.dividends, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(ROUTES.domesticStocks, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(ROUTES.mutualfunds, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
  await page.route(ROUTES.assetBalances, (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '[]' }),
  );
}

// =====================================================================
// 一覧取得失敗
// =====================================================================

test('配当金一覧取得が 401 のときエラー Alert が表示される', async ({ page }) => {
  await setupCommonMocks(page);
  // 後から登録したハンドラが優先されるため、401 で上書き
  await page.route(ROUTES.dividends, (route) =>
    route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: JSON.stringify({ error: '認証が必要です' }),
    }),
  );

  await page.goto('/receipts');

  await expect(
    page.getByRole('alert').filter({ hasText: /エラー/ }),
  ).toBeVisible({ timeout: 10000 });
});

test('資産管理一覧取得が 401 のときエラー Alert が表示される', async ({ page }) => {
  await setupCommonMocks(page);
  await page.route(ROUTES.assetBalances, (route) =>
    route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: JSON.stringify({ error: '認証が必要です' }),
    }),
  );

  await page.goto('/assetbalance');

  await expect(
    page.getByRole('alert').filter({ hasText: /エラー/ }),
  ).toBeVisible({ timeout: 10000 });
});

// =====================================================================
// CSV 行レベルエラー
// =====================================================================

test('検証 API が失敗したときエラーが表示され保存が押せない', async ({ page }) => {
  await setupCommonMocks(page);
  await page.route(ROUTES.dividendsCsvPreview, (route) =>
    route.fulfill({
      status: 500,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'サーバーエラーが発生しました' }),
    }),
  );

  await page.goto('/csv?kind=dividend');
  const fileInput = page.locator('[data-testid="csv-file-input"]');
  await expect(fileInput).toBeEnabled({ timeout: 10000 });
  await fileInput.setInputFiles(path.join(FIXTURE_DIR, 'dividend-base.csv'));

  await expect(page.getByRole('alert').filter({ hasText: /エラー/ })).toBeVisible({
    timeout: 10000,
  });
  // 未検証のファイルは保存へ送らない
  await expect(page.getByRole('button', { name: /保存する/ }).first()).toBeDisabled();
  // 検証に失敗したら選び直せる場所へフォーカスを戻す
  await expect(fileInput).toBeFocused({ timeout: 10000 });
});

test('有効な行が0件のときは保存が押せない', async ({ page }) => {
  await setupCommonMocks(page);
  await page.route(ROUTES.dividendsCsvPreview, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: 2,
        valid_rows: 0,
        errors: [
          { row: 1, message: '受取金額が数値ではありません' },
          { row: 2, message: '受取日の形式が不正です' },
        ],
        rows: [],
      }),
    }),
  );

  await page.goto('/csv?kind=dividend');
  const fileInput = page.locator('[data-testid="csv-file-input"]');
  await expect(fileInput).toBeEnabled({ timeout: 10000 });
  await fileInput.setInputFiles(path.join(FIXTURE_DIR, 'dividend-base.csv'));

  await expect(
    page.getByTestId('csv-action-bar'),
  ).toContainText('取り込める行がありません', { timeout: 10000 });
  await expect(
    page.getByRole('button', { name: /保存する/ }).first(),
  ).toBeDisabled();
});

test('全行が同じエラーのときメッセージは1回だけ出て件数が分かる', async ({ page }) => {
  await setupCommonMocks(page);
  const message = '受取通貨: 円のみ取り込めます（）';
  const errors = Array.from({ length: 8 }, (_, i) => ({
    row: i + 1,
    message,
  }));
  await page.route(ROUTES.dividendsCsvPreview, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: errors.length,
        valid_rows: 0,
        errors,
        rows: [],
      }),
    }),
  );

  await page.goto('/csv?kind=dividend');
  const fileInput = page.locator('[data-testid="csv-file-input"]');
  await expect(fileInput).toBeEnabled({ timeout: 10000 });
  await fileInput.setInputFiles(path.join(FIXTURE_DIR, 'dividend-base.csv'));

  const notice = page.locator('[data-testid="csv-preview-notice"]');
  await expect(notice).toBeVisible({ timeout: 10000 });
  // 同じ文言は行数分繰り返さず1回だけ出す
  await expect(
    notice.getByText('受取通貨: 円のみ取り込めます'),
  ).toHaveCount(1);
  await expect(notice).toContainText('8件エラー');
  await expect(notice).toContainText('8行');
  // 全行落ちのときは見出し不一致を疑う案内を出す
  await expect(
    notice.getByText('見出しの行が想定と違う可能性があります'),
  ).toBeVisible();
});

test('有効行が残る同一エラーでは見出し不一致の案内を出さない', async ({ page }) => {
  await setupCommonMocks(page);
  const message = '数量: 数値ではありません';
  const errors = Array.from({ length: 4 }, (_, i) => ({
    row: i + 1,
    message,
  }));
  await page.route(ROUTES.dividendsCsvPreview, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: errors.length + 1,
        valid_rows: 1,
        errors,
        rows: [
          {
            settlement_date: '2026-03-10',
            product: '国内株式',
            account: '特定・一般',
            security_code: '9918',
            security_name: '架空サンプル',
            unit_price: 100,
            shares: 10,
            dividends_before_tax: 1000,
            taxes: 200,
            net_amount_received: 800,
          },
        ],
      }),
    }),
  );

  await page.goto('/csv?kind=dividend');
  const fileInput = page.locator('[data-testid="csv-file-input"]');
  await expect(fileInput).toBeEnabled({ timeout: 10000 });
  await fileInput.setInputFiles(path.join(FIXTURE_DIR, 'dividend-base.csv'));

  const notice = page.locator('[data-testid="csv-preview-notice"]');
  // 同一メッセージの集約自体は行うが、見出しの誤誘導はしない
  await expect(notice).toContainText('4行が同じエラーです', { timeout: 10000 });
  await expect(
    notice.getByText('見出しの行が想定と違う可能性があります'),
  ).toHaveCount(0);
});

test('CSV プレビューで行エラーが返ったとき warning Alert に一覧表示される', async ({ page }) => {
  await setupCommonMocks(page);
  // プレビューAPI: 行エラーを含む正常レスポンス（HTTP 200）
  await page.route(ROUTES.dividendsCsvPreview, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_rows: 3,
        valid_rows: 1,
        errors: [
          { row: 2, message: '受取金額が数値ではありません' },
          { row: 3, message: '受取日の形式が不正です' },
        ],
        rows: [
          {
            settlement_date: '2026-03-10',
            product: '国内株式',
            account: '特定・一般',
            security_code: '9918',
            security_name: '架空サンプル',
            unit_price: 100,
            shares: 10,
            dividends_before_tax: 1000,
            taxes: 200,
            net_amount_received: 800,
          },
        ],
      }),
    }),
  );

  await page.goto('/csv?kind=dividend');

  // 初回取得中は input が disabled で change を捨てるため、有効化を待ってから投入する
  const fileInput = page.locator('[data-testid="csv-file-input"]');
  await expect(fileInput).toBeEnabled({ timeout: 10000 });
  // ファイル選択でプレビュー API が自動的に呼ばれる
  await fileInput.setInputFiles(
    path.join(FIXTURE_DIR, 'dividend-base.csv'),
  );

  // warning Alert に行エラー一覧が表示されることを確認
  await expect(page.getByText('2件エラー').first()).toBeVisible({ timeout: 10000 });
  await expect(page.getByText('2行目').first()).toBeVisible();
  await expect(page.getByText('3行目').first()).toBeVisible();
});
