import type { Browser, Locator, Page } from '@playwright/test';
import { expect, test } from '../support/test';
import AxeBuilder from '@axe-core/playwright';
import * as path from 'path';

const MOCK_USER = { id: 1, email: 'test@example.com', name: 'テストユーザー' };

const ROUTES = {
  authMe: /\/api\/v1\/session$/,
  dividends: /\/api\/v1\/dividends(?:\?.*)?$/,
  domesticStocks: /\/api\/v1\/domestic-stock-transactions(?:\?.*)?$/,
  mutualfunds: /\/api\/v1\/mutual-fund-transactions(?:\?.*)?$/,
  assetBalances: /\/api\/v1\/asset-balances(?:\?.*)?$/,
  stock: /\/api\/v1\/stocks(?:\?.*)?$/,
  dividendPreview: /\/api\/v1\/dividend-import-validations$/,
  dividendUpload: /\/api\/v1\/dividend-imports$/,
  dividendPerShare: /\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/,
};

const CSV_FIXTURES = path.resolve(process.cwd(), 'e2e/__fixtures__/csv');

const EMPTY_PAGE = { data: [], total: 0, page: 1, per_page: 0 };

const DIVIDEND_ROW = {
  id: '00000000-0000-0000-0000-000000000001',
  settlement_date: '2024-03-01',
  product: '特定口座',
  account: 'SBI証券',
  security_code: '7203',
  security_name: 'トヨタ自動車',
  unit_price: '30.0',
  shares: '100',
  dividends_before_tax: '3000',
  taxes: '609',
  net_amount_received: '2391',
  created_at: '2024-03-01T00:00:00Z',
  updated_at: '2024-03-01T00:00:00Z',
};

const NEGATIVE_DOMESTIC_ROW = {
  id: '00000000-0000-0000-0000-000000000004',
  trade_date: '2024-02-01',
  settlement_date: '2024-02-03',
  account: 'SBI証券',
  security_code: '7974',
  security_name: '任天堂',
  shares: 10,
  asked_price: 5000,
  proceeds: 55000,
  purchase_price: 5000,
  realized_profit_and_loss: -8000,
  taxes: 0,
  realized_profit_and_loss_after_tax: -8000,
  created_at: '2024-02-01T00:00:00Z',
  updated_at: '2024-02-01T00:00:00Z',
};

const ASSET_ROW = {
  id: 'asset-balance-1',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
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

const MOCK_STOCK = {
  date: '2024-03-01',
  code: '7974',
  name: '任天堂',
  market_category: 'プライム',
};

const PC = { width: 1280, height: 800 };
const MOBILE = { width: 390, height: 844 };

function paginated(data: unknown[]) {
  return { data, total: data.length, page: 1, per_page: data.length };
}

function json(body: unknown, status = 200) {
  return { status, contentType: 'application/json', body: JSON.stringify(body) };
}

async function liveRegionTexts(page: Page): Promise<string[]> {
  return page.evaluate(() =>
    Array.from(document.querySelectorAll('[role="status"], [role="alert"], [aria-live]'))
      .map((el) => ((el as HTMLElement).innerText ?? '').trim())
      .filter((text) => text.length > 0),
  );
}

// データがあると右レールは畳まれた状態で始まるので、レール内の UI に触れる前に開く。
// トグルはタブのデータ到着(Ready)まで描画されないため、出現を待ってから押す
async function openReceiptRail(page: Page) {
  const rail = page.getByTestId('receipt-utility-rail');
  if (await rail.isVisible()) {
    return;
  }
  const toggle = page.getByTestId('receipt-utility-toggle');
  await toggle.waitFor({ state: 'visible', timeout: 10000 });
  await toggle.click();
  await expect(rail).toBeVisible();
}

async function mockSession(page: Page, user: unknown = MOCK_USER, status = 200) {
  await page.route(ROUTES.authMe, (route) =>
    route.fulfill(json(user, status)),
  );
}

async function mockEmptyLists(page: Page) {
  await page.route(ROUTES.dividends, (route) => route.fulfill(json(EMPTY_PAGE)));
  await page.route(ROUTES.domesticStocks, (route) => route.fulfill(json(EMPTY_PAGE)));
  await page.route(ROUTES.mutualfunds, (route) => route.fulfill(json(EMPTY_PAGE)));
  await page.route(ROUTES.assetBalances, (route) => route.fulfill(json(EMPTY_PAGE)));
  await page.route(ROUTES.stock, (route) => route.fulfill(json([])));
  await page.route(ROUTES.dividendPerShare, (route) => route.fulfill(json({ items: [] })));
}

async function mockReceiptsData(page: Page) {
  await page.route(ROUTES.dividends, (route) =>
    route.fulfill(json(paginated([DIVIDEND_ROW]))),
  );
  await page.route(ROUTES.domesticStocks, (route) =>
    route.fulfill(json(paginated([NEGATIVE_DOMESTIC_ROW]))),
  );
  await page.route(ROUTES.mutualfunds, (route) => route.fulfill(json(EMPTY_PAGE)));
}

async function mockAssetData(page: Page) {
  await page.route(ROUTES.assetBalances, (route) =>
    route.fulfill(json(paginated([ASSET_ROW]))),
  );
  await page.route(ROUTES.dividendPerShare, (route) =>
    route.fulfill(
      json({
        items: [
          {
            security_code: '7203',
            dividend_per_share: 90,
            status: 'ok',
            fetched_at: '2026-01-01T00:00:00Z',
            is_stale: false,
          },
        ],
      }),
    ),
  );
}

interface AxeExclusion {
  issue: number;
  rule: string;
  target: string;
  labelPrefix: string;
}

const AXE_EXCLUSIONS: AxeExclusion[] = [];

const AXE_TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];

