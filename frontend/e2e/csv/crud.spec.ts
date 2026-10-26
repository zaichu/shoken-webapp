import type { Page } from '@playwright/test';
import { expect, test } from '../support/test';
import {
  type CsvKindParam,
  csvImportRow,
  csvRow,
  mockCsvKindStats,
  openCsvDelete,
  savedCount,
} from '../support/csv-page';
import * as path from 'path';

/**
 * 取引明細・資産管理の CSV 管理ページ E2E。
 * API はモックし、fixture CSV を使う。
 */

function fixtureDir(): string {
  return path.resolve(test.info().project.testDir, 'fixtures/csv');
}

const USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

// backend の一覧行と同じフィールド名。DB 行として登録時に id・タイムスタンプを付ける
const SYN_COMM = {
  settlement_date: '2025-12-08',
  product: '国内株式',
  account: '特定・一般',
  security_code: '9911',
  security_name: '架空通信',
  unit_price: 30,
  shares: 500,
  dividends_before_tax: 15000,
  taxes: 3047,
  net_amount_received: 11953,
};
const SYN_TELECOM = {
  settlement_date: '2025-12-08',
  product: '国内株式',
  account: '特定・一般',
  security_code: '9910',
  security_name: '架空電信電話',
  unit_price: 4,
  shares: 2500,
  dividends_before_tax: 10000,
  taxes: 2031,
  net_amount_received: 7969,
};
const SYN_AUTO = {
  settlement_date: '2025-12-09',
  product: '国内株式',
  account: '特定・一般',
  security_code: '9904',
  security_name: '架空自動車',
  unit_price: 60,
  shares: 200,
  dividends_before_tax: 12000,
  taxes: 2438,
  net_amount_received: 9562,
};
const DIVIDEND_BASE = [
  {
    settlement_date: '2025-12-09',
    product: '国内株式',
    account: '特定・一般',
    security_code: '9908',
    security_name: '架空リース',
    unit_price: 90.00,
    shares: 200,
    dividends_before_tax: 18000,
    taxes: 3656,
    net_amount_received: 14344,
  },
  {
    settlement_date: '2025-12-10',
    product: '国内株式',
    account: '特定・一般',
    security_code: '9907',
    security_name: '架空フィナンシャル・グループ',
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
    security_code: '9906',
    security_name: '架空エレクトロン',
    unit_price: 1600,
    shares: 10,
    dividends_before_tax: 16000,
    taxes: 3252,
    net_amount_received: 12748,
  },
];
// 5行中 SYN_COMM 2行は重複スキップ → 3件反映
const DIVIDEND_ADDITIONAL = [SYN_COMM, SYN_COMM, SYN_COMM, SYN_TELECOM, SYN_AUTO];
const DIVIDEND_ADDITIONAL_INSERTED = [SYN_COMM, SYN_TELECOM, SYN_AUTO];

