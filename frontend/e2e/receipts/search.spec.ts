import type { Page } from '@playwright/test';
import { expect, test, json, paginated } from '../support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000011',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const DIVIDENDS = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    settlement_date: '2024-03-01',
    product: '特定口座',
    account: 'SBI証券',
    security_code: '9904',
    security_name: '架空自動車',
    unit_price: 30.0,
    shares: 100,
    dividends_before_tax: 3000,
    taxes: 609,
    net_amount_received: 2391,
    created_at: '2024-03-01T00:00:00Z',
    updated_at: '2024-03-01T00:00:00Z',
  },
  {
    id: '00000000-0000-0000-0000-000000000002',
    settlement_date: '2023-06-01',
    product: 'NISA口座',
    account: '楽天証券',
    security_code: '9910',
    security_name: '架空電信電話',
    unit_price: 120.0,
    shares: 200,
    dividends_before_tax: 5000,
    taxes: 1000,
    net_amount_received: 4000,
    created_at: '2023-06-01T00:00:00Z',
    updated_at: '2023-06-01T00:00:00Z',
  },
];

async function mockReceipts(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(MOCK_USER)));
  await page.route(/\/api\/v1\/dividends(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated(DIVIDENDS))),
  );
  await page.route(/\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
  await page.route(/\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/, (route) =>
    route.fulfill(json(paginated([]))),
  );
}

async function gotoReceipts(page: Page, width: number, height: number) {
  await page.setViewportSize({ width, height });
  await mockReceipts(page);
  await page.goto('/receipts');
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('2');
}

function pcSearch(page: Page) {
  return page.getByRole('search', { name: '取引明細の検索' });
}

// 閉じた後でトリガーのボタンへフォーカスが戻る。そこから Shift+Tab で
// 検索欄内の解除ボタンまで戻り、Enter で外せることを確かめる
async function shiftTabTo(page: Page, name: string, maxSteps = 12) {
  for (let step = 0; step < maxSteps; step += 1) {
    const label = await page.evaluate(() => {
      const element = document.activeElement as HTMLElement | null;
      return element?.getAttribute('aria-label') ?? element?.textContent ?? '';
    });
    if (label === name) {
      return;
    }
    await page.keyboard.press('Shift+Tab');
  }
  throw new Error(`フォーカスが「${name}」に届かない`);
}

test('PC: 口座の条件を足すとタグが出て一覧・件数・集計が同じ対象に絞られる', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  await pcSearch(page).getByRole('button', { name: '＋ 口座' }).click();
  const dialog = page.getByRole('dialog', { name: '口座を選ぶ' });
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: 'SBI証券', exact: true }).click();

  // 選ぶとすぐ反映され、小窓が閉じてタグが出る
  await expect(dialog).toBeHidden();
  await expect(page.getByRole('button', { name: '口座 SBI証券 を解除' })).toBeVisible();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
  await expect(pcSearch(page)).toContainText('1件 / 2件');

  const table = page.getByRole('table');
  await expect(table).toContainText('架空自動車');
  await expect(table).not.toContainText('架空電信電話');
});

test('PC: タグの × で条件を外す', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  await pcSearch(page).getByRole('button', { name: '＋ 口座' }).click();
  await page
    .getByRole('dialog', { name: '口座を選ぶ' })
    .getByRole('button', { name: 'SBI証券', exact: true })
    .click();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');

  await page.getByRole('button', { name: '口座 SBI証券 を解除' }).click();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('2');
  await expect(page.getByRole('button', { name: '口座 SBI証券 を解除' })).toHaveCount(0);
});

test('PC: すべてクリアで全条件を外す', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  await pcSearch(page).getByRole('button', { name: '＋ 口座' }).click();
  await page
    .getByRole('dialog', { name: '口座を選ぶ' })
    .getByRole('button', { name: 'SBI証券', exact: true })
    .click();
  await pcSearch(page).getByRole('button', { name: '＋ 商品' }).click();
  await page
    .getByRole('dialog', { name: '商品を選ぶ' })
    .getByRole('button', { name: '特定口座', exact: true })
    .click();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');

  await pcSearch(page).getByRole('button', { name: 'すべてクリア' }).click();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('2');
  await expect(pcSearch(page)).toContainText('2件 / 2件');
});

test('PC: 期間の小窓で「年」を選んで絞り込める', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  await pcSearch(page).getByRole('button', { name: '＋ 期間' }).click();
  const dialog = page.getByRole('dialog', { name: '期間を選ぶ' });
  await dialog.getByRole('button', { name: '年', exact: true }).click();
  await dialog.getByRole('button', { name: '2024年', exact: true }).click();
  await dialog.getByRole('button', { name: 'この期間で絞る' }).click();

  await expect(dialog).toBeHidden();
  await expect(page.getByRole('button', { name: '期間 2024年 を解除' })).toBeVisible();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
  await expect(page.getByRole('table')).toContainText('架空自動車');
});

test('PC: 期間の小窓で「年月」を選んで絞り込める', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  await pcSearch(page).getByRole('button', { name: '＋ 期間' }).click();
  const dialog = page.getByRole('dialog', { name: '期間を選ぶ' });
  await dialog.getByRole('button', { name: '年月', exact: true }).click();
  await dialog.getByRole('combobox', { name: '年' }).selectOption('2023');
  await dialog.getByRole('combobox', { name: '月' }).selectOption('06');
  await dialog.getByRole('button', { name: 'この期間で絞る' }).click();

  await expect(page.getByRole('button', { name: '期間 2023年6月 を解除' })).toBeVisible();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
  await expect(page.getByRole('table')).toContainText('架空電信電話');
});