async function scanAxe(page: Page, label: string) {
  const results = await new AxeBuilder({ page }).withTags(AXE_TAGS).analyze();
  const applied: string[] = [];
  const remaining = results.violations
    .filter((v) => v.impact === 'critical' || v.impact === 'serious' || v.impact === 'moderate')
    .map((v) => {
      const nodes = v.nodes.filter((n) => {
        const target = (n.target ?? []).join(' ');
        const html = n.html ?? '';
        const hit = AXE_EXCLUSIONS.find(
          (e) =>
            e.rule === v.id &&
            label.startsWith(e.labelPrefix) &&
            (target.includes(e.target) || html.includes(e.target)),
        );
        if (hit) {
          applied.push(`#${hit.issue} ${v.id} ${target}`);
          return false;
        }
        return true;
      });
      return { ...v, nodes };
    })
    .filter((v) => v.nodes.length > 0);
  const minor = results.violations.filter((v) => v.impact === 'minor');
  if (minor.length > 0) {
    console.log(
      `[minor] ${label}: ` +
        minor.map((v) => `${v.id}(${(v.nodes ?? []).length})`).join(', '),
    );
  }
  for (const a of applied) {
    console.log(`[除外] ${label}: ${a}`);
  }
  expect(
    remaining,
    `[${label}] ` +
      remaining
        .map(
          (v) =>
            `[${v.impact}] ${v.id}: ${v.nodes.map((n) => (n.target ?? []).join(' ')).join(' / ')}`,
        )
        .join('\n'),
  ).toHaveLength(0);
}

async function scanBothViewports(
  page: Page,
  label: string,
  go: () => Promise<void>,
  waitNetworkIdle = true,
  assertState?: () => Promise<void>,
) {
  for (const [name, size] of [['PC', PC], ['mobile', MOBILE]] as const) {
    await page.setViewportSize(size);
    await go();
    if (waitNetworkIdle) {
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(400);
    } else {
      await page.waitForTimeout(900);
    }
    if (assertState) {
      await assertState();
    }
    await scanAxe(page, `${label}(${name})`);
  }
  await page.setViewportSize(PC);
}

test('ホームに moderate 以上の WCAG 違反がない', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await scanBothViewports(page, 'ホーム', () => page.goto('/'));
});

