import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

async function setupMocks(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
  );
  await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
  );
  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(json({ items: [] })),
  );
}

const TABS = [
  { param: 'all', id: 'asset-kind-tab-all' },
  { param: 'stock', id: 'asset-kind-tab-stock' },
  { param: 'fund', id: 'asset-kind-tab-fund' },
] as const;

test.describe('資産管理のタブと URL', () => {
  test('URL を直接開くとそのタブが開く', async ({ page }) => {
    await setupMocks(page);
    for (const tab of TABS) {
      await page.goto(`/assetbalance?tab=${tab.param}`);
      await expect(page.getByTestId(tab.id)).toHaveAttribute(
        'aria-selected',
        'true',
      );
      await expect(page).toHaveURL(`/assetbalance?tab=${tab.param}`);
    }
  });

  test('再読み込みしても同じタブが開く', async ({ page }) => {
    await setupMocks(page);
    for (const tab of TABS) {
      await page.goto(`/assetbalance?tab=${tab.param}`);
      await expect(page.getByTestId(tab.id)).toHaveAttribute(
        'aria-selected',
        'true',
      );
      await page.reload();
      await expect(page.getByTestId(tab.id)).toHaveAttribute(
        'aria-selected',
        'true',
      );
      await expect(page).toHaveURL(`/assetbalance?tab=${tab.param}`);
    }
  });

  test('知らない値・値なしは最初のタブ(すべて)', async ({ page }) => {
    await setupMocks(page);
    await page.goto('/assetbalance?tab=bogus');
    await expect(page.getByTestId('asset-kind-tab-all')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-kind-tab-all')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('タブを切り替えると URL が変わり履歴は積まれない', async ({ page }) => {
    await setupMocks(page);
    await page.goto('/');
    await page.goto('/assetbalance?tab=stock');
    await expect(page.getByTestId('asset-kind-tab-stock')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await page.getByTestId('asset-kind-tab-fund').click();
    await expect(page.getByTestId('asset-kind-tab-fund')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await expect(page).toHaveURL('/assetbalance?tab=fund');

    // タブ切替は replace のため、戻るで前のページ(/)に戻る
    await page.goBack();
    await expect(page).toHaveURL('/');
  });

  test('絞り込んだ国内株式タブからホーム経由で全タブを開くと選択条件が消える', async ({
    page,
  }) => {
    await mockSession(page);
    const stocks = [
      {
        id: 'asset-a',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
        security_code: '9901',
        security_name: '架空自動車A',
        account: '特定',
        shares: 100,
        average_purchase_price: 1000,
        total_purchase_amount: 100000,
        current_price: 1100,
        daily_change: 10,
        price_as_of: '2026-10-20',
        prev_close: 1090,
      },
      {
        id: 'asset-b',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
        security_code: '9902',
        security_name: '架空自動車B',
        account: 'NISA',
        shares: 100,
        average_purchase_price: 1000,
        total_purchase_amount: 100000,
        current_price: 1100,
        daily_change: 10,
        price_as_of: '2026-10-20',
        prev_close: 1090,
      },
    ];
    await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) =>
      route.fulfill(json({ data: stocks, total: stocks.length, page: 1, per_page: 1000 })),
    );
    await page.route(/\/api\/v1\/fund-balances(?:\?.*)?$/, (route) =>
      route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
    );
    await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
      route.fulfill(json({ items: [] })),
    );

    await page.goto('/assetbalance?tab=stock');
    const table = page.getByTestId('asset-holdings-table');
    await expect(table).toBeVisible();
    // 口座で絞り込む
    await page.getByTestId('asset-condition-account').click();
    await page.getByRole('button', { name: 'NISA' }).click();
    await page.getByRole('button', { name: '閉じる' }).click();
    await expect(
      page.getByRole('button', { name: '口座 NISA を解除' }),
    ).toBeVisible();
    await expect(table).not.toContainText('架空自動車A');
    // ホームの構成カードから全タブを開く(SPA 遷移のため絞り込みが残る)
    await page.getByRole('link', { name: '証券Web トップ' }).click();
    await page.getByTestId('home-composition-card').getByRole('link').click();
    await expect(page).toHaveURL('/assetbalance?tab=all');
    await expect(page.getByTestId('asset-kind-tab-all')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    // 選択条件は消え、両口座の行が出る
    await expect(
      page.getByRole('button', { name: '口座 NISA を解除' }),
    ).toHaveCount(0);
    const merged = page.getByTestId('asset-merged-table');
    await expect(merged).toBeVisible();
    await expect(merged).toContainText('架空自動車A');
    await expect(merged).toContainText('架空自動車B');
  });
});

async function touchStart(page: Page, testid: string, x: number, y: number) {
  await page.getByTestId(testid).evaluate(
    (element, { x: clientX, y: clientY }: { x: number; y: number }) => {
      const start = new Touch({
        identifier: 1,
        target: element,
        clientX,
        clientY,
      });
      element.dispatchEvent(
        new TouchEvent('touchstart', {
          touches: [start],
          bubbles: true,
          cancelable: true,
        }),
      );
    },
    { x, y },
  );
}

async function touchMove(page: Page, testid: string, x: number, y: number) {
  await page.getByTestId(testid).evaluate(
    (element, { x: clientX, y: clientY }: { x: number; y: number }) => {
      const moved = new Touch({
        identifier: 1,
        target: element,
        clientX,
        clientY,
      });
      element.dispatchEvent(
        new TouchEvent('touchmove', {
          touches: [moved],
          bubbles: true,
          cancelable: true,
        }),
      );
    },
    { x, y },
  );
}

async function touchEnd(page: Page, testid: string, x: number, y: number) {
  await page.getByTestId(testid).evaluate(
    (element, { x: clientX, y: clientY }: { x: number; y: number }) => {
      const end = new Touch({
        identifier: 1,
        target: element,
        clientX,
        clientY,
      });
      element.dispatchEvent(
        new TouchEvent('touchend', {
          changedTouches: [end],
          bubbles: true,
          cancelable: true,
        }),
      );
    },
    { x, y },
  );
}

test.describe('資産管理のタブと URL(スマホ)', () => {
  test.use({ viewport: { width: 390, height: 844 } });

  test('スワイプでタブを替えると URL が変わる', async ({ page }) => {
    await setupMocks(page);
    await page.goto('/assetbalance');
    await expect(page.getByTestId('asset-kind-tab-all-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await touchStart(page, 'assetbalance-main-stage', 300, 400);
    await touchMove(page, 'assetbalance-main-stage', 190, 400);
    await touchMove(page, 'assetbalance-main-stage', 90, 400);
    await touchEnd(page, 'assetbalance-main-stage', 90, 400);
    await expect(page.getByTestId('asset-kind-tab-stock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await expect(page).toHaveURL('/assetbalance?tab=stock');
  });

  test('タブ指定なしで戻ると既定タブに戻り再読み込みでも同じ', async ({
    page,
  }) => {
    await setupMocks(page);
    await page.goto('/assetbalance?tab=fund');
    await expect(page.getByTestId('asset-kind-tab-fund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    // 下部ナビの資産管理リンク(bare)で戻ると既定タブに戻る
    await page
      .getByRole('navigation', { name: 'メインナビゲーション' })
      .getByRole('link', { name: '資産管理' })
      .click();
    await expect(page.getByTestId('asset-kind-tab-all-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await expect(page).toHaveURL('/assetbalance');
    await page.reload();
    await expect(page.getByTestId('asset-kind-tab-all-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });
});