test('PC: 期間の小窓で「日」を選んで絞り込める', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  await pcSearch(page).getByRole('button', { name: '＋ 期間' }).click();
  const dialog = page.getByRole('dialog', { name: '期間を選ぶ' });
  await dialog.getByRole('button', { name: '日', exact: true }).click();
  await dialog.getByLabel('日付').fill('2024-03-01');
  await dialog.getByRole('button', { name: 'この期間で絞る' }).click();

  await expect(page.getByRole('button', { name: '期間 2024年3月1日 を解除' })).toBeVisible();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
});

test('PC: 期間の小窓で「範囲」を選んで絞り込める', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  await pcSearch(page).getByRole('button', { name: '＋ 期間' }).click();
  const dialog = page.getByRole('dialog', { name: '期間を選ぶ' });
  await dialog.getByRole('button', { name: '範囲', exact: true }).click();
  await dialog.getByLabel('開始日').fill('2023-01-01');
  await dialog.getByLabel('終了日').fill('2023-12-31');
  await dialog.getByRole('button', { name: 'この期間で絞る' }).click();

  await expect(
    page.getByRole('button', { name: '期間 2023年1月1日〜2023年12月31日 を解除' }),
  ).toBeVisible();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
});

test('PC: 期間の小窓で「すべて」に戻すと期間の条件が外れる', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  await pcSearch(page).getByRole('button', { name: '＋ 期間' }).click();
  const dialog = page.getByRole('dialog', { name: '期間を選ぶ' });
  await dialog.getByRole('button', { name: '年', exact: true }).click();
  await dialog.getByRole('button', { name: '2024年', exact: true }).click();
  await dialog.getByRole('button', { name: 'この期間で絞る' }).click();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');

  await pcSearch(page).getByRole('button', { name: '＋ 期間' }).click();
  const reopened = page.getByRole('dialog', { name: '期間を選ぶ' });
  await reopened.getByRole('button', { name: 'すべて', exact: true }).click();
  await reopened.getByRole('button', { name: 'この期間で絞る' }).click();

  await expect(page.getByTestId('tab-count-dividend')).toHaveText('2');
  await expect(page.getByRole('button', { name: /期間 .* を解除/ })).toHaveCount(0);
});

test('PC: Esc で小窓が閉じ、条件は反映されない', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  await pcSearch(page).getByRole('button', { name: '＋ 期間' }).click();
  const dialog = page.getByRole('dialog', { name: '期間を選ぶ' });
  await expect(dialog).toBeVisible();
  await page.keyboard.press('Escape');

  await expect(dialog).toBeHidden();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('2');
  await expect(page.getByRole('button', { name: /期間 .* を解除/ })).toHaveCount(0);
});

test('PC: 外側のクリックで小窓が閉じる', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  await pcSearch(page).getByRole('button', { name: '＋ 期間' }).click();
  await expect(page.getByRole('dialog', { name: '期間を選ぶ' })).toBeVisible();

  // 背景の面(小窓の外)をクリックすると閉じる
  await page.locator('.condition-backdrop').click({ position: { x: 320, y: 10 } });
  await expect(page.getByRole('dialog', { name: '期間を選ぶ' })).toBeHidden();
});

test('PC: 銘柄名の入力で絞り込める', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  await pcSearch(page).getByPlaceholder('銘柄名・コード').fill('架空電信電話');
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
  await expect(pcSearch(page)).toContainText('1件 / 2件');
  await expect(page.getByRole('table')).toContainText('架空電信電話');
});

test('PC: キーボードだけで条件を足して外せる', async ({ page }) => {
  await gotoReceipts(page, 1280, 800);

  const trigger = pcSearch(page).getByRole('button', { name: '＋ 口座' });
  await trigger.focus();
  await page.keyboard.press('Enter');
  const dialog = page.getByRole('dialog', { name: '口座を選ぶ' });
  // 開いたら小窓にフォーカスが移る
  await expect(dialog).toBeFocused();
  await page.keyboard.press('Tab');
  await page.keyboard.press('Enter');
  // 選ぶと小窓が閉じてトリガーにフォーカスが戻る
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');

  // 検索欄のタグまで戻って Enter で外せる
  await shiftTabTo(page, '口座 SBI証券 を解除');
  await page.keyboard.press('Enter');
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('2');
});

test('スマホ: 条件を足すと下から出るシートで期間を選べる', async ({ page }) => {
  await gotoReceipts(page, 390, 844);

  await page.getByRole('button', { name: '＋ 期間' }).click();
  const sheet = page.getByRole('dialog', { name: '期間を選ぶ' });
  await expect(sheet).toBeVisible();
  await sheet.getByRole('button', { name: '年月', exact: true }).click();
  await sheet.getByRole('combobox', { name: '年' }).selectOption('2024');
  await sheet.getByRole('combobox', { name: '月' }).selectOption('03');
  await sheet.getByRole('button', { name: 'この期間で絞る' }).click();

  await expect(sheet).toBeHidden();
  await expect(page.getByRole('button', { name: '期間 2024年3月 を解除' })).toBeVisible();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
  await expect(page.getByRole('search', { name: '検索', exact: true })).toContainText(
    '1件 / 2件',
  );
});

test('スマホ: 口座のシートで選んで絞り込める', async ({ page }) => {
  await gotoReceipts(page, 390, 844);

  await page.getByRole('button', { name: '＋ 口座' }).click();
  const sheet = page.getByRole('dialog', { name: '口座を選ぶ' });
  await sheet.getByRole('button', { name: '楽天証券', exact: true }).click();

  await expect(sheet).toBeHidden();
  await expect(page.getByRole('button', { name: '口座 楽天証券 を解除' })).toBeVisible();
  await expect(page.getByTestId('tab-count-dividend')).toHaveText('1');
  await expect(page.getByTestId('receipt-card')).toHaveCount(1);
});