const SYN_ENERGY = {
  trade_date: '2026-02-09',
  settlement_date: '2026-02-12',
  security_code: '9902',
  security_name: '架空エネルギーホールディングス',
  account: '特定',
  shares: 100,
  asked_price: 1500,
  proceeds: 150000,
  purchase_price: 1400,
  realized_profit_and_loss: 10000,
  taxes: 2031,
  realized_profit_and_loss_after_tax: 7969,
};
const SYN_POWER = {
  trade_date: '2026-01-15',
  settlement_date: '2026-01-18',
  security_code: '9912',
  security_name: '架空電力ホールディングス',
  account: '特定',
  shares: 200,
  asked_price: 600,
  proceeds: 120000,
  purchase_price: 680,
  realized_profit_and_loss: -16000,
  taxes: 0,
  realized_profit_and_loss_after_tax: -16000,
};
const SYN_SHIPPING = {
  trade_date: '2026-01-20',
  settlement_date: '2026-01-23',
  security_code: '9909',
  security_name: '架空海運',
  account: '特定',
  shares: 50,
  asked_price: 5200,
  proceeds: 260000,
  purchase_price: 4800,
  realized_profit_and_loss: 20000,
  taxes: 4063,
  realized_profit_and_loss_after_tax: 15937,
};
const SYN_POWER_WEST = {
  trade_date: '2026-02-12',
  settlement_date: '2026-02-16',
  security_code: '9913',
  security_name: '架空電力西',
  account: '特定',
  shares: 100,
  asked_price: 2000,
  proceeds: 200000,
  purchase_price: 1900,
  realized_profit_and_loss: 10000,
  taxes: 2031,
  realized_profit_and_loss_after_tax: 7969,
};
const SYN_ELECTRIC = {
  trade_date: '2026-02-18',
  settlement_date: '2026-02-21',
  security_code: '9903',
  security_name: '架空電機',
  account: '特定',
  shares: 150,
  asked_price: 2100,
  proceeds: 315000,
  purchase_price: 2200,
  realized_profit_and_loss: -15000,
  taxes: 0,
  realized_profit_and_loss_after_tax: -15000,
};
const DOMESTIC_BASE = [SYN_ENERGY, SYN_POWER, SYN_SHIPPING];
// 11行中、base 登録済みの架空海運 index=1 のみスキップ → 10件反映
const DOMESTIC_ADDITIONAL = [
  SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_POWER_WEST, SYN_ELECTRIC,
];
const DOMESTIC_ADDITIONAL_INSERTED = [
  SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_SHIPPING, SYN_POWER_WEST, SYN_ELECTRIC,
];

const SYN_US_EQUITY_BASE = {
  trade_date: '2022-10-28',
  settlement_date: '2022-11-02',
  fund_name: '架空インデックス 米国株式',
  dividends: '再投資型',
  account: '特定',
  shares: 3000000,
  exchange_rate: 0,
  cancellation_unit_price_yen: 12000,
  cancellation_amount_yen: 3600000,
  average_acquisition_price_yen: 10000,
  realized_profit_and_loss: 600000,
  taxes: 121860,
  realized_profit_and_loss_after_tax: 478140,
};
const SYN_ALL_US_EQUITY = {
  trade_date: '2022-12-26',
  settlement_date: '2022-12-29',
  fund_name: '架空全米株式インデックス・ファンド',
  dividends: '再投資型',
  account: '特定',
  shares: 500000,
  exchange_rate: 0,
  cancellation_unit_price_yen: 15200,
  cancellation_amount_yen: 7600000,
  average_acquisition_price_yen: 14800,
  realized_profit_and_loss: 200000,
  taxes: 40620,
  realized_profit_and_loss_after_tax: 159380,
};
const SYN_US_EQUITY_ADDITIONAL = {
  trade_date: '2026-01-08',
  settlement_date: '2026-01-14',
  fund_name: '架空インデックス 米国株式',
  dividends: '再投資型',
  account: '特定',
  shares: 700000,
  exchange_rate: 0,
  cancellation_unit_price_yen: 30000,
  cancellation_amount_yen: 2100000,
  average_acquisition_price_yen: 29000,
  realized_profit_and_loss: 70000,
  taxes: 14217,
  realized_profit_and_loss_after_tax: 55783,
};
const SYN_GLOBAL_EQUITY = {
  trade_date: '2026-02-05',
  settlement_date: '2026-02-10',
  fund_name: '架空インデックス 全世界株式(オール・ワールド)',
  dividends: '再投資型',
  account: '特定',
  shares: 1200000,
  exchange_rate: 0,
  cancellation_unit_price_yen: 22500,
  cancellation_amount_yen: 27000000,
  average_acquisition_price_yen: 21000,
  realized_profit_and_loss: 1800000,
  taxes: 365580,
  realized_profit_and_loss_after_tax: 1434420,
};
const MUTUALFUND_BASE = [SYN_US_EQUITY_BASE, SYN_ALL_US_EQUITY];
// 3行中、ファイル内の S&P500 重複1行のみスキップ → 2件反映
const MUTUALFUND_ADDITIONAL = [SYN_US_EQUITY_ADDITIONAL, SYN_US_EQUITY_ADDITIONAL, SYN_GLOBAL_EQUITY];
const MUTUALFUND_ADDITIONAL_INSERTED = [SYN_US_EQUITY_ADDITIONAL, SYN_GLOBAL_EQUITY];