test('銘柄検索に moderate 以上の WCAG 違反がない', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await page.route(ROUTES.stock, (route) => route.fulfill(json(MOCK_STOCK)));
  await scanBothViewports(page, '銘柄検索(空)', () => page.goto('/search'));
  await page.setViewportSize(PC);
  await page.goto('/search');
  await page.waitForLoadState('networkidle');
  await page.getByRole('textbox', { name: '銘柄コードまたは銘柄名' }).fill('7974');
  await page.keyboard.press('Enter');
  await expect(page.getByText('任天堂').first()).toBeVisible({ timeout: 10000 });
  await scanAxe(page, '銘柄検索(結果あり,PC)');
  await page.setViewportSize(MOBILE);
  await page.waitForTimeout(400);
  await scanAxe(page, '銘柄検索(結果あり,mobile)');
  await page.setViewportSize(PC);
});

test('資産管理に moderate 以上の WCAG 違反がない', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await scanBothViewports(
    page,
    '資産管理(空)',
    () => page.goto('/assetbalance'),
    true,
    () =>
      expect(page.getByText('資産管理データがありません').first()).toBeVisible({
        timeout: 10000,
      }),
  );
  await mockAssetData(page);
  await scanBothViewports(
    page,
    '資産管理(データあり)',
    () => page.goto('/assetbalance'),
    true,
    () => expect(page.getByTestId('asset-portfolio-summary')).toBeVisible({ timeout: 10000 }),
  );
});

test('資産管理の取得失敗と読み込み中に moderate 以上の WCAG 違反がない', async ({
  page,
}) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await page.route(ROUTES.assetBalances, (route) =>
    route.fulfill(json({ error: '取得に失敗しました' }, 500)),
  );
  await scanBothViewports(
    page,
    '資産管理(取得失敗)',
    () => page.goto('/assetbalance'),
    true,
    () => expect(page.getByTestId('list-load-error')).toBeVisible({ timeout: 10000 }),
  );
  await page.unroute(ROUTES.assetBalances);
  await page.route(ROUTES.assetBalances, () => {});
  await scanBothViewports(
    page,
    '資産管理(読み込み中)',
    () => page.goto('/assetbalance'),
    false,
    () => expect(page.getByTestId('list-skeleton')).toBeVisible({ timeout: 10000 }),
  );
});

test('取引明細の3タブに moderate 以上の WCAG 違反がない', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await mockReceiptsData(page);
  const tabs = [
    { label: '配当金', id: 'tab-dividend', visible: DIVIDEND_ROW.security_name },
    { label: '国内株式', id: 'tab-domesticstock', visible: NEGATIVE_DOMESTIC_ROW.security_name },
    { label: '投資信託', id: 'tab-mutualfund', visible: 'データがありません' },
  ];
  for (const [name, size] of [['PC', PC], ['mobile', MOBILE]] as const) {
    await page.setViewportSize(size);
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');
    for (const tab of tabs) {
      await page.click(`button[role="tab"][id="${tab.id}"]`);
      await expect(page.locator(`#${tab.id}`)).toHaveAttribute('aria-selected', 'true');
      await expect(
        page
          .locator(`#tabpanel-${tab.id.replace('tab-', '')}`)
          .getByText(tab.visible)
          .filter({ visible: true })
          .first(),
      ).toBeVisible({ timeout: 10000 });
      await scanAxe(page, `取引明細${tab.label}(${name})`);
    }
  }
  await page.setViewportSize(PC);
});

test('取引明細の取得失敗と読み込み中に moderate 以上の WCAG 違反がない', async ({
  page,
}) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await page.route(ROUTES.dividends, (route) =>
    route.fulfill(json({ error: '取得に失敗しました' }, 500)),
  );
  await scanBothViewports(
    page,
    '取引明細(取得失敗)',
    () => page.goto('/receipts'),
    true,
    () => expect(page.getByTestId('list-load-error')).toBeVisible({ timeout: 10000 }),
  );
  await page.unroute(ROUTES.dividends);
  await page.route(ROUTES.dividends, () => {});
  await scanBothViewports(
    page,
    '取引明細(読み込み中)',
    () => page.goto('/receipts'),
    false,
    () => expect(page.getByTestId('list-skeleton')).toBeVisible({ timeout: 10000 }),
  );
});

