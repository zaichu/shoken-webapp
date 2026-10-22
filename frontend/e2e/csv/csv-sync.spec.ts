import { Page } from '@playwright/test';
import { expect, test, json, paginated } from '../support/test';

/**
 * 「CSV 最新取得」ボタンの E2E。拡張機能は window.chrome をモックして、
 * メッセージの行き来だけを確かめる(楽天証券・拡張機能の実物は使わない)。
 */

const USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

// CSV の中身が画面に漏れていないことを確かめるための目印
const SECRET_MARK = 'ひみつ銘柄マーカー';

async function mockSession(page: Page, loggedIn = true) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(
      loggedIn
        ? json(USER)
        : { status: 401, contentType: 'application/json', body: '{"message":"unauthorized"}' },
    ),
  );
}

// 各ページが初期表示で呼ぶ一覧系 API を空で返す
async function mockListApis(page: Page) {
  const empty = () => json(paginated([]));
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => route.fulfill(empty()));
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(empty()),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(empty()),
  );
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => route.fulfill(empty()));
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) => route.fulfill(empty()));
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

// 拡張機能の chrome.runtime.connect を仕込む。state に emit で好きな応答を流せる
async function mockExtension(page: Page) {
  await page.addInitScript(() => {
    const state = {
      ids: [] as string[],
      requests: [] as { type?: string; kinds?: string[] }[],
      emit: (_message: unknown) => {},
    };
    const listeners: ((message: unknown) => void)[] = [];
    const port = {
      onMessage: {
        addListener: (fn: (message: unknown) => void) => {
          listeners.push(fn);
        },
      },
      onDisconnect: { addListener: (_fn: (p: unknown) => void) => {} },
      postMessage: (message: { type?: string; kinds?: string[] }) => {
        state.requests.push(message);
      },
      disconnect: () => {},
    };
    state.emit = (message) => listeners.forEach((fn) => fn(message));
    (window as unknown as { __csvSyncMock: typeof state }).__csvSyncMock = state;
    (window as unknown as { chrome: unknown }).chrome = {
      runtime: {
        connect: (id: string) => {
          state.ids.push(id);
          return port;
        },
      },
    };
  });
}

async function emitMessage(page: Page, message: unknown) {
  await page.evaluate(
    (m) => (window as unknown as { __csvSyncMock: { emit: (x: unknown) => void } }).__csvSyncMock.emit(m),
    message,
  );
}

test.describe('CSV 最新取得', () => {
  test('CSV管理: ボタンで拡張機能を呼び出し、進行と受け取り結果が出る', async ({ page }) => {
    await mockSession(page);
    await mockListApis(page);
    await mockExtension(page);
    await page.goto('/receipts/csv?kind=dividend');
    const button = page.getByTestId('csv-sync-button');
    await expect(button).toBeVisible();

    await button.click();
    const request = await page.evaluate(
      () => (window as unknown as { __csvSyncMock: { requests: unknown[] } }).__csvSyncMock.requests[0],
    );
    expect(request).toMatchObject({
      type: 'fetchLatestCsv',
      kinds: ['assetbalance', 'fundbalance', 'dividend', 'domesticstock', 'mutualfund'],
    });
    // 実行中は再クリックできない
    await expect(button).toBeDisabled();

    await emitMessage(page, { type: 'progress', state: 'login-wait' });
    await expect(page.getByText('楽天証券のタブでログインしてください')).toBeVisible();

    await emitMessage(page, { type: 'progress', state: 'fetching', kind: 'assetbalance' });
    await emitMessage(page, {
      type: 'result',
      csvs: [
        'assetbalance',
        'fundbalance',
        'dividend',
        'domesticstock',
        'mutualfund',
      ].map((kind) => ({
        kind,
        content: `コード,名称\n9901,${SECRET_MARK}\n`,
        fetchedAt: '2026-10-23T00:00:00Z',
        fileRemoved: true,
      })),
      failures: [],
    });

    const list = page.getByTestId('csv-sync-kinds');
    await expect(list).toBeVisible();
    for (const kind of [
      'assetbalance',
      'fundbalance',
      'dividend',
      'domesticstock',
      'mutualfund',
    ]) {
      await expect(page.getByTestId(`csv-sync-kind-${kind}`)).toContainText('受け取った(2行)');
    }
    // CSV の中身(個人情報)は表示しない
    await expect(page.getByText(SECRET_MARK)).toHaveCount(0);
    // 結果を受けたら再実行できる
    await expect(button).toBeEnabled();
  });

  test('拡張機能が無いとき導入案内とストアへのリンクが出る', async ({ page }) => {
    await mockSession(page);
    await mockListApis(page);
    // connect 自体が無い環境(Chrome 以外・拡張機能未導入)
    await page.addInitScript(() => {
      (window as unknown as { chrome: unknown }).chrome = {};
    });
    await page.goto('/receipts/csv?kind=dividend');
    await page.getByTestId('csv-sync-button').click();
    await expect(page.getByText('CSV 取得用の拡張機能が見つかりません')).toBeVisible();
    await expect(
      page.getByRole('link', { name: /Chrome ウェブストアで拡張機能を導入する/ }),
    ).toBeVisible();
  });

  test('file-access-disabled の失敗は設定手順の案内が出る', async ({ page }) => {
    await mockSession(page);
    await mockListApis(page);
    await mockExtension(page);
    await page.goto('/receipts/csv?kind=dividend');
    await page.getByTestId('csv-sync-button').click();
    await emitMessage(page, {
      type: 'result',
      csvs: [],
      failures: [
        {
          kind: 'dividend',
          reason: 'ファイルへのアクセスが許可されていません',
          code: 'file-access-disabled',
        },
      ],
    });
    await expect(page.getByTestId('csv-sync-kind-dividend')).toContainText('失敗');
    await expect(
      page.getByText('ファイルの URL へのアクセスが許可されていません'),
    ).toBeVisible();
    await expect(page.getByText('「ファイルの URL へのアクセスを許可する」を ON にする')).toBeVisible();
  });

  test('資産管理・取引明細の見出し横にもボタンがある', async ({ page }) => {
    await mockSession(page);
    await mockListApis(page);
    await mockExtension(page);

    await page.goto('/receipts');
    await expect(page.getByTestId('receipts-workspace')).toBeVisible();
    await expect(page.getByTestId('csv-sync-button')).toBeVisible();

    await page.goto('/assetbalance');
    await expect(page.getByTestId('csv-sync-button')).toBeVisible();
    // 一覧画面では実行したあとも状態が見える
    await page.getByTestId('csv-sync-button').click();
    await emitMessage(page, { type: 'progress', state: 'login-wait' });
    await expect(page.getByTestId('csv-sync-status')).toBeVisible();
    await expect(page.getByText('楽天証券のタブでログインしてください')).toBeVisible();
  });

  test('ログインしていないときはボタンが出ない', async ({ page }) => {
    await mockSession(page, false);
    await page.goto('/receipts/csv?kind=dividend');
    await expect(page.getByTestId('csv-sync-button')).toHaveCount(0);
  });
});
