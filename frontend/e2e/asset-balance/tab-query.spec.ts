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
});