test('CSVプレビューと確認モーダルに moderate 以上の WCAG 違反がない', async ({
  page,
}) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await mockReceiptsData(page);
  await page.route(ROUTES.dividendPreview, (route) =>
    route.fulfill(json({ total_rows: 1, valid_rows: 1, errors: [], rows: [] })),
  );
  for (const [name, size] of [['PC', PC], ['mobile', MOBILE]] as const) {
    await page.setViewportSize(size);
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');
    await openReceiptRail(page);
    const fileInput = page.locator('[data-testid="csv-file-input"]');
    await expect(fileInput).toBeEnabled({ timeout: 10000 });
    await fileInput.setInputFiles(path.join(CSV_FIXTURES, 'dividend-base.csv'));
    await expect(
      page.locator('[role="status"][aria-live="polite"]').first(),
    ).toContainText('追加で保存されます', { timeout: 10000 });
    await scanAxe(page, `CSVプレビュー(${name})`);
    const railToggle = page.locator('[data-testid="receipt-csv-toggle"]');
    if ((await railToggle.count()) > 0 && (await railToggle.isVisible())) {
      const delVisible = await page
        .getByRole('button', { name: /全件削除/ })
        .first()
        .isVisible()
        .catch(() => false);
      if (!delVisible) {
        await railToggle.click();
      }
    }
    await page.getByRole('button', { name: /全件削除/ }).first().click();
    await expect(page.getByRole('dialog')).toBeVisible({ timeout: 5000 });
    await scanAxe(page, `確認モーダル(${name})`);
    await page.keyboard.press('Escape');
    await expect(page.getByRole('dialog')).toHaveCount(0);
  }
  await page.setViewportSize(PC);
});

test('CSVプレビューの行エラー表示に moderate 以上の WCAG 違反がない', async ({
  page,
}) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await mockReceiptsData(page);
  await page.route(ROUTES.dividendPreview, (route) =>
    route.fulfill(
      json({
        total_rows: 3,
        valid_rows: 1,
        errors: [
          { row: 2, message: '受取金額が数値ではありません' },
          { row: 3, message: '受取日の形式が不正です' },
        ],
        rows: [],
      }),
    ),
  );
  await scanBothViewports(page, 'CSVプレビュー(行エラー)', async () => {
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');
    const fileInput = page.locator('[data-testid="csv-file-input"]');
    await expect(fileInput).toBeEnabled({ timeout: 10000 });
    await fileInput.setInputFiles(path.join(CSV_FIXTURES, 'dividend-base.csv'));
    await expect(
      page.locator('[role="status"][aria-live="polite"]').first(),
    ).toContainText('件エラー', { timeout: 10000 });
  });
});

test('フィルター展開時に moderate 以上の WCAG 違反がない', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await mockReceiptsData(page);
  await mockAssetData(page);
  for (const [name, size] of [['PC', PC], ['mobile', MOBILE]] as const) {
    await page.setViewportSize(size);
    await page.goto('/receipts');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(400);
await openReceiptRail(page);
    const header = page.locator('[data-testid="search-card-header"]:visible, [data-testid="receipt-search-toggle"]:visible').first();
    if ((await header.getAttribute('aria-expanded')) === 'false') {
      await header.click();
      await page.waitForTimeout(300);
    }
    await expect(header).toHaveAttribute('aria-expanded', 'true');
    await scanAxe(page, `フィルター展開(${name})`);
    await page.goto('/assetbalance');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(400);
    // 検索カードは行がある時だけ描画される。空・取得失敗だと header が無くスキャンが黙って飛ばされる
    const assetHeader = page.locator('[data-testid="search-card-header"]:visible, [data-testid="receipt-search-toggle"]:visible').first();
    await expect(assetHeader).toBeVisible({ timeout: 10000 });
    if ((await assetHeader.getAttribute('aria-expanded')) === 'false') {
      await assetHeader.click();
      await page.waitForTimeout(300);
    }
    await expect(assetHeader).toHaveAttribute('aria-expanded', 'true');
    await scanAxe(page, `資産フィルター展開(${name})`);
  }
  await page.setViewportSize(PC);
});

