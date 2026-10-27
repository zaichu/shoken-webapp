import type { Page } from '@playwright/test';
import { expect, test, json, paginated } from '../support/test';
import { csvRow, mockCsvKindStats, savedCount } from '../support/csv-page';

/**
 * CSV 管理ページの表そのものの E2E。
 * 件数・状態は csv-kinds/stats をモックし、一覧 API は空で返す。
 */

const USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const LIST_PATHS = [
  /\/api\/v1\/asset-balances(?:\?.*)?$/,
  /\/api\/v1\/fund-balances(?:\?.*)?$/,
  /\/api\/v1\/dividends(?:\?.*)?$/,
  /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
  /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
];

async function mockLists(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(USER)));
  for (const path of LIST_PATHS) {
    await page.route(path, (route) => route.fulfill(json(paginated([]))));
  }
}

/** JST の daysAgo 日前 12:00 を、API が返す UTC の ISO 文字列に直す */
function jstNoonIso(daysAgo: number, hour = 12, minute = 0): string {
  const now = new Date(Date.now() + 9 * 60 * 60 * 1000);
  const utcMillis = Date.UTC(
    now.getUTCFullYear(),
    now.getUTCMonth(),
    now.getUTCDate() - daysAgo,
    hour - 9,
    minute,
  );
  return new Date(utcMillis).toISOString();
}

test('5種類の行が出て、状態が最終取込時刻で変わる', async ({ page }) => {
  await mockLists(page);
  await mockCsvKindStats(page, {
    counts: () => ({ assetbalance: 12, fundbalance: 5 }),
    latest: () => ({
      assetbalance: jstNoonIso(0),
      fundbalance: jstNoonIso(3, 9, 15),
    }),
  });

  await page.goto('/csv');

  for (const kind of [
    'assetbalance',
    'fundbalance',
    'dividend',
    'domesticstock',
    'mutualfund',
  ] as const) {
    await expect(csvRow(page, kind)).toHaveCount(1);
  }

  await expect(savedCount(page, 'assetbalance')).toContainText('12件');
  await expect(csvRow(page, 'assetbalance')).toContainText('今日 12:00 に更新');
  await expect(csvRow(page, 'fundbalance')).toContainText(
    /3日前\(\d{1,2}\/\d{1,2} 09:15\)に更新/,
  );
  await expect(csvRow(page, 'dividend')).toContainText('まだ取得していません');
  await expect(csvRow(page, 'domesticstock')).toContainText('まだ取得していません');
  await expect(csvRow(page, 'mutualfund')).toContainText('まだ取得していません');
});

test('⋯ のメニューは3項目で、件数が0件の削除だけ選べない', async ({ page }) => {
  await mockLists(page);
  await mockCsvKindStats(page, {
    counts: () => ({ assetbalance: 12, fundbalance: 5 }),
  });

  await page.goto('/csv');

  const menu = csvRow(page, 'assetbalance').getByRole('button', { name: 'メニュー' });
  await menu.click();
  await expect(page.getByRole('button', { name: '楽天証券から再取得' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'ファイルから取り込み' })).toBeVisible();
  await expect(page.getByRole('button', { name: '削除', exact: true })).toBeEnabled();

  // 開いたまま別の行のメニューを開くと、表示中の項目がその行のものに切り替わる
  await menu.click();
  await csvRow(page, 'dividend').getByRole('button', { name: 'メニュー' }).click();
  await expect(page.getByRole('button', { name: '削除', exact: true })).toBeDisabled();
});

test('残高は当日だけ新しく、明細は7日以内が新しい', async ({ page }) => {
  await mockLists(page);
  await mockCsvKindStats(page, {
    counts: () => ({
      assetbalance: 1,
      fundbalance: 1,
      dividend: 1,
      domesticstock: 1,
    }),
    latest: () => ({
      assetbalance: jstNoonIso(0),
      fundbalance: jstNoonIso(3),
      dividend: jstNoonIso(3),
      domesticstock: jstNoonIso(8),
    }),
  });

  await page.goto('/csv');

  const statusOf = (kind: 'assetbalance' | 'fundbalance' | 'dividend' | 'domesticstock' | 'mutualfund') =>
    csvRow(page, kind).locator('span[data-status]');
  await expect(statusOf('assetbalance')).toHaveAttribute('data-status', 'fresh');
  await expect(statusOf('fundbalance')).toHaveAttribute('data-status', 'stale');
  await expect(statusOf('dividend')).toHaveAttribute('data-status', 'fresh');
  await expect(statusOf('domesticstock')).toHaveAttribute('data-status', 'stale');
  await expect(statusOf('mutualfund')).toHaveAttribute('data-status', 'never');
});