type TabKey = 'dividend' | 'domesticstock' | 'mutualfund';
type KindParam = 'dividend' | 'stock' | 'fund';

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
  await mockCsvKindStats(page, {
    counts: () => ({
      dividend: db.dividend.length,
      domesticstock: db.domesticstock.length,
      mutualfund: db.mutualfund.length,
    }),
  });
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
  kind: KindParam;
  /** 表の行を指す data-kind。旧い kind 値とは別 */
  row: CsvKindParam;
  label: string;
  rowLabel: string;
  baseFile: string;
  additionalFile: string;
  baseName: string;
  additionalName: string;
}

const SCENARIOS: Scenario[] = [
  {
    tab: 'dividend',
    kind: 'dividend',
    row: 'dividend',
    label: '配当金',
    rowLabel: '取引明細(配当金)',
    baseFile: 'dividend-base.csv',
    additionalFile: 'dividend-additional.csv',
    baseName: '架空リース',
    additionalName: '架空自動車',
  },
  {
    tab: 'domesticstock',
    kind: 'stock',
    row: 'domesticstock',
    label: '国内株式',
    rowLabel: '取引明細(国内株式)',
    baseFile: 'domesticstock-base.csv',
    additionalFile: 'domesticstock-additional.csv',
    baseName: '架空海運',
    additionalName: '架空電力西',
  },
  {
    tab: 'mutualfund',
    kind: 'fund',
    row: 'mutualfund',
    label: '投資信託',
    rowLabel: '取引明細(投資信託)',
    baseFile: 'mutualfund-base.csv',
    additionalFile: 'mutualfund-additional.csv',
    baseName: '架空インデックス 米国株式',
    additionalName: '架空インデックス 全世界株式(オール・ワールド)',
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

      // 一覧から CSV 管理ページへ移動する
      await page.goto('/receipts');
      if (scenario.tab !== 'dividend') {
        await page.getByRole('tab', { name: scenario.label }).click();
      }
      await page.getByRole('link', { name: 'CSV管理' }).click();
      await expect(page).toHaveURL(`/csv?kind=${scenario.kind}`);

      // 表は5種類を常に出し、`?kind=` で指定した種類の取り込みだけを開く
      await expect(csvRow(page, scenario.row)).toHaveCount(1);
      const fileInput = csvImportRow(page).getByTestId('csv-file-input');
      await expect(fileInput).toBeAttached();
      await expect(savedCount(page, scenario.row)).toContainText('0件');

      const fixture = TAB_FIXTURES[scenario.tab].files;
      const base = fixture[scenario.baseFile];
      const additional = fixture[scenario.additionalFile];
      const baseCount = base.insertedRows.length;
      const additionalCount = additional.insertedRows.length;
      const totalCount = baseCount + additionalCount;

      // 取得中・保存中は input が disabled で change イベントが捨てられるため、有効化を待ってから投入する
      await expect(fileInput).toBeEnabled();
      await fileInput.setInputFiles(path.join(fixtureDir(), scenario.baseFile));
      await expect(page.getByText(`${base.previewRows.length}件を追加で保存できます`)).toBeVisible();
      await expect(
        page.getByTestId('receipt-table-body').getByRole('cell', { name: scenario.baseName }),
      ).toBeVisible();
      await page
        .getByRole('button', { name: `${base.previewRows.length}件を保存する` })
        .click();
      const notice = page.getByTestId('csv-save-result-notice');
      await expect(notice).toContainText(`${baseCount}件反映`);
      // 保存後は行の件数と状態を取り直す
      await expect(savedCount(page, scenario.row)).toContainText(`${baseCount}件`);

      await expect(fileInput).toBeEnabled();
      await fileInput.setInputFiles(path.join(fixtureDir(), scenario.additionalFile));
      await expect(
        page.getByText(`${additional.previewRows.length}件を追加で保存できます`),
      ).toBeVisible();
      await page
        .getByRole('button', { name: `${additional.previewRows.length}件を保存する` })
        .click();
      await expect(notice).toContainText(`${additionalCount}件反映`);
      await expect(notice).toContainText(
        `${additional.previewRows.length - additionalCount}件スキップ`,
      );

      // 削除は行の⋯メニューから確認モーダルで行う
      await openCsvDelete(page, scenario.row);
      const modal = page.getByRole('dialog');
      await expect(modal).toContainText(`${scenario.rowLabel}を削除しますか?`);
      await expect(modal).toContainText('元に戻せません');
      await expect(modal).toContainText(`${totalCount}件`);

      // 初期フォーカスは「キャンセル」にある
      const cancelButton = modal.getByRole('button', { name: 'キャンセル' });
      await expect(cancelButton).toBeFocused();

      // キャンセルでモーダルが閉じ、DELETE は送られない
      await cancelButton.click();
      await expect(modal).toBeHidden();
      expect(deleteCounts[scenario.tab]).toBe(0);

      // 再度開いて確定する
      await openCsvDelete(page, scenario.row);
      await modal.getByRole('button', { name: `${totalCount}件を削除する` }).click();
      // 確定時に DELETE が1回だけ送られる
      expect(deleteCounts[scenario.tab]).toBe(1);
      await expect(modal).toBeHidden();
      // 削除後は件数を取り直し、その行の削除は選べなくなる
      await expect(savedCount(page, scenario.row)).toContainText('0件');
    });
  }

  test('削除が失敗したらモーダルにエラーを残し、再試行で削除できる', async ({ page }) => {
    const rows = DIVIDEND_BASE.map(asDbRow);
    let savedRows = rows.length;
    let deleteCount = 0;
    await mockCsvKindStats(page, { counts: () => ({ dividend: savedRows }) });
    await page.route(/\/api\/v1\/session$/, (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(USER),
      }),
    );
    await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) => {
      if (route.request().method() === 'DELETE') {
        deleteCount += 1;
        if (deleteCount === 1) {
          return route.fulfill({
            status: 400,
            contentType: 'application/json',
            body: JSON.stringify({ error: { message: '削除に失敗しました' } }),
          });
        }
        savedRows = 0;
        return route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: '{}',
        });
      }
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ data: rows, total: rows.length, page: 1, per_page: rows.length }),
      });
    });

    await page.goto('/csv?kind=dividend');
    const count = savedCount(page, 'dividend');
    await expect(count).toContainText('3件', { timeout: 10000 });

    await openCsvDelete(page, 'dividend');
    const modal = page.getByRole('dialog');
    await expect(modal).toBeVisible();
    // 開いた時点で件数を取り直すため、確定は件数が確定してから押せる
    const confirmButton = modal.getByRole('button', { name: '3件を削除する' });
    await expect(confirmButton).toBeEnabled();
    await confirmButton.click();

    // 失敗: モーダルは開いたまま理由を出す
    await expect(modal.getByRole('alert')).toContainText('削除に失敗しました', {
      timeout: 10000,
    });
    expect(deleteCount).toBe(1);

    // 同じモーダルから再試行できる
    await confirmButton.click();
    await expect(modal).toBeHidden();
    expect(deleteCount).toBe(2);
    await expect(count).toContainText('0件');
  });

  test('取込が400で失敗したときはサーバーの理由をそのまま表示する', async ({ page }) => {
    const db: Record<TabKey, unknown[]> = {
      dividend: [],
      domesticstock: [],
      mutualfund: [],
    };
    await setupMocks(page, db);
    await page.route(TAB_FIXTURES.dividend.importPath, (route) =>
      route.fulfill({
        status: 400,
        contentType: 'application/json',
        body: JSON.stringify({
          error: {
            code: 'CSV_ERROR',
            message: 'CSVファイル（.csv）のみアップロードできます',
          },
        }),
      }),
    );
    await page.goto('/csv?kind=dividend');

    const fileInput = page.getByTestId('csv-file-input');
    await expect(fileInput).toBeEnabled();
    await fileInput.setInputFiles(path.join(fixtureDir(), 'dividend-base.csv'));
    const saveCount = TAB_FIXTURES.dividend.files['dividend-base.csv'].previewRows.length;
    await page.getByRole('button', { name: `${saveCount}件を保存する` }).click();

    await expect(page.getByRole('alert')).toContainText(
      'CSVファイル（.csv）のみアップロードできます',
    );
  });
});