test('ログインと404に moderate 以上の WCAG 違反がない', async ({ page }) => {
  await page.unrouteAll({ behavior: 'wait' });
  await page.route(ROUTES.authMe, (route) => route.fulfill(json({}, 401)));
  await scanBothViewports(page, 'ログイン', () => page.goto('/login'), true, () =>
    expect(page.getByRole('button', { name: /Googleでログイン/ })).toBeVisible({ timeout: 10000 }),
  );
  await mockSession(page);
  await scanBothViewports(page, '404', () => page.goto('/no-such-page-xyz'), true, () =>
    expect(page.getByRole('heading', { name: '404 - ページが見つかりません' })).toBeVisible({
      timeout: 10000,
    }),
  );
});

// .focus() は tabindex=-1 や順序外にも当たるため、Tab キーだけで到達できることを固定する
async function tabTo(page: Page, target: Locator, maxPresses = 250): Promise<void> {
  await expect(target).toBeVisible();
  const reached = () =>
    target
      .evaluate((el) => el === document.activeElement)
      .catch(() => false);
  for (let pressed = 0; pressed < maxPresses; pressed += 1) {
    if (await reached()) {
      return;
    }
    await page.keyboard.press('Tab');
  }
  expect(await reached(), `${maxPresses}回 Tab しても ${target} に届かなかった`).toBe(true);
}

test('Tab だけで主要な操作ができる', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await mockReceiptsData(page);
  await page.route(ROUTES.dividendPreview, (route) =>
    route.fulfill(json({ total_rows: 1, valid_rows: 1, errors: [], rows: [] })),
  );
  await page.setViewportSize(PC);
  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');
  await openReceiptRail(page);

  const dividendTab = page.locator('button[role="tab"][id="tab-dividend"]');
  const domesticTab = page.locator('button[role="tab"][id="tab-domesticstock"]');
  await tabTo(page, dividendTab);
  await expect(dividendTab).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(dividendTab).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('ArrowRight');
  await expect(domesticTab).toBeFocused();
  await expect(domesticTab).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('ArrowLeft');
  await expect(dividendTab).toBeFocused();
  await expect(dividendTab).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('Space');
  await expect(dividendTab).toHaveAttribute('aria-selected', 'true');

  const header = page.locator('[data-testid="search-card-header"]:visible, [data-testid="receipt-search-toggle"]:visible').first();
  await tabTo(page, header);
  await expect(header).toBeFocused();
  const before = await header.getAttribute('aria-expanded');
  await page.keyboard.press('Enter');
  await expect(header).toHaveAttribute(
    'aria-expanded',
    before === 'true' ? 'false' : 'true',
  );
  await page.keyboard.press('Space');
  await expect(header).toHaveAttribute('aria-expanded', before);

  const fileInput = page.locator('[data-testid="csv-file-input"]');
  await expect(fileInput).toBeEnabled({ timeout: 10000 });
  await tabTo(page, fileInput);
  await expect(fileInput).toBeFocused();
  const labelOutline = await page
    .locator('[data-testid="csv-file-trigger"]')
    .evaluate((el) => {
      const style = getComputedStyle(el);
      return `${style.outlineStyle}/${style.outlineWidth}`;
    });
  expect(labelOutline, 'CSVの入力にフォーカスしても表示ラベルに枠が出ない').not.toMatch(
    /^none\/(0px|none)$/,
  );
  const [chooser] = await Promise.all([
    page.waitForEvent('filechooser'),
    page.keyboard.press('Enter'),
  ]);
  await chooser.setFiles(path.join(CSV_FIXTURES, 'dividend-base.csv'));
  await expect(
    page.locator('[role="status"][aria-live="polite"]').first(),
  ).toContainText('追加で保存されます', { timeout: 10000 });

  const deleteButton = page.getByRole('button', { name: /全件削除/ }).first();
  await tabTo(page, deleteButton);
  await expect(deleteButton).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('dialog')).toBeVisible({ timeout: 5000 });
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toHaveCount(0);
});

