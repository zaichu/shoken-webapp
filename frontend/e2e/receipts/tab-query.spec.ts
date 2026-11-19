import type { Page } from '@playwright/test';
import { expect, test, json, mockSession } from '../support/test';

async function setupMocks(page: Page) {
  await mockSession(page);
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json({ data: [], total: 0, page: 1, per_page: 1 })),
  );
}

const TABS = [
  { param: 'dividend', id: 'tab-dividend' },
  { param: 'stock', id: 'tab-domesticstock' },
  { param: 'fund', id: 'tab-mutualfund' },
] as const;

test.describe('取引明細のタブと URL', () => {
  test('URL を直接開くとそのタブが開く', async ({ page }) => {
    await setupMocks(page);
    for (const tab of TABS) {
      await page.goto(`/receipts?tab=${tab.param}`);
      await expect(page.locator(`#${tab.id}`)).toHaveAttribute(
        'aria-selected',
        'true',
      );
      await expect(page).toHaveURL(`/receipts?tab=${tab.param}`);
    }
  });

  test('再読み込みしても同じタブが開く', async ({ page }) => {
    await setupMocks(page);
    for (const tab of TABS) {
      await page.goto(`/receipts?tab=${tab.param}`);
      await expect(page.locator(`#${tab.id}`)).toHaveAttribute(
        'aria-selected',
        'true',
      );
      await page.reload();
      await expect(page.locator(`#${tab.id}`)).toHaveAttribute(
        'aria-selected',
        'true',
      );
      await expect(page).toHaveURL(`/receipts?tab=${tab.param}`);
    }
  });

  test('知らない値・値なしは最初のタブ(配当金)', async ({ page }) => {
    await setupMocks(page);
    await page.goto('/receipts?tab=bogus');
    await expect(page.locator('#tab-dividend')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await page.goto('/receipts');
    await expect(page.locator('#tab-dividend')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('タブを切り替えると URL が変わり履歴は積まれない', async ({ page }) => {
    await setupMocks(page);
    await page.goto('/');
    await page.goto('/receipts?tab=stock');
    await expect(page.locator('#tab-domesticstock')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await page.locator('#tab-mutualfund').click();
    await expect(page.locator('#tab-mutualfund')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await expect(page).toHaveURL('/receipts?tab=fund');

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

test.describe('取引明細のタブと URL(スマホ)', () => {
  test.use({ viewport: { width: 390, height: 844 } });

  test('スワイプでタブを替えると URL が変わる', async ({ page }) => {
    await setupMocks(page);
    await page.goto('/receipts');
    await expect(page.locator('#tab-dividend-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    await touchStart(page, 'receipts-workspace', 300, 400);
    await touchMove(page, 'receipts-workspace', 190, 400);
    await touchMove(page, 'receipts-workspace', 90, 400);
    await touchEnd(page, 'receipts-workspace', 90, 400);
    await expect(page.locator('#tab-domesticstock-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await expect(page).toHaveURL('/receipts?tab=stock');
  });

  test('タブ指定なしで戻ると設定の初期タブに戻り再読み込みでも同じ', async ({
    page,
  }) => {
    await setupMocks(page);
    await page.goto('/receipts?tab=fund');
    await expect(page.locator('#tab-mutualfund-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );

    // 下部ナビの取引明細リンク(bare)で戻ると設定の初期タブに戻る
    await page
      .getByRole('navigation', { name: 'メインナビゲーション' })
      .getByRole('link', { name: '取引明細' })
      .click();
    await expect(page.locator('#tab-dividend-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await expect(page).toHaveURL('/receipts');
    await page.reload();
    await expect(page.locator('#tab-dividend-mobile')).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });
});