// 資産管理は取引明細と違い、追加ではなく全件置換になる
// モック応答は fixture CSV の行と一致させる
const ASSET_SYN_RESOURCE = {
  security_code: '9901',
  security_name: '架空資源開発',
  shares: 200,
  average_purchase_price: 2000,
  total_purchase_amount: 400000,
  current_price: 3000,
  daily_change: 50,
  market_value: 600000,
  profit_loss_rate: 50.00,
};
const ASSET_SYN_GAME = {
  security_code: '9905',
  security_name: '架空ゲーム',
  shares: 1000,
  average_purchase_price: 5000,
  total_purchase_amount: 5000000,
  current_price: 7000,
  daily_change: 100,
  market_value: 7000000,
  profit_loss_rate: 40.00,
};
const ASSET_SYN_RESOURCE_UPDATED = {
  ...ASSET_SYN_RESOURCE,
  shares: 300,
  total_purchase_amount: 600000,
  market_value: 900000,
};
const ASSET_MUFJ = {
  security_code: '9907',
  security_name: '架空フィナンシャルＧ',
  shares: 300,
  average_purchase_price: 3000,
  total_purchase_amount: 900000,
  current_price: 2900,
  daily_change: 90,
  market_value: 870000,
  profit_loss_rate: -3.33,
};
const ASSET_SYN_COMM = {
  security_code: '9911',
  security_name: '架空通信',
  shares: 600,
  average_purchase_price: 2000,
  total_purchase_amount: 1200000,
  current_price: 2500,
  daily_change: 20,
  market_value: 1500000,
  profit_loss_rate: 25,
};
const ASSET_BASE = [ASSET_SYN_RESOURCE, ASSET_SYN_GAME];
// 置換の証明のため、base の架空ゲームを含まない別3件にする
const ASSET_UPDATED = [ASSET_SYN_RESOURCE_UPDATED, ASSET_MUFJ, ASSET_SYN_COMM];
const ASSET_FILES: Record<string, unknown[]> = {
  'assetbalance-base.csv': ASSET_BASE,
  'assetbalance-updated.csv': ASSET_UPDATED,
};