test('フォーカスが見えてスキップリンクが使える', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await page.setViewportSize(PC);
  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');
  await expect(page.getByRole('tab')).toHaveCount(3, { timeout: 10000 });
  await expect(page.getByRole('button', { name: 'メニュー' })).toBeVisible({ timeout: 10000 });
  const skip = page.locator('.skip-link');
  await skip.focus();
  await expect(skip).toBeFocused();
  const skipBox = await skip.boundingBox();
  expect(skipBox).not.toBeNull();
  expect(skipBox!.height).toBeGreaterThan(24);
  await page.keyboard.press('Tab');
  await expect(page.locator('header a[href="/"]').first()).toBeFocused();
  await page.keyboard.press('Shift+Tab');
  await expect(skip).toBeFocused();

  await page.click('button[role="tab"][id="tab-dividend"]');
  await page.keyboard.press('Tab');
  const outline = await page.evaluate(() => {
    const el = document.activeElement;
    if (!el) {
      return 'none';
    }
    const style = getComputedStyle(el);
    return `${style.outlineStyle}/${style.outlineWidth}/${style.boxShadow.slice(0, 20)}`;
  });
  expect(outline).not.toMatch(/^none\/0px\/none/);
});

interface TargetOffender {
  tag: string;
  name: string;
  testid: string;
  cls: string;
  w: number;
  h: number;
}

interface TargetExclusion {
  issue: number;
  mobileOnly: boolean;
  match: (t: TargetOffender) => boolean;
}

const TARGET_EXCLUSIONS: TargetExclusion[] = [];

function targetExcluded(t: TargetOffender, mobile: boolean): number | null {
  const hit = TARGET_EXCLUSIONS.find((e) => (mobile || !e.mobileOnly) && e.match(t));
  return hit ? hit.issue : null;
}

async function collectSmallTargets(page: Page, min: number): Promise<TargetOffender[]> {
  return page.evaluate((threshold: number) => {
    const list: TargetOffender[] = [];
    const els = document.querySelectorAll(
      'button, a[href], input, select, textarea, [role="tab"]',
    );
    for (const el of Array.from(els)) {
      const h = el as HTMLElement;
      if (h.hasAttribute('disabled')) {
        continue;
      }
      if (h.getAttribute('aria-hidden') === 'true') {
        continue;
      }
      // 非選択のタブは roving tabindex で -1 だが、クリックで選べるので測る
      if (h.getAttribute('tabindex') === '-1' && h.getAttribute('role') !== 'tab') {
        continue;
      }
      const inputType = h.getAttribute('type');
      if (inputType === 'hidden') {
        continue;
      }
      const style = getComputedStyle(h);
      if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') {
        continue;
      }
      let rect = h.getBoundingClientRect();
      if (rect.width < 2 && rect.height < 2) {
        const label = h instanceof HTMLInputElement ? h.labels?.[0] : undefined;
        if (!label) {
          continue;
        }
        rect = label.getBoundingClientRect();
        if (rect.width < 2 && rect.height < 2) {
          continue;
        }
      }
      if (rect.width < threshold || rect.height < threshold) {
        const cls = h.className && typeof h.className === 'string' ? h.className : '';
        list.push({
          tag: h.tagName,
          name: (h.getAttribute('aria-label') ?? (h as HTMLElement).innerText ?? '').trim().slice(0, 32),
          testid: h.dataset.testid ?? '',
          cls: cls.slice(0, 80),
          w: Math.round(rect.width),
          h: Math.round(rect.height),
        });
      }
    }
    return list;
  }, min);
}

