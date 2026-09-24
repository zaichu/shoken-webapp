import { expect, test, type Page } from '@playwright/test';
import * as path from 'path';

/**
 * 取引明細 CSV 取込・削除の E2E。
 * API はモックし、fixture CSV は frontend/e2e の既存ファイルをそのまま使う。
 */

function fixtureDir(): string {
  return path.resolve(
    test.info().project.testDir,
    '../../frontend/e2e/__fixtures__/csv',
  );
}

const USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

// backend の一覧行と同じフィールド名。DB 行として登録時に id・タイムスタンプを付ける
const KDDI = {
  settlement_date: '2025-12-08',
  product: '国内株式',
  account: '特定・一般',
  security_code: '9433',
  security_name: 'ＫＤＤＩ',
  unit_price: 40,
  shares: 600,
  dividends_before_tax: 24000,
  taxes: 4875,
  net_amount_received: 19125,
};
const NTT = {
  settlement_date: '2025-12-08',
  product: '国内株式',
  account: '特定・一般',
  security_code: '9432',
  security_name: '日本電信電話',
  unit_price: 5,
  shares: 2000,
  dividends_before_tax: 10000,
  taxes: 2031,
  net_amount_received: 7969,
};
const TOYOTA = {
  settlement_date: '2025-12-09',
  product: '国内株式',
  account: '特定・一般',
  security_code: '7203',
  security_name: 'トヨタ自動車',
  unit_price: 120,
  shares: 100,
  dividends_before_tax: 12000,
  taxes: 2438,
  net_amount_received: 9562,
};
const DIVIDEND_BASE = [
  {
    settlement_date: '2025-12-09',
    product: '国内株式',
    account: '特定・一般',
    security_code: '8591',
    security_name: 'オリックス',
    unit_price: 93.76,
    shares: 200,
    dividends_before_tax: 18752,
    taxes: 3808,
    net_amount_received: 14944,
  },
  {
    settlement_date: '2025-12-10',
    product: '国内株式',
    account: '特定・一般',
    security_code: '8306',
    security_name: '三菱ＵＦＪフィナンシャル・グループ',
    unit_price: 50,
    shares: 300,
    dividends_before_tax: 15000,
    taxes: 3047,
    net_amount_received: 11953,
  },
  {
    settlement_date: '2025-09-26',
    product: '国内株式',
    account: '特定・一般',
    security_code: '8035',
    security_name: '東京エレクトロン',
    unit_price: 1600,
    shares: 10,
    dividends_before_tax: 16000,
    taxes: 3252,
    net_amount_received: 12748,
  },
];
// 5行中 KDDI 2行は重複スキップ → 3件反映
const DIVIDEND_ADDITIONAL = [KDDI, KDDI, KDDI, NTT, TOYOTA];
const DIVIDEND_ADDITIONAL_INSERTED = [KDDI, NTT, TOYOTA];