test('⋯ のメニューは外側を押す・Esc・項目選択で閉じる', async ({ page }) => {
  await mockLists(page);
  await mockCsvKindStats(page, {
    counts: () => ({ assetbalance: 12 }),
  });

  await page.goto('/csv');

  const menuButton = csvRow(page, 'assetbalance').getByRole('button', { name: 'メニュー' });
  const refetch = page.getByRole('button', { name: '楽天証券から再取得' });

  // 開くと最初の項目へフォーカスが移る
  await menuButton.click();
  await expect(refetch).toBeVisible();
  await expect(refetch).toBeFocused();

  // 外側を押すと閉じる
  await page.getByText('楽天証券から最新のデータを取得').click();
  await expect(refetch).toHaveCount(0);

  // Esc で閉じてフォーカスが ⋯ に戻る
  await menuButton.click();
  await expect(refetch).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(refetch).toHaveCount(0);
  await expect(menuButton).toBeFocused();

  // 項目を選んだら閉じる(ファイル選択はキャンセル扱いで閉じるだけにする)
  await menuButton.click();
  await expect(refetch).toBeVisible();
  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'ファイルから取り込み' }).click();
  await expect(refetch).toHaveCount(0);
  await (await chooser).setFiles([]);
});

test('`?kind=` を付けても何も開かない', async ({ page }) => {
  await mockLists(page);
  await mockCsvKindStats(page, {
    counts: () => ({ assetbalance: 12 }),
  });

  // `?kind=` を付けても何も開かない。行ごとの隠し入力は付いている
  await page.goto('/csv?kind=dividend');
  await expect(csvRow(page, 'dividend')).toBeVisible();
  await expect(csvRow(page, 'dividend').getByTestId('csv-file-input')).toBeAttached();
});

test('取得中も取得ボタンの位置は変わらない', async ({ page }) => {
  await mockLists(page);
  await mockCsvKindStats(page, { counts: () => ({}) });
  await page.addInitScript(() => {
    (window as unknown as { chrome: unknown }).chrome = {
      runtime: {
        connect: () => ({
          onMessage: { addListener: () => {} },
          onDisconnect: { addListener: () => {} },
          postMessage: () => {},
          disconnect: () => {},
        }),
      },
    };
  });

  await page.goto('/csv');
  const button = page.getByTestId('csv-fetch-button');
  await expect(button).toHaveText('最新を取得する');
  const before = await button.boundingBox();

  await button.click();
  await expect(button).toHaveText('取得中...');
  const during = await button.boundingBox();

  expect(during?.x).toBe(before?.x);
  expect(during?.y).toBe(before?.y);
});

test.describe('390px', () => {
  test.use({ viewport: { width: 390, height: 844 } });

  test('行の文字は折り返さず、⋯ は各行に1つだけ出る', async ({ page }) => {
    await mockLists(page);
    await mockCsvKindStats(page, {
      counts: () => ({ assetbalance: 12 }),
      latest: () => ({ assetbalance: jstNoonIso(0, 21, 39) }),
    });

    await page.goto('/csv');
    await expect(csvRow(page, 'assetbalance')).toContainText('今日 21:39 に更新');

    // 種類名と状態は1行のまま(折り返すと行の高さが増える)
    const wrapped = await page
      .locator('tbody tr[data-kind] td:nth-child(1), tbody tr[data-kind] td:nth-child(3)')
      .evaluateAll((cells) =>
        cells.map((cell) => {
          const span = cell.querySelector('span') ?? cell;
          const lineHeight = parseFloat(getComputedStyle(span).lineHeight);
          return span.getBoundingClientRect().height > lineHeight * 1.5;
        }),
      );
    expect(wrapped.every((value) => value === false)).toBe(true);

    for (const kind of [
      'assetbalance',
      'fundbalance',
      'dividend',
      'domesticstock',
      'mutualfund',
    ] as const) {
      await expect(csvRow(page, kind).getByRole('button', { name: 'メニュー' })).toHaveCount(1);
    }
  });
});