async function assertTargetSize(
  page: Page,
  size: { width: number; height: number },
  min: number,
) {
  await page.setViewportSize(size);
  const mobile = min >= 44;
  const routes: Array<[string, () => Promise<void>]> = [
    ['ホーム', () => page.goto('/')],
    ['銘柄検索', () => page.goto('/search')],
    [
      '資産管理',
      async () => {
        await page.goto('/assetbalance');
        await expect(page.getByTestId('asset-portfolio-summary')).toBeVisible({ timeout: 10000 });
      },
    ],
    [
      '取引明細(配当金)',
      async () => {
        await page.goto('/receipts');
        await expect(
          page.getByText(DIVIDEND_ROW.security_name).filter({ visible: true }).first(),
        ).toBeVisible({ timeout: 10000 });
      },
    ],
    [
      '取引明細(国内株式)',
      async () => {
        await page.click('button[role="tab"][id="tab-domesticstock"]');
        await expect(page.locator('#tab-domesticstock')).toHaveAttribute('aria-selected', 'true');
        await expect(
          page.getByText(NEGATIVE_DOMESTIC_ROW.security_name).filter({ visible: true }).first(),
        ).toBeVisible({ timeout: 10000 });
      },
    ],
  ];
  for (const [label, go] of routes) {
    await go();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(500);
    const small = await collectSmallTargets(page, min);
    const remaining = small.filter((t) => targetExcluded(t, mobile) === null);
    for (const t of small) {
      const issue = targetExcluded(t, mobile);
      if (issue !== null) {
        console.log(`[除外] ${label}: #${issue} ${t.tag} "${t.name}" ${t.w}x${t.h}`);
      }
    }
    expect(
      remaining,
      `[${label}] ` +
        remaining.map((t) => `${t.tag} "${t.name}" ${t.w}x${t.h} (${t.testid})`).join('\n'),
    ).toHaveLength(0);
  }
  await page.setViewportSize(PC);
}

test('スマホで操作要素が44px以上ある(除外はIssue付き)', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await mockReceiptsData(page);
  await mockAssetData(page);
  await page.route(ROUTES.stock, (route) => route.fulfill(json(MOCK_STOCK)));
  await assertTargetSize(page, MOBILE, 44);
});

test('PC幅で操作要素が24px以上ある(除外はIssue付き)', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await mockReceiptsData(page);
  await mockAssetData(page);
  await page.route(ROUTES.stock, (route) => route.fulfill(json(MOCK_STOCK)));
  await assertTargetSize(page, PC, 24);
});

test('320px幅で横スクロールが出ない', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await mockReceiptsData(page);
  await page.setViewportSize({ width: 320, height: 800 });
  for (const url of ['/', '/search', '/assetbalance', '/receipts']) {
    await page.goto(url);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(300);
    const width = await page.evaluate(() => document.scrollingElement?.scrollWidth ?? 0);
    expect(width, url).toBeLessThanOrEqual(320);
  }
  await page.setViewportSize(PC);
});

test('CSVの保存結果が支援技術に伝わる', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await page.route(ROUTES.dividendPreview, (route) =>
    route.fulfill(json({ total_rows: 1, valid_rows: 1, errors: [], rows: [] })),
  );
  let upload: unknown = { inserted: 1, skipped: 0, errors: [] };
  await page.route(ROUTES.dividendUpload, (route) => route.fulfill(json(upload)));
  await page.setViewportSize(PC);
  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');
  await openReceiptRail(page);
  const fileInput = page.locator('[data-testid="csv-file-input"]');
  const preview = page.locator('[role="status"][aria-live="polite"]').first();

  for (const failure of [false, true]) {
    upload = failure
      ? { inserted: 0, skipped: 0, errors: [{ row: 2, message: '受取金額が数値ではありません' }] }
      : { inserted: 1, skipped: 0, errors: [] };
    // 反映0件の全失敗と成功は別の見出しで読み上げられる
    const heading = failure ? '保存できませんでした' : '保存しました';
    await expect(fileInput).toBeEnabled({ timeout: 10000 });
    await fileInput.setInputFiles(path.join(CSV_FIXTURES, 'dividend-base.csv'));
    await expect(preview).toContainText('追加で保存されます', { timeout: 10000 });
    const before = await liveRegionTexts(page);
    await page.getByRole('button', { name: /追加で保存/ }).first().click();
    const notice = page.getByTestId('csv-save-result-notice');
    await expect(notice).toBeVisible({ timeout: 10000 });
    await expect(notice).toContainText(heading);
    if (failure) {
      await expect(notice).toContainText('1件エラー');
    } else {
      await expect(notice).toContainText('1件反映');
    }
    const after = await liveRegionTexts(page);
    expect(after, 'ライブリージョンの内容が保存で変わらなかった').not.toEqual(before);
    expect(
      after.filter((text) => text.includes(heading)),
      '保存結果がライブリージョンに伝わらなかった',
    ).not.toHaveLength(0);
  }
});