const ENEOS = {
  trade_date: '2026-02-09',
  settlement_date: '2026-02-12',
  security_code: '5020',
  security_name: 'ＥＮＥＯＳホールディングス',
  account: '特定',
  shares: 100,
  asked_price: 1441,
  proceeds: 144100,
  purchase_price: 1350,
  realized_profit_and_loss: 9100,
  taxes: 1849,
  realized_profit_and_loss_after_tax: 7251,
};
const TEPCO = {
  trade_date: '2026-01-15',
  settlement_date: '2026-01-18',
  security_code: '9501',
  security_name: '東京電力ホールディングス',
  account: '特定',
  shares: 200,
  asked_price: 600,
  proceeds: 120000,
  purchase_price: 680,
  realized_profit_and_loss: -16000,
  taxes: 0,
  realized_profit_and_loss_after_tax: -16000,
};
const MOL = {
  trade_date: '2026-01-20',
  settlement_date: '2026-01-23',
  security_code: '9104',
  security_name: '商船三井',
  account: '特定',
  shares: 50,
  asked_price: 5200,
  proceeds: 260000,
  purchase_price: 4800,
  realized_profit_and_loss: 20000,
  taxes: 4063,
  realized_profit_and_loss_after_tax: 15937,
};
const KYUSHU_DEN = {
  trade_date: '2026-02-12',
  settlement_date: '2026-02-16',
  security_code: '9508',
  security_name: '九州電力',
  account: '特定',
  shares: 100,
  asked_price: 1880,
  proceeds: 188000,
  purchase_price: 1770,
  realized_profit_and_loss: 11000,
  taxes: 2235,
  realized_profit_and_loss_after_tax: 8765,
};
const MITSUBISHI_DEN = {
  trade_date: '2026-02-18',
  settlement_date: '2026-02-21',
  security_code: '6503',
  security_name: '三菱電機',
  account: '特定',
  shares: 150,
  asked_price: 2100,
  proceeds: 315000,
  purchase_price: 2200,
  realized_profit_and_loss: -15000,
  taxes: 0,
  realized_profit_and_loss_after_tax: -15000,
};
const DOMESTIC_BASE = [ENEOS, TEPCO, MOL];
// 11行中、base 登録済みの商船三井 index=1 のみスキップ → 10件反映
const DOMESTIC_ADDITIONAL = [
  MOL, MOL, MOL, MOL, MOL, MOL, MOL, MOL, MOL, KYUSHU_DEN, MITSUBISHI_DEN,
];
const DOMESTIC_ADDITIONAL_INSERTED = [
  MOL, MOL, MOL, MOL, MOL, MOL, MOL, MOL, KYUSHU_DEN, MITSUBISHI_DEN,
];

const SP500_BASE = {
  trade_date: '2022-10-28',
  settlement_date: '2022-11-02',
  fund_name: 'eMAXIS Slim 米国株式(S&P500)',
  dividends: '再投資型',
  account: '特定',
  shares: 3721147,
  exchange_rate: 0,
  cancellation_unit_price_yen: 19661,
  cancellation_amount_yen: 7316147,
  average_acquisition_price_yen: 18005.2,
  realized_profit_and_loss: 615849,
  taxes: 125073,
  realized_profit_and_loss_after_tax: 490776,
};
const RAKUTEN_FUND = {
  trade_date: '2022-12-26',
  settlement_date: '2022-12-29',
  fund_name: '楽天・全米株式インデックス・ファンド',
  dividends: '再投資型',
  account: '特定',
  shares: 500000,
  exchange_rate: 0,
  cancellation_unit_price_yen: 15200,
  cancellation_amount_yen: 7600000,
  average_acquisition_price_yen: 14800,
  realized_profit_and_loss: 200000,
  taxes: 40630,
  realized_profit_and_loss_after_tax: 159370,
};
const SP500_ADDITIONAL = {
  trade_date: '2026-01-08',
  settlement_date: '2026-01-14',
  fund_name: 'eMAXIS Slim 米国株式(S&P500)',
  dividends: '再投資型',
  account: '特定',
  shares: 739553,
  exchange_rate: 0,
  cancellation_unit_price_yen: 39709,
  cancellation_amount_yen: 2936691,
  average_acquisition_price_yen: 39534.99,
  realized_profit_and_loss: 12868,
  taxes: 2614,
  realized_profit_and_loss_after_tax: 10254,
};
const ORKAN = {
  trade_date: '2026-02-05',
  settlement_date: '2026-02-10',
  fund_name: 'eMAXIS Slim 全世界株式(オール・カントリー)',
  dividends: '再投資型',
  account: '特定',
  shares: 1200000,
  exchange_rate: 0,
  cancellation_unit_price_yen: 22500,
  cancellation_amount_yen: 27000000,
  average_acquisition_price_yen: 21000,
  realized_profit_and_loss: 1800000,
  taxes: 365670,
  realized_profit_and_loss_after_tax: 1434330,
};
const MUTUALFUND_BASE = [SP500_BASE, RAKUTEN_FUND];
// 3行中、ファイル内の S&P500 重複1行のみスキップ → 2件反映
const MUTUALFUND_ADDITIONAL = [SP500_ADDITIONAL, SP500_ADDITIONAL, ORKAN];
const MUTUALFUND_ADDITIONAL_INSERTED = [SP500_ADDITIONAL, ORKAN];