test.describe('資産管理 CSV 取込・削除', () => {
  async function setupAssetApi(
    page: Page,
    overrides?: {
      previewBody?: (rows: unknown[]) => object;
      importBody?: (rows: unknown[]) => object;
      savedRows?: (rows: unknown[]) => unknown[];
    },
  ) {
    const api = { db: [] as unknown[], deleteCount: 0 };
    await mockCsvKindStats(page, { counts: () => ({ assetbalance: api.db.length }) });
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
        body: JSON.stringify(
          overrides?.previewBody?.(rows) ?? {
            total_rows: rows.length,
            valid_rows: rows.length,
            errors: [],
            rows,
          },
        ),
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
      api.db = (overrides?.savedRows?.(rows) ?? rows).map(asDbRow);
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(
          overrides?.importBody?.(rows) ?? {
            inserted: rows.length,
            skipped: 0,
            errors: [],
          },
        ),
      });
    });
    await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
      if (route.request().method() === 'DELETE') {
        api.deleteCount += 1;
        api.db = [];
        return route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: '{}',
        });
      }
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          data: api.db,
          total: api.db.length,
          page: 1,
          per_page: api.db.length,
        }),
      });
    });
    return api;
  }

  test('base2件の取込後にupdated3件で全件置換し全件削除する', async ({ page }) => {
    const api = await setupAssetApi(page);

    // 一覧から CSV 管理ページへ移動する
    await page.goto('/assetbalance');
    await expect(page.getByText('資産管理データがありません')).toBeVisible();
    await page.getByRole('link', { name: 'CSV管理' }).click();
    await expect(page).toHaveURL('/csv');

    const fileInput = csvImportRow(page).getByTestId('csv-file-input');
    await expect(fileInput).toBeAttached();
    await expect(savedCount(page, 'assetbalance')).toContainText('0件');

    await expect(fileInput).toBeEnabled();
    await fileInput.setInputFiles(path.join(fixtureDir(), 'assetbalance-base.csv'));
    await expect(page.getByRole('button', { name: '2件 全件置換で保存' })).toBeEnabled();
    await expect(page.getByText('2件を全件置換で保存できます')).toBeVisible();
    // プレビュー行がページ内に出る(資産管理はテーブル表示)
    const preview = page.getByTestId('csv-preview');
    await expect(preview.getByText('架空資源開発').first()).toBeVisible();
    await expect(preview.getByText('架空ゲーム').first()).toBeVisible();

    await page.getByRole('button', { name: '2件 全件置換で保存' }).click();
    const notice = page.getByTestId('csv-save-result-notice');
    await expect(notice).toContainText('2件反映');
    await expect(savedCount(page, 'assetbalance')).toContainText('2件');

    await expect(fileInput).toBeEnabled();
    await fileInput.setInputFiles(path.join(fixtureDir(), 'assetbalance-updated.csv'));
    await expect(page.getByRole('button', { name: '3件 全件置換で保存' })).toBeEnabled();
    await expect(
      page.getByTestId('csv-preview').getByText('架空フィナンシャルＧ').first(),
    ).toBeVisible();

    await page.getByRole('button', { name: '3件 全件置換で保存' }).click();
    await expect(notice).toContainText('3件反映');
    await expect(savedCount(page, 'assetbalance')).toContainText('3件');

    // 削除は国内株式の行の⋯メニューから確認モーダルで行う
    await openCsvDelete(page, 'assetbalance');
    const modal = page.getByRole('dialog');
    await expect(modal).toContainText('資産管理(国内株式)を削除しますか?');
    await expect(modal).toContainText('元に戻せません');
    await expect(modal).toContainText('3件');

    // 初期フォーカスは「キャンセル」にある
    const cancelButton = modal.getByRole('button', { name: 'キャンセル' });
    await expect(cancelButton).toBeFocused();

    // キャンセルでモーダルが閉じ、DELETE は送られない
    await cancelButton.click();
    await expect(modal).toBeHidden();
    expect(api.deleteCount).toBe(0);

    // 再度開いて確定する
    await openCsvDelete(page, 'assetbalance');
    await modal.getByRole('button', { name: '3件を削除する' }).click();
    // 確定時に DELETE が1回だけ送られる
    expect(api.deleteCount).toBe(1);
    await expect(modal).toBeHidden();
    await expect(savedCount(page, 'assetbalance')).toContainText('0件');
  });

  test('不正行を含むCSVは修正するまで全件置換で保存できない', async ({ page }) => {
    const rowErrors = [
      { row: 2, message: '銘柄コードが不正です' },
      { row: 3, message: '株数が不正です' },
    ];
    await setupAssetApi(page, {
      previewBody: () => ({
        total_rows: 3,
        valid_rows: 1,
        errors: rowErrors,
        rows: [ASSET_SYN_RESOURCE],
      }),
    });

    await page.goto('/csv');
    const fileInput = page.getByTestId('csv-file-input');
    await expect(fileInput).toBeEnabled();
    await fileInput.setInputFiles(path.join(fixtureDir(), 'assetbalance-base.csv'));

    // 全件置換はエラー行を含む CSV を backend が拒否するため、有効行だけの保存を約束しない
    const notice = page.getByTestId('csv-preview-notice');
    await expect(notice).toContainText('全件置換で保存できません');
    await expect(notice).toContainText('2行目: 銘柄コードが不正です');
    await expect(
      page.getByRole('button', { name: /全件置換で保存/ }).first(),
    ).toBeDisabled();
    const preview = page.getByTestId('csv-preview');
    await expect(preview.getByText('架空資源開発').first()).toBeVisible();
    await expect(preview.getByText('架空ゲーム')).toHaveCount(0);
  });
});
