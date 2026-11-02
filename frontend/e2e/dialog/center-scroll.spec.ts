import type { Page } from '@playwright/test';
import { expect, test, json, paginated, mockSession } from '../support/test';

test.use({ viewport: { width: 390, height: 844 } });

function dividendRow(index: number) {
  const day = String((index % 28) + 1).padStart(2, '0');
  return {
    id: `00000000-0000-0000-0000-00000000${String(index).padStart(4, '0')}`,
    settlement_date: `2024-03-${day}`,
    product: '特定口座',
    account: '架空証券',
    security_code: `99${String(10 + (index % 80))}`,
    security_name: `架空銘柄${index}`,
    unit_price: 30.0,
    shares: 100,
    dividends_before_tax: 3000,
    taxes: 609,
    net_amount_received: 2391,
    created_at: `2024-03-${day}T00:00:00Z`,
    updated_at: `2024-03-${day}T00:00:00Z`,
  };
}

async function mockLongList(page: Page) {
  await mockSession(page);
  const rows = Array.from({ length: 40 }, (_, index) => dividendRow(index));
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(rows))),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
}

async function appScrollTop(page: Page): Promise<number> {
  return page.evaluate(() => {
    const scroller = document.querySelector<HTMLElement>('.app-scroll');
    return scroller ? scroller.scrollTop : window.scrollY;
  });
}

async function scrolledToBottom(page: Page): Promise<void> {
  await page.evaluate(() => {
    const scroller = document.querySelector<HTMLElement>('.app-scroll');
    if (scroller) {
      scroller.scrollTop = scroller.scrollHeight;
    } else {
      window.scrollTo(0, document.body.scrollHeight);
    }
  });
  await expect
    .poll(async () => appScrollTop(page), { timeout: 5000 })
    .toBeGreaterThan(200);
}

async function dialogCenter(page: Page, testid: string) {
  const dialog = page.getByTestId(testid);
  await expect(dialog).toBeVisible();
  await expect(page.getByRole('dialog')).toBeVisible();
  const box = await dialog.boundingBox();
  expect(box).not.toBeNull();
  const viewport = page.viewportSize() ?? { width: 390, height: 844 };
  const centerX = box!.x + box!.width / 2;
  const centerY = box!.y + box!.height / 2;
  expect(Math.abs(centerX - viewport.width / 2)).toBeLessThan(60);
  expect(Math.abs(centerY - viewport.height / 2)).toBeLessThan(120);
  return dialog;
}

test('スマホ: 一覧の下端で詳細ダイアログを開いても中央に出て後ろは動かない', async ({
  page,
}) => {
  await mockLongList(page);
  await page.goto('/receipts');
  await expect(page.getByTestId('receipt-card')).toHaveCount(40);

  await scrolledToBottom(page);
  const before = await appScrollTop(page);

  const lastCard = page.getByTestId('receipt-card').last();
  await lastCard.getByTestId('receipt-row-button').click();
  const dialog = await dialogCenter(page, 'receipt-detail-dialog');

  await page.mouse.move(8, 8);
  await page.mouse.wheel(0, 800);
  await expect.poll(async () => appScrollTop(page)).toBe(before);

  await dialog.getByTestId('detail-dialog-close').click();
  await expect(dialog).toBeHidden();
  expect(await appScrollTop(page)).toBe(before);
});

test('スマホ: 検索ダイアログは中央に出て後ろは動かない', async ({ page }) => {
  await mockLongList(page);
  await page.goto('/receipts');
  await expect(page.getByTestId('receipt-card')).toHaveCount(40);

  await scrolledToBottom(page);
  const before = await appScrollTop(page);

  // 入口を押すとき指は画面を動かさない。click は要素まで寄せるため
  // 開く操作自体はスクロールなしの発行で行う
  await page.evaluate(() => {
    document
      .querySelector('[data-testid="receipts-search-sheet-entry"]')
      ?.dispatchEvent(new MouseEvent('click', { bubbles: true }));
  });
  const dialog = await dialogCenter(page, 'receipts-search-sheet');

  await page.mouse.move(8, 8);
  await page.mouse.wheel(0, 800);
  await expect.poll(async () => appScrollTop(page)).toBe(before);

  await page.keyboard.press('Escape');
  await expect(dialog).toBeHidden();
  expect(await appScrollTop(page)).toBe(before);
});