type TabKey = 'dividend' | 'domesticstock' | 'mutualfund';

interface TabFixture {
  listPath: RegExp;
  previewPath: RegExp;
  importPath: RegExp;
  files: Record<string, { previewRows: unknown[]; insertedRows: unknown[] }>;
}

const TAB_FIXTURES: Record<TabKey, TabFixture> = {
  dividend: {
    listPath: /\/api\/v1\/dividends(?:\?.*)?$/,
    previewPath: /\/api\/v1\/dividend-import-validations$/,
    importPath: /\/api\/v1\/dividend-imports$/,
    files: {
      'dividend-base.csv': { previewRows: DIVIDEND_BASE, insertedRows: DIVIDEND_BASE },
      'dividend-additional.csv': {
        previewRows: DIVIDEND_ADDITIONAL,
        insertedRows: DIVIDEND_ADDITIONAL_INSERTED,
      },
    },
  },
  domesticstock: {
    listPath: /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
    previewPath: /\/api\/v1\/domestic-stock-import-validations$/,
    importPath: /\/api\/v1\/domestic-stock-imports$/,
    files: {
      'domesticstock-base.csv': {
        previewRows: DOMESTIC_BASE,
        insertedRows: DOMESTIC_BASE,
      },
      'domesticstock-additional.csv': {
        previewRows: DOMESTIC_ADDITIONAL,
        insertedRows: DOMESTIC_ADDITIONAL_INSERTED,
      },
    },
  },
  mutualfund: {
    listPath: /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
    previewPath: /\/api\/v1\/mutual-fund-import-validations$/,
    importPath: /\/api\/v1\/mutual-fund-imports$/,
    files: {
      'mutualfund-base.csv': {
        previewRows: MUTUALFUND_BASE,
        insertedRows: MUTUALFUND_BASE,
      },
      'mutualfund-additional.csv': {
        previewRows: MUTUALFUND_ADDITIONAL,
        insertedRows: MUTUALFUND_ADDITIONAL_INSERTED,
      },
    },
  },
};

function uploadFileName(body: string): string {
  return /filename="([^"]+)"/.exec(body)?.[1] ?? '';
}

let rowSeq = 0;
function asDbRow(row: unknown): unknown {
  rowSeq += 1;
  return {
    id: `00000000-0000-0000-0000-${String(rowSeq).padStart(12, '0')}`,
    ...(row as Record<string, unknown>),
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  };
}

async function setupMocks(page: Page, db: Record<TabKey, unknown[]>) {
  // 全件削除はデータ消失経路のため、DELETE が確認前・キャンセル後に出ないことを回数で固定する
  const deleteCounts: Record<TabKey, number> = {
    dividend: 0,
    domesticstock: 0,
    mutualfund: 0,
  };
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(USER),
    }),
  );
  for (const [key, fixture] of Object.entries(TAB_FIXTURES) as [TabKey, TabFixture][]) {
    await page.route(fixture.listPath, (route) => {
      if (route.request().method() === 'DELETE') {
        deleteCounts[key] += 1;
        db[key] = [];
        return route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: '{}',
        });
      }
      const data = db[key];
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ data, total: data.length, page: 1, per_page: data.length }),
      });
    });
    await page.route(fixture.previewPath, (route) => {
      const file = fixture.files[uploadFileName(route.request().postData() ?? '')];
      const rows = file?.previewRows ?? [];
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          total_rows: rows.length,
          valid_rows: rows.length,
          errors: [],
          rows,
        }),
      });
    });
    await page.route(fixture.importPath, (route) => {
      const file = fixture.files[uploadFileName(route.request().postData() ?? '')];
      if (!file) {
        return route.fulfill({
          status: 400,
          contentType: 'application/json',
          body: JSON.stringify({ message: '不正なファイルです' }),
        });
      }
      db[key].push(...file.insertedRows.map(asDbRow));
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          inserted: file.insertedRows.length,
          skipped: file.previewRows.length - file.insertedRows.length,
          errors: [],
        }),
      });
    });
  }
  return deleteCounts;
}