test('読み込み中が支援技術に伝わる', async ({ page }) => {
  await mockSession(page);
  await page.route(ROUTES.dividends, () => {});
  await page.route(ROUTES.domesticStocks, (route) => route.fulfill(json(EMPTY_PAGE)));
  await page.route(ROUTES.mutualfunds, (route) => route.fulfill(json(EMPTY_PAGE)));
  await page.setViewportSize(PC);
  await page.goto('/receipts');
  const loading = page
    .getByTestId('receipts-workspace')
    .getByRole('status')
    .filter({ hasText: /読み込んでいます|確認しています/ });
  await expect(loading.first()).toBeVisible({ timeout: 10000 });
  await expect(page.locator('[data-testid="receipts-workspace"]').locator('..')).toHaveAttribute(
    'aria-busy',
    'true',
  );
  // 読み込み中は選択タブのパネルだけだと非選択タブの aria-controls が解決しない
  for (const id of ['dividend', 'domesticstock', 'mutualfund']) {
    const controls = await page
      .locator(`button[role="tab"][id="tab-${id}"]`)
      .getAttribute('aria-controls');
    await expect(page.locator(`#${controls}`)).toHaveCount(1);
  }
});

test('取得失敗が支援技術に伝わる', async ({ page }) => {
  await mockSession(page);
  await page.route(ROUTES.dividends, (route) =>
    route.fulfill(json({ error: '取得に失敗しました' }, 500)),
  );
  await page.route(ROUTES.domesticStocks, (route) => route.fulfill(json(EMPTY_PAGE)));
  await page.route(ROUTES.mutualfunds, (route) => route.fulfill(json(EMPTY_PAGE)));
  await page.setViewportSize(PC);
  await page.goto('/receipts');
  await expect(page.getByRole('alert').filter({ hasText: /エラー/ }).first()).toBeVisible({
    timeout: 10000,
  });
  await expect(page.getByRole('button', { name: '再読み込み' })).toBeVisible();
});

test('マイナスは符号と色の両方で表す', async ({ page }) => {
  await mockSession(page);
  await mockEmptyLists(page);
  await mockReceiptsData(page);
  await page.setViewportSize(PC);
  await page.goto('/receipts');
  await page.waitForLoadState('networkidle');
  await page.click('button[role="tab"][id="tab-domesticstock"]');
  await page.waitForTimeout(500);
  const negative = await page.evaluate(() => {
    const token = getComputedStyle(document.documentElement)
      .getPropertyValue('--color-negative')
      .trim();
    const probe = document.createElement('span');
    probe.style.color = token;
    document.body.append(probe);
    const negativeColor = getComputedStyle(probe).color;
    probe.remove();
    return {
      negativeColor,
      cells: Array.from(document.querySelectorAll('[data-negative="true"]')).map((el) => ({
        text: (el.textContent ?? '').trim().slice(0, 40),
        color: getComputedStyle(el).color,
      })),
    };
  });
  expect(negative.negativeColor).not.toBe('');
  expect(negative.cells.length).toBeGreaterThan(0);
  for (const cell of negative.cells) {
    const amount = cell.text.replace(/^[\s¥￥$€£]+/, '');
    expect(amount, `符号なしのマイナス表示: ${cell.text}`).toMatch(/^[-−]|マイナス/);
    expect(cell.color, `色だけのマイナス表示: ${cell.text}`).toBe(negative.negativeColor);
  }
});

test('prefers-reduced-motion で動きを止める', async ({ browser }: { browser: Browser }) => {
  const context = await browser.newContext({ reducedMotion: 'reduce' });
  const page = await context.newPage();
  await mockSession(page);
  await mockEmptyLists(page);
  await page.route(ROUTES.dividends, () => {});
  await page.setViewportSize(PC);
  await page.goto('/receipts');
  await page.waitForTimeout(800);
  const animation = await page.evaluate(() => {
    const el = document.querySelector('.animate-pulse, .animate-spin');
    if (!el) {
      return 'none found';
    }
    const style = getComputedStyle(el);
    return `${style.animationName}/${style.animationDuration}`;
  });
  expect(animation).toMatch(/^none\//);
  await context.close();
});
