import type { Locator, Page } from '@playwright/test';
import { expect, json } from './test';

/** 表の行を指す data-kind 値(API・拡張機能と同じ) */
export type CsvKindParam =
  | 'assetbalance'
  | 'fundbalance'
  | 'dividend'
  | 'domesticstock'
  | 'mutualfund';

const KIND_LABELS: Record<CsvKindParam, string> = {
  assetbalance: '資産管理(国内株式)',
  fundbalance: '資産管理(投資信託)',
  dividend: '取引明細(配当金)',
  domesticstock: '取引明細(国内株式)',
  mutualfund: '取引明細(投資信託)',
};

/** CSV 管理ページの1行 */
export function csvRow(page: Page, kind: CsvKindParam): Locator {
  return page.locator(`tr[data-kind="${kind}"]`);
}

/** 行の「保存済み」件数 */
export function savedCount(page: Page, kind: CsvKindParam): Locator {
  return csvRow(page, kind).getByTestId('saved-count');
}

/** 行の状態文 */
export function rowStatus(page: Page, kind: CsvKindParam): Locator {
  return csvRow(page, kind).locator('span[data-status]');
}

/** ⋯ メニューを開く */
export async function openCsvMenu(page: Page, kind: CsvKindParam): Promise<void> {
  await csvRow(page, kind).getByRole('button', { name: 'メニュー' }).click();
  await expect(page.getByRole('button', { name: '楽天証券から再取得' })).toBeVisible();
}

/**
 * ⋯ メニューからファイル取り込みで OS の選択を開き、ファイルを渡す。
 * 新しい流れはプレビューなしでそのまま保存し、結果は行の状態に出る。
 */
export async function importCsvFile(
  page: Page,
  kind: CsvKindParam,
  filePath: string,
): Promise<void> {
  await openCsvMenu(page, kind);
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'ファイルから取り込み' }).click();
  await (await chooser).setFiles(filePath);
}

/** ⋯ メニューから削除確認を開く */
export async function openCsvDelete(page: Page, kind: CsvKindParam): Promise<void> {
  await openCsvMenu(page, kind);
  await page.getByRole('button', { name: '削除', exact: true }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
}

interface StatsOptions {
  /** 保存済み件数。呼ばれるたびに読むので、テスト中の DB 更新をそのまま反映できる */
  counts: () => Partial<Record<CsvKindParam, number>>;
  /** 最終取込日時(ISO8601, UTC)。状態の表示を固定したいときだけ指定する */
  latest?: () => Partial<Record<CsvKindParam, string>>;
}

/** 種類ごとの件数・最終取込日時を返す API をモックする */
export async function mockCsvKindStats(page: Page, options: StatsOptions): Promise<void> {
  await page.route(/\/api\/v1\/csv-kinds\/stats$/, (route) => {
    const counts = options.counts();
    const latest = options.latest?.() ?? {};
    return route.fulfill(
      json({
        kinds: (Object.keys(KIND_LABELS) as CsvKindParam[]).map((kind) => ({
          kind,
          label: KIND_LABELS[kind],
          count: counts[kind] ?? 0,
          ...(latest[kind] ? { latest_imported_at: latest[kind] } : {}),
        })),
      }),
    );
  });
}