interface Scenario {
  tab: TabKey;
  label: string;
  baseFile: string;
  additionalFile: string;
  baseName: string;
  additionalName: string;
}

const SCENARIOS: Scenario[] = [
  {
    tab: 'dividend',
    label: '配当金',
    baseFile: 'dividend-base.csv',
    additionalFile: 'dividend-additional.csv',
    baseName: 'オリックス',
    additionalName: 'トヨタ自動車',
  },
  {
    tab: 'domesticstock',
    label: '国内株式',
    baseFile: 'domesticstock-base.csv',
    additionalFile: 'domesticstock-additional.csv',
    baseName: '商船三井',
    additionalName: '九州電力',
  },
  {
    tab: 'mutualfund',
    label: '投資信託',
    baseFile: 'mutualfund-base.csv',
    additionalFile: 'mutualfund-additional.csv',
    baseName: 'eMAXIS Slim 米国株式(S&P500)',
    additionalName: 'eMAXIS Slim 全世界株式(オール・カントリー)',
  },
];

test.describe('取引明細 CSV 取込・削除', () => {
  for (const scenario of SCENARIOS) {
    test(`${scenario.label} - base取込→追加取込(反映・スキップ)→全件削除`, async ({ page }) => {
      const db: Record<TabKey, unknown[]> = {
        dividend: [],
        domesticstock: [],
        mutualfund: [],
      };
      const deleteCounts = await setupMocks(page, db);
      await page.goto('/receipts');

      if (scenario.tab !== 'dividend') {
        await page.getByRole('tab', { name: scenario.label }).click();
      }
      const fileInput = page.getByTestId('csv-file-input');
      await expect(fileInput).toBeAttached();
      // DB が空の間は全件削除ボタンを出さない
      await expect(page.getByRole('button', { name: /全件削除/ })).toHaveCount(0);

      const fixture = TAB_FIXTURES[scenario.tab].files;
      const base = fixture[scenario.baseFile];
      const additional = fixture[scenario.additionalFile];
      const baseCount = base.insertedRows.length;
      const additionalCount = additional.insertedRows.length;
      const totalCount = baseCount + additionalCount;

      await fileInput.setInputFiles(path.join(fixtureDir(), scenario.baseFile));
      await expect(
        page.getByText(`${base.previewRows.length}件 追加で保存されます`),
      ).toBeVisible();
      await expect(page.getByRole('cell', { name: scenario.baseName })).toBeVisible();
      await page
        .getByRole('button', { name: `${base.previewRows.length}件 追加で保存` })
        .click();
      const notice = page.getByTestId('csv-save-result-notice');
      await expect(notice).toContainText(`${baseCount}件反映`);
      await expect(
        page.getByRole('button', { name: `全件削除 (${baseCount}件)` }),
      ).toBeVisible();
      await expect(page.getByRole('cell', { name: scenario.baseName })).toBeVisible();

      await fileInput.setInputFiles(
        path.join(fixtureDir(), scenario.additionalFile),
      );
      await expect(
        page.getByText(`${additional.previewRows.length}件 追加で保存されます`),
      ).toBeVisible();
      await page
        .getByRole('button', { name: `${additional.previewRows.length}件 追加で保存` })
        .click();
      await expect(notice).toContainText(`${additionalCount}件反映`);
      await expect(notice).toContainText(
        `${additional.previewRows.length - additionalCount}件スキップ`,
      );
      await expect(
        page.getByRole('button', { name: `全件削除 (${totalCount}件)` }),
      ).toBeVisible();
      await expect(page.getByRole('cell', { name: scenario.additionalName })).toBeVisible();

      await page.getByRole('button', { name: /全件削除/ }).click();
      const dialog = page.getByRole('dialog');
      await expect(dialog).toContainText('この操作は取り消せません');
      await expect(dialog).toContainText(`${totalCount}件`);
      // 確認モーダルを開いただけでは DELETE を送らない
      expect(deleteCounts[scenario.tab]).toBe(0);
      await page.keyboard.press('Escape');
      await expect(dialog).toHaveCount(0);
      await expect(
        page.getByRole('button', { name: `全件削除 (${totalCount}件)` }),
      ).toBeVisible();
      expect(deleteCounts[scenario.tab]).toBe(0);

      await page.getByRole('button', { name: /全件削除/ }).click();
      await page.getByRole('button', { name: '削除する' }).click();
      await expect(page.getByText('データがありません')).toBeVisible();
      await expect(page.getByRole('button', { name: /全件削除/ })).toHaveCount(0);
      // 確定時に DELETE が1回だけ送られる
      expect(deleteCounts[scenario.tab]).toBe(1);
      await expect(notice).toHaveCount(0);
    });
  }
});

