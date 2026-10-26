import { Page } from '@playwright/test';
import { expect, test, json, paginated } from '../support/test';
import { csvRow, mockCsvKindStats } from '../support/csv-page';

/**
 * 「CSV 最新取得」ボタンの E2E。拡張機能は window.chrome をモックして、
 * メッセージの行き来と自動取り込みだけを確かめる(楽天証券・拡張機能の実物は使わない)。
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

// 拡張機能の chrome.runtime.connect を仕込む。state に emit で好きな応答を流せる。
// 実物と同じく connect ごとに別 port を返す(再実行で破棄済みのリスナーを呼ばない)
async function mockExtension(page: Page) {
  await page.addInitScript(() => {
    const state = {
      ids: [] as string[],
      requests: [] as { type?: string; kinds?: string[] }[],
      emit: (_message: unknown) => {},
    };
    const ports: { listeners: ((message: unknown) => void)[] }[] = [];
    state.emit = (message) =>
      ports.forEach((port) => port.listeners.forEach((fn) => fn(message)));
    (window as unknown as { __csvSyncMock: typeof state }).__csvSyncMock = state;
    (window as unknown as { chrome: unknown }).chrome = {
      runtime: {
        connect: (id: string) => {
          state.ids.push(id);
          const port = { listeners: [] as ((message: unknown) => void)[] };
          ports.push(port);
          return {
            onMessage: {
              addListener: (fn: (message: unknown) => void) => {
                port.listeners.push(fn);
              },
            },
            onDisconnect: { addListener: (_fn: (p: unknown) => void) => {} },
            postMessage: (message: { type?: string; kinds?: string[] }) => {
              state.requests.push(message);
            },
            disconnect: () => {
              port.listeners = [];
            },
          };
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

interface SyncKind {
  kind: string;
  csv: string;
  listPath: RegExp;
  previewPath: RegExp;
  importPath: RegExp;
  // 取り込み後に一覧 API が返す行(表の更新を確かめる目印の名前を含む)
  row: Record<string, unknown>;
}

// 受け取る CSV は安全装置が通す形式(必須列あり・データ行あり)。中身に SECRET_MARK を埋めて
// 画面に出ないことも確かめる(一覧 API が返すのは別の目印行で、CSV 本文ではない)
const SYNC_KINDS: SyncKind[] = [
  {
    kind: 'assetbalance',
    csv: `銘柄コード,銘柄名,保有数量［株］\n9901,${SECRET_MARK},100\n`,
    listPath: /\/api\/v1\/asset-balances(?:\?.*)?$/,
    previewPath: /\/api\/v1\/asset-balance-import-validations$/,
    importPath: /\/api\/v1\/asset-balance-imports$/,
    row: {
      security_code: '9901',
      security_name: '架空同期銘柄',
      account: '特定',
      shares: 100,
      average_purchase_price: 1000,
      total_purchase_amount: 100000,
      current_price: 1100,
      daily_change: 10,
      price_as_of: '2026-10-23',
      prev_close: 1090,
    },
  },
  {
    kind: 'fundbalance',
    csv: `投資信託種別,口座区分,ファンド,保有数量［口］\n投資信託,特定,${SECRET_MARK},100\n`,
    listPath: /\/api\/v1\/fund-balances(?:\?.*)?$/,
    previewPath: /\/api\/v1\/fund-balance-import-validations$/,
    importPath: /\/api\/v1\/fund-balance-imports$/,
    row: {
      fund_name: '架空同期ファンド',
      account: '特定',
      dividends_course: '再投資型',
      shares: 100,
      average_purchase_price: 1000,
      total_purchase_amount: 100000,
      base_price: 1100,
      base_price_day_change: 10,
      base_price_month_change: null,
      market_value: 110000,
      profit_loss: 10000,
      profit_loss_rate: 10,
      total_return: 10000,
      price_as_of: '2026-10-23',
      price_source: 'csv',
    },
  },
  {
    kind: 'dividend',
    csv: `入金日,商品,口座,銘柄\n2026/10/01,国内株式,特定,${SECRET_MARK}\n`,
    listPath: /\/api\/v1\/dividends(?:\?.*)?$/,
    previewPath: /\/api\/v1\/dividend-import-validations$/,
    importPath: /\/api\/v1\/dividend-imports$/,
    row: {
      settlement_date: '2026-10-01',
      product: '国内株式',
      account: '特定・一般',
      security_code: '9903',
      security_name: '架空同期配当',
      unit_price: 10,
      shares: 100,
      dividends_before_tax: 1000,
      taxes: 203,
      net_amount_received: 797,
    },
  },
  {
    kind: 'domesticstock',
    csv: `約定日,受渡日,銘柄コード,口座\n2026/10/01,2026/10/03,${SECRET_MARK},特定\n`,
    listPath: /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
    previewPath: /\/api\/v1\/domestic-stock-import-validations$/,
    importPath: /\/api\/v1\/domestic-stock-imports$/,
    row: {
      trade_date: '2026-10-01',
      settlement_date: '2026-10-03',
      security_code: '9904',
      security_name: '架空同期売買',
      account: '特定',
      shares: 100,
      asked_price: 1000,
      proceeds: 100000,
      purchase_price: 950,
      realized_profit_and_loss: 5000,
      taxes: 1015,
      realized_profit_and_loss_after_tax: 3985,
    },
  },
  {
    kind: 'mutualfund',
    csv: `約定日,受渡日,ファンド名,口座\n2026/10/01,2026/10/03,${SECRET_MARK},特定\n`,
    listPath: /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
    previewPath: /\/api\/v1\/mutual-fund-import-validations$/,
    importPath: /\/api\/v1\/mutual-fund-imports$/,
    row: {
      trade_date: '2026-10-01',
      settlement_date: '2026-10-03',
      fund_name: '架空同期投信',
      dividends: '再投資型',
      account: '特定',
      shares: 100,
      exchange_rate: 0,
      cancellation_unit_price_yen: 1000,
      cancellation_amount_yen: 100000,
      average_acquisition_price_yen: 900,
      realized_profit_and_loss: 10000,
      taxes: 2031,
      realized_profit_and_loss_after_tax: 7969,
    },
  },
];

function emitAllCsvs(page: Page) {
  return emitMessage(page, {
    type: 'result',
    csvs: SYNC_KINDS.map(({ kind, csv }) => ({
      kind,
      content: csv,
      fetchedAt: '2026-10-23T00:00:00Z',
      fileRemoved: true,
    })),
    failures: [],
  });
}

// 取り込み経路の API モック。import が来た種類の行を db に積み、一覧 API は db を返す。
// assetSavedTotal を渡すと保有残高の保存済み件数をあらかじめ持つ状態にする(激減確認用)。
// heldPreview を渡すとその種類の preview 応答を gate が解くまで保留する(応答待ち中の
// リセットを再現するため)
async function mockSyncApis(
  page: Page,
  options: {
    assetSavedTotal?: number;
    heldPreview?: { kind: string; gate: Promise<void> };
    // その種類の最初の import POST だけを gate が解くまで保留する(分割送信の途中を再現するため)
    heldImport?: { kind: string; gate: Promise<void> };
  } = {},
) {
  let seq = 0;
  const asDbRow = (row: Record<string, unknown>) => {
    seq += 1;
    return {
      id: `00000000-0000-0000-0000-${String(seq).padStart(12, '0')}`,
      ...row,
      created_at: '2026-10-01T00:00:00Z',
      updated_at: '2026-10-01T00:00:00Z',
    };
  };
  const db = new Map<string, unknown[]>(SYNC_KINDS.map(({ kind }) => [kind, []]));
  if (options.assetSavedTotal) {
    db.set(
      'assetbalance',
      Array.from({ length: options.assetSavedTotal }, (_, i) =>
        asDbRow({
          ...SYNC_KINDS[0].row,
          security_code: `98${String(i).padStart(2, '0')}`,
        }),
      ),
    );
  }
  const calls = { preview: [] as string[], import: [] as string[], list: [] as string[] };

  for (const fixture of SYNC_KINDS) {
    await page.route(fixture.listPath, (route) => {
      calls.list.push(fixture.kind);
      const data = db.get(fixture.kind) ?? [];
      return route.fulfill(json(paginated(data)));
    });
    await page.route(fixture.previewPath, async (route) => {
      calls.preview.push(fixture.kind);
      if (options.heldPreview?.kind === fixture.kind) {
        await options.heldPreview.gate;
      }
      return route.fulfill(json({ total_rows: 1, valid_rows: 1, errors: [], rows: [] }));
    });
    let importHeld = false;
    await page.route(fixture.importPath, async (route) => {
      calls.import.push(fixture.kind);
      if (options.heldImport?.kind === fixture.kind && !importHeld) {
        importHeld = true;
        await options.heldImport.gate;
      }
      const rows = db.get(fixture.kind);
      if (rows) rows.push(asDbRow(fixture.row));
      return route.fulfill(json({ inserted: 1, skipped: 0, errors: [] }));
    });
  }
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
  return calls;
}

async function emitFailureResult(page: Page) {
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
}

test.describe('CSV 最新取得', () => {
  test('CSV管理: 5種類を受け取って自動で取り込み、結果と表の更新が出る', async ({ page }) => {
    await mockSession(page);
    const calls = await mockSyncApis(page);
    await mockExtension(page);

    // 一覧のストアを先に作っておく(取り込み後の再取得が未訪問ページを作らないことの確認用)
    await page.goto('/receipts');
    await expect(page.getByTestId('receipts-workspace')).toBeVisible();
    await page.goto('/assetbalance');
    await expect(page.getByTestId('assetbalance-workspace')).toBeVisible();

    await page.goto('/csv?kind=dividend');
    const button = page.getByTestId('csv-fetch-button');
    await button.click();
    const request = await page.evaluate(
      () => (window as unknown as { __csvSyncMock: { requests: unknown[] } }).__csvSyncMock.requests[0],
    );
    expect(request).toMatchObject({
      type: 'fetchLatestCsv',
      kinds: ['assetbalance', 'fundbalance', 'dividend', 'domesticstock', 'mutualfund'],
    });
    await expect(button).toBeDisabled();

    await emitAllCsvs(page);

    // 5種類とも既存の preview → import を通って取り込み結果が出る
    for (const { kind } of SYNC_KINDS) {
      await expect(page.getByTestId(`csv-sync-kind-${kind}`)).toContainText('取り込んだ(1件');
    }
    for (const { kind } of SYNC_KINDS) {
      expect(calls.preview.filter((k) => k === kind)).toHaveLength(1);
      expect(calls.import.filter((k) => k === kind)).toHaveLength(1);
    }
    // CSV の中身(個人情報)は表示しない
    await expect(page.getByText(SECRET_MARK)).toHaveCount(0);
    await expect(button).toBeEnabled();

    // 取り込んだ種類の一覧が再取得され、戻ったページの表に新しい行が出る
    await page.goto('/receipts');
    await expect(page.getByText('架空同期配当').first()).toBeVisible();
    await page.goto('/assetbalance');
    await expect(page.getByText('架空同期銘柄').first()).toBeVisible();
    await page.getByRole('tab', { name: '投資信託' }).click();
    await expect(page.getByText('架空同期ファンド').first()).toBeVisible();
  });

  test('件数が半分未満に減る種類は確認を出し、「いいえ」でその種類だけ取り込まない', async ({
    page,
  }) => {
    await mockSession(page);
    const calls = await mockSyncApis(page, { assetSavedTotal: 10 });
    await mockExtension(page);
    await page.goto('/csv?kind=dividend');
    await page.getByTestId('csv-fetch-button').click();
    await emitAllCsvs(page);

    // 保有残高だけ 10件→1件 で確認が出て、後続の種類は回答を待って止まる
    const confirm = page.getByTestId('csv-sync-confirm');
    await expect(confirm).toBeVisible();
    await expect(confirm).toContainText('件数が大きく減っています');
    expect(calls.import).toEqual([]);

    // 確認待ちの間も実行中のままで、連打で二重に流せない
    await expect(page.getByTestId('csv-fetch-button')).toBeDisabled();

    await page.getByTestId('csv-sync-confirm-no').click();
    await expect(confirm).toHaveCount(0);
    await expect(page.getByTestId('csv-sync-kind-assetbalance')).toContainText(
      '取り込まなかった',
    );
    // 残りの種類は続けて取り込む
    for (const { kind } of SYNC_KINDS.slice(1)) {
      await expect(page.getByTestId(`csv-sync-kind-${kind}`)).toContainText('取り込んだ(1件');
    }
    expect(calls.import).not.toContain('assetbalance');
  });

  test('激減の確認で「はい」を選ぶと件数判定を飛ばして取り込む', async ({ page }) => {
    await mockSession(page);
    const calls = await mockSyncApis(page, { assetSavedTotal: 10 });
    await mockExtension(page);
    await page.goto('/csv?kind=dividend');
    await page.getByTestId('csv-fetch-button').click();
    await emitAllCsvs(page);

    await page.getByTestId('csv-sync-confirm').waitFor();
    await page.getByTestId('csv-sync-confirm-yes').click();
    for (const { kind } of SYNC_KINDS) {
      await expect(page.getByTestId(`csv-sync-kind-${kind}`)).toContainText('取り込んだ(1件');
    }
    expect(calls.import).toContain('assetbalance');
  });

  test('取り込み応答待ちにログアウトすると残りの取り込み POST は送らない', async ({ page }) => {
    // ログアウト(DELETE)で以後は未認証に切り替わるセッション
    let loggedIn = true;
    await page.route(/\/api\/v1\/session$/, (route) => {
      if (route.request().method() === 'DELETE') {
        loggedIn = false;
        return route.fulfill(json({ message: 'ok' }));
      }
      return route.fulfill(
        loggedIn
          ? json(USER)
          : { status: 401, contentType: 'application/json', body: '{"message":"unauthorized"}' },
      );
    });
    let releasePreview: () => void = () => {};
    const gate = new Promise<void>((resolve) => {
      releasePreview = resolve;
    });
    const calls = await mockSyncApis(page, { heldPreview: { kind: 'assetbalance', gate } });
    await mockExtension(page);
    await page.goto('/csv?kind=dividend');
    await page.getByTestId('csv-fetch-button').click();
    await emitAllCsvs(page);

    // 先頭の種類が preview 応答待ちの間にログアウトする
    await expect.poll(() => calls.preview.includes('assetbalance')).toBe(true);
    await page.getByRole('button', { name: /テストユーザーのアカウントメニュー/ }).click();
    await page.getByRole('menuitem', { name: 'ログアウト' }).click();
    // リセットされると進行表示の内容が消える(パネル自体は残るが中身は空になる)
    await expect(page.getByTestId('csv-sync-kinds')).toHaveCount(0);

    releasePreview();
    // preview の応答が戻っても旧セッションの取り込みは送らない
    await page.waitForTimeout(500);
    expect(calls.import).toEqual([]);
  });

  test('分割送信の途中でログアウトすると残りのチャンクは送らない', async ({ page }) => {
    let loggedIn = true;
    await page.route(/\/api\/v1\/session$/, (route) => {
      if (route.request().method() === 'DELETE') {
        loggedIn = false;
        return route.fulfill(json({ message: 'ok' }));
      }
      return route.fulfill(
        loggedIn
          ? json(USER)
          : { status: 401, contentType: 'application/json', body: '{"message":"unauthorized"}' },
      );
    });
    let releaseImport: () => void = () => {};
    const gate = new Promise<void>((resolve) => {
      releaseImport = resolve;
    });
    const calls = await mockSyncApis(page, { heldImport: { kind: 'dividend', gate } });
    await mockExtension(page);
    await page.goto('/csv?kind=dividend');
    await page.getByTestId('csv-fetch-button').click();
    // 追加系の取り込みは500行ごとの分割送信(2チャンク目が残る形にする)
    const dividendCsv =
      '入金日,商品,口座,銘柄\n' +
      Array.from({ length: 501 }, (_, i) => `2026/10/01,国内株式,特定,${SECRET_MARK}${i}`).join(
        '\n',
      ) +
      '\n';
    await emitMessage(page, {
      type: 'result',
      csvs: [
        { kind: 'dividend', content: dividendCsv, fetchedAt: '2026-10-23T00:00:00Z' },
      ],
      failures: [],
    });

    // 1チャンク目の import が応答待ちの間にログアウトする
    await expect.poll(() => calls.import.includes('dividend')).toBe(true);
    await page.getByRole('button', { name: /テストユーザーのアカウントメニュー/ }).click();
    await page.getByRole('menuitem', { name: 'ログアウト' }).click();
    // リセットされると進行表示の内容が消える(パネル自体は残るが中身は空になる)
    await expect(page.getByTestId('csv-sync-kinds')).toHaveCount(0);

    releaseImport();
    // 1チャンク目の応答が戻っても2チャンク目以降は送らない
    await page.waitForTimeout(500);
    expect(calls.import).toEqual(['dividend']);
  });

  test('安全装置で取り込めない CSV は理由を出して取り込まない', async ({ page }) => {
    await mockSession(page);
    const calls = await mockSyncApis(page);
    await mockExtension(page);
    await page.goto('/csv?kind=dividend');
    await page.getByTestId('csv-fetch-button').click();
    await emitMessage(page, {
      type: 'result',
      csvs: [
        {
          kind: 'assetbalance',
          content: '<html><body>ログイン</body></html>',
          fetchedAt: '2026-10-23T00:00:00Z',
        },
        { kind: 'dividend', content: '入金日,商品,口座,銘柄\n', fetchedAt: '2026-10-23T00:00:00Z' },
      ],
      failures: [],
    });
    await expect(page.getByTestId('csv-sync-kind-assetbalance')).toContainText(
      '取り込まなかった',
    );
    await expect(page.getByTestId('csv-sync-kind-dividend')).toContainText('取り込まなかった');
    // 安全装置で止めた分は preview/import を呼ばない
    expect(calls.import).toEqual([]);
    expect(calls.preview).toEqual([]);
  });

  test('取得に失敗した種類は表の行にも「取得できず」と詳細を出す', async ({ page }) => {
    await mockSession(page);
    await mockSyncApis(page);
    await mockCsvKindStats(page, { counts: () => ({}) });
    await mockExtension(page);
    await page.goto('/csv?kind=dividend');
    await page.getByTestId('csv-fetch-button').click();
    await emitFailureResult(page);

    await expect(csvRow(page, 'dividend')).toContainText('取得できず');
    const detailButton = csvRow(page, 'dividend').getByTestId('status-detail');
    const detail = csvRow(page, 'dividend').getByTestId('status-detail-content');
    await expect(detailButton).toBeVisible();
    await expect(detail).toHaveCount(0);

    await detailButton.click();
    await expect(detail).toBeVisible();

    await detailButton.click();
    await expect(detail).toHaveCount(0);
  });

  test('行の⋯メニューの再取得はその種類だけを拡張機能へ渡す', async ({ page }) => {
    await mockSession(page);
    const calls = await mockSyncApis(page);
    await mockCsvKindStats(page, { counts: () => ({}) });
    await mockExtension(page);
    await page.goto('/csv?kind=dividend');

    await csvRow(page, 'dividend').getByRole('button', { name: 'メニュー' }).click();
    await page.getByRole('button', { name: '楽天証券から再取得' }).click();

    const request = await page.evaluate(
      () => (window as unknown as { __csvSyncMock: { requests: unknown[] } }).__csvSyncMock.requests[0],
    );
    expect(request).toMatchObject({ type: 'fetchLatestCsv', kinds: ['dividend'] });
    // 1種類だけの依頼なので、他の種類の取り込みは始まらない
    expect(calls.preview).toEqual([]);
  });

  test('拡張機能が無いとき導入案内とストアへのリンクが出る', async ({ page }) => {
    await mockSession(page);
    await mockSyncApis(page);
    // connect 自体が無い環境(Chrome 以外・拡張機能未導入)
    await page.addInitScript(() => {
      (window as unknown as { chrome: unknown }).chrome = {};
    });
    await page.goto('/csv?kind=dividend');
    await page.getByTestId('csv-fetch-button').click();
    await expect(page.getByText('CSV 取得用の拡張機能が見つかりません')).toBeVisible();
    await expect(
      page.getByRole('link', { name: /Chrome ウェブストアで拡張機能を導入する/ }),
    ).toBeVisible();
  });

  test('file-access-disabled の失敗は設定手順の案内が出る', async ({ page }) => {
    await mockSession(page);
    await mockSyncApis(page);
    await mockExtension(page);
    await page.goto('/csv?kind=dividend');
    await page.getByTestId('csv-fetch-button').click();
    await emitFailureResult(page);
    await expect(page.getByTestId('csv-sync-kind-dividend')).toContainText('失敗');
    await expect(
      page.getByText('ファイルの URL へのアクセスが許可されていません'),
    ).toBeVisible();
    await expect(page.getByText('「ファイルの URL へのアクセスを許可する」を ON にする')).toBeVisible();
  });

  test('資産管理・取引明細の見出し横にはボタンがなくなり、CSV管理画面で操作する', async ({ page }) => {
    await mockSession(page);
    await mockSyncApis(page);
    await mockExtension(page);

    // 一覧画面にはボタンが出ない
    await page.goto('/receipts');
    await expect(page.getByTestId('receipts-workspace')).toBeVisible();
    await expect(page.getByTestId('csv-fetch-button')).toHaveCount(0);

    await page.goto('/assetbalance');
    await expect(page.getByTestId('csv-fetch-button')).toHaveCount(0);

    // CSV管理画面にはパネルがあり、ボタンと進行表示がある
    await page.goto('/csv?kind=dividend');
    await expect(page.getByTestId('csv-fetch-button')).toBeVisible();
    await page.getByTestId('csv-fetch-button').click();
    await emitMessage(page, { type: 'progress', state: 'fetching', kind: 'dividend' });
    await expect(page.getByTestId('csv-sync-kind-dividend')).toContainText('取得中');
  });

  test('ログインしていないときはボタンが出ない', async ({ page }) => {
    await mockSession(page, false);
    await page.goto('/csv?kind=dividend');
    await expect(page.getByTestId('csv-fetch-button')).toHaveCount(0);
  });
});