// 資産管理はタブではなく独立ページで、追加ではなく全件置換になる
const ASSET_TOYOTA = {
  security_code: '7203',
  security_name: 'トヨタ自動車',
  shares: 100,
  executing_shares: 0,
  average_purchase_price: 2500,
  total_purchase_amount: 250000,
  current_price: 2600,
  daily_change: 50,
  market_value: 260000,
  profit_loss_rate: 4.0,
};
const ASSET_SONY = {
  security_code: '6758',
  security_name: 'ソニーグループ',
  shares: 50,
  executing_shares: 0,
  average_purchase_price: 3000,
  total_purchase_amount: 150000,
  current_price: 3200,
  daily_change: -20,
  market_value: 160000,
  profit_loss_rate: 6.67,
};
const ASSET_NINTENDO = {
  security_code: '7974',
  security_name: '任天堂',
  shares: 30,
  executing_shares: 0,
  average_purchase_price: 8000,
  total_purchase_amount: 240000,
  current_price: 8200,
  daily_change: 100,
  market_value: 246000,
  profit_loss_rate: 2.5,
};
const ASSET_MUFJ = {
  security_code: '8306',
  security_name: '三菱ＵＦＪフィナンシャル・グループ',
  shares: 200,
  executing_shares: 0,
  average_purchase_price: 600,
  total_purchase_amount: 120000,
  current_price: 650,
  daily_change: 5,
  market_value: 130000,
  profit_loss_rate: 8.33,
};
const ASSET_BASE = [ASSET_TOYOTA, ASSET_SONY];
// 置換の証明のため、base のソニーを含まない別3件にする
const ASSET_UPDATED = [ASSET_TOYOTA, ASSET_NINTENDO, ASSET_MUFJ];
const ASSET_FILES: Record<string, unknown[]> = {
  'assetbalance-base.csv': ASSET_BASE,
  'assetbalance-updated.csv': ASSET_UPDATED,
};

test.describe('資産管理 CSV 取込・削除', () => {
  test('base2件の取込後にupdated3件で全件置換し全件削除する', async ({ page }) => {
    let db: unknown[] = [];
    // DELETE が確認前・キャンセル後に出ないことを回数で固定する
    let deleteCount = 0;
    await page.route(/\/api\/v1\/session$/, (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(USER),
      }),
    );
    await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ items: [] }),
      }),
    );
    await page.route(/\/api\/v1\/asset-balance-import-validations$/, (route) => {
      const rows = ASSET_FILES[uploadFileName(route.request().postData() ?? '')] ?? [];
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          total_rows: rows.length,
          valid_rows: rows.length,
          errors: [],
          rows,
        }),
      });
    });
    await page.route(/\/api\/v1\/asset-balance-imports$/, (route) => {
      const rows = ASSET_FILES[uploadFileName(route.request().postData() ?? '')];
      if (!rows) {
        return route.fulfill({
          status: 400,
          contentType: 'application/json',
          body: JSON.stringify({ message: '不正なファイルです' }),
        });
      }
      // 全件置換なので既存行を残さず入れ替える
      db = rows.map(asDbRow);
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ inserted: rows.length, skipped: 0, errors: [] }),
      });
    });
    await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
      if (route.request().method() === 'DELETE') {
        deleteCount += 1;
        db = [];
        return route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: '{}',
        });
      }
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ data: db, total: db.length, page: 1, per_page: db.length }),
      });
    });

    const card = (name: string) =>
      page.locator('[data-testid="portfolio-card-identity"]', { hasText: name });

    await page.goto('/assetbalance');
    const fileInput = page.getByTestId('csv-file-input');
    await expect(fileInput).toBeAttached();
    // DB が空の間は全件削除ボタンを出さない
    await expect(page.getByRole('button', { name: /全件削除/ })).toHaveCount(0);
    await expect(page.getByText('資産管理データがありません')).toBeVisible();

    await fileInput.setInputFiles(path.join(fixtureDir(), 'assetbalance-base.csv'));
    await expect(
      page.getByRole('button', { name: '2件 全件置換で保存' }),
    ).toBeEnabled();
    // プレビュー行が一覧に出る
    await expect(card('トヨタ自動車')).toBeVisible();
    await expect(card('ソニーグループ')).toBeVisible();

    await page.getByRole('button', { name: '2件 全件置換で保存' }).click();
    const notice = page.getByTestId('csv-save-result-notice');
    await expect(notice).toContainText('2件反映');
    await expect(notice).toContainText('全件置換');
    await expect(
      page.getByRole('button', { name: '全件削除 (2件)' }),
    ).toBeVisible();

    await fileInput.setInputFiles(path.join(fixtureDir(), 'assetbalance-updated.csv'));
    await expect(
      page.getByRole('button', { name: '3件 全件置換で保存' }),
    ).toBeEnabled();
    await expect(card('任天堂')).toBeVisible();

    await page.getByRole('button', { name: '3件 全件置換で保存' }).click();
    await expect(notice).toContainText('3件反映');
    await expect(
      page.getByRole('button', { name: '全件削除 (3件)' }),
    ).toBeVisible();
    // 追加ではなく置換なので base にだけあった行は消える
    await expect(card('ソニーグループ')).toHaveCount(0);
    await expect(card('任天堂')).toBeVisible();

    await page.getByRole('button', { name: /全件削除/ }).click();
    const dialog = page.getByRole('dialog');
    await expect(dialog).toContainText('資産管理データの全件削除');
    await expect(dialog).toContainText('この操作は取り消せません');
    await expect(dialog).toContainText('3件');
    // 確認モーダルを開いただけでは DELETE を送らない
    expect(deleteCount).toBe(0);
    await page.keyboard.press('Escape');
    await expect(dialog).toHaveCount(0);
    await expect(
      page.getByRole('button', { name: '全件削除 (3件)' }),
    ).toBeVisible();
    expect(deleteCount).toBe(0);

    await page.getByRole('button', { name: /全件削除/ }).click();
    await page.getByRole('button', { name: '削除する' }).click();
    await expect(page.getByText('資産管理データがありません')).toBeVisible();
    await expect(page.getByRole('button', { name: /全件削除/ })).toHaveCount(0);
    // 確定時に DELETE が1回だけ送られる
    expect(deleteCount).toBe(1);
    await expect(notice).toHaveCount(0);
  });
});
