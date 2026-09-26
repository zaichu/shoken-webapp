import { expect, test, type Page } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000010',
  email: 'test@example.com',
  name: 'テストユーザー',
};

const TOYOTA = {
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

const SONY = {
  id: 'asset-balance-2',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
  security_code: '6758',
  security_name: 'ソニーグループ',
  shares: 50,
  executing_shares: 0,
  average_purchase_price: 12000,
  total_purchase_amount: 600000,
  current_price: 11500,
  daily_change: -100,
  market_value: 575000,
  profit_loss_rate: -4.17,
};

const FACETS = {
  securities: [
    { value: '7203', label: 'トヨタ自動車', count: 1 },
    { value: '6758', label: 'ソニーグループ', count: 1 },
  ],
};

const EXPECTED_PROMPT = `あなたは日本株の公開情報を整理する調査サポーターです。
以下の保有銘柄データをもとに、保有株の構成レビュー（ポートフォリオ総評の下書き）を作成してください。

【重要】
- このプロンプトによる回答は金融商品取引業者・投資顧問としての助言ではなく、売買指示・目標株価・断定的推奨を行わないでください
- 最新情報を確認できない場合は「確認不能」と明記し、提供された保有構成から分かる範囲だけで整理してください
- 最新情報の取得・判断はユーザー自身が行う前提としてください
- このアプリから渡す保有データだけで含み損益や時価評価を判断しないでください
- 確認できない情報は推測せず「不明」または「要確認」と明記してください
- 銘柄コードが曖昧な場合は、市場・上場銘柄の確認から始めてください
- 投資目的、投資期間、リスク許容度、流動性需要、税務状況、他の資産状況が不足しているため、個別判断が必要な場合は追加質問として列挙してください

【作成してほしい内容】
1. ポートフォリオ総評（保有構成の概要・特徴）
2. 構成上のリスク視点（集中リスク・業種偏り・銘柄偏りの読み取り）
3. 各銘柄について確認を推奨する公開情報の観点（決算・IR・業績・指標など）
4. 追加で確認すべきニュース・開示・リスク要因
5. 個別判断に必要な追加質問リスト

【保有銘柄データ】
銘柄コード | 銘柄名 | 保有株数 | 平均取得単価
7203 | トヨタ自動車 | 100 | ¥2,500
6758 | ソニーグループ | 50 | ¥12,000`;

function jsonResponse(body: unknown, status = 200) {
  return {
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  };
}

async function stubClipboard(
  page: Page,
  mode: 'ok' | 'reject' | 'missing' = 'ok',
) {
  await page.addInitScript((stubMode) => {
    const writes: string[] = [];
    Object.defineProperty(window, '__clipboardWrites', {
      value: writes,
      configurable: true,
    });
    const clipboard =
      stubMode === 'missing'
        ? undefined
        : {
            writeText: (text: string) => {
              writes.push(text);
              return stubMode === 'reject'
                ? Promise.reject(new Error('denied'))
                : Promise.resolve();
            },
          };
    Object.defineProperty(window.navigator, 'clipboard', {
      value: clipboard,
      configurable: true,
    });
  }, mode);
}

async function setupAssetBalanceMocks(page: Page, rows: unknown[] = [TOYOTA, SONY]) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill(jsonResponse(MOCK_USER)),
  );

  await page.route(/\/api\/v1\/asset-balances(?:\?.*)?$/, (route) => {
    if (route.request().method() !== 'GET') {
      return route.fallback();
    }
    return route.fulfill(
      jsonResponse({
        data: rows,
        total: rows.length,
        page: 1,
        per_page: 1000,
        facets: FACETS,
      }),
    );
  });

  await page.route(/\/api\/v1\/dividend-per-share-estimates(?:\?.*)?$/, (route) =>
    route.fulfill(jsonResponse({ items: [] })),
  );
}

async function gotoAssetBalance(page: Page) {
  await page.goto('/assetbalance');
  await expect(page.getByTestId('assetbalance-utility-rail')).toBeVisible();
}

async function shoot(page: Page, name: string) {
  const dir = path.resolve(test.info().project.testDir, '../../.playwright-mcp');
  await fs.promises.mkdir(dir, { recursive: true });
  await page.screenshot({
    path: path.join(dir, `leptos-assetbalance-review-${name}.png`),
    fullPage: true,
  });
}

test('1920px では見直し促進カードがレール内に表示され、クリップボードへ見直し用のプロンプトをコピーする', async ({
  page,
}) => {
  await stubClipboard(page);
  await setupAssetBalanceMocks(page);
  await page.setViewportSize({ width: 1920, height: 1080 });
  await gotoAssetBalance(page);
  await expect(page.getByTestId('portfolio-pie-chart')).toBeVisible();

  const rail = page.getByTestId('assetbalance-utility-rail');
  const card = rail.getByTestId('asset-review-prompt-card');
  await expect(card).toBeVisible();
  await expect(
    card.getByText('AI総評プロンプト', { exact: true }),
  ).toBeVisible();

  const button = card.getByRole('button', {
    name: 'AI総評プロンプトをコピー',
  });
  await expect(button).toBeEnabled();

  // レール内の順序は CSV → 検索 → 見直し促進カード(React と同じ末尾)
  const order = await rail
    .locator('> div')
    .first()
    .evaluate((el) =>
      Array.from(el.children)
        .map((child) => (child as HTMLElement).dataset?.testid ?? '')
        .filter((id) => id.length > 0),
    );
  expect(order[order.length - 1]).toBe('asset-review-prompt-card');

  await button.click();
  await expect(
    card.getByRole('button', { name: 'コピーしました！' }),
  ).toBeVisible();

  const written = await page.evaluate(
    () => (window as unknown as { __clipboardWrites: string[] }).__clipboardWrites,
  );
  expect(written).toHaveLength(1);
  expect(written[0]).toBe(EXPECTED_PROMPT);

  // 3秒でラベルが元に戻る
  await expect(
    card.getByRole('button', { name: 'AI総評プロンプトをコピー' }),
  ).toBeVisible({ timeout: 6_000 });

  await shoot(page, '1920');
});

test('390px でも見直し促進カードはレール1枚カード内の末尾セクションになる', async ({
  page,
}) => {
  await stubClipboard(page);
  await setupAssetBalanceMocks(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await gotoAssetBalance(page);
  await expect(page.getByTestId('portfolio-valuation-card').first()).toBeVisible();

  const rail = page.getByTestId('assetbalance-utility-rail');
  const outerCard = rail.locator('> div').first();
  const card = rail.getByTestId('asset-review-prompt-card');
  await expect(card).toBeVisible();
  const cardStyle = await card.evaluate((el) => {
    const s = getComputedStyle(el);
    return {
      borderTopWidth: s.borderTopWidth,
      borderRadius: s.borderTopLeftRadius,
    };
  });
  expect(cardStyle.borderTopWidth).toBe('0px');
  expect(Number.parseFloat(cardStyle.borderRadius)).toBe(0);

  const order = await outerCard.evaluate((el) =>
    Array.from(el.children)
      .map((child) => (child as HTMLElement).dataset?.testid ?? '')
      .filter((id) => id.length > 0),
  );
  expect(order[order.length - 1]).toBe('asset-review-prompt-card');

  const button = card.getByRole('button', {
    name: 'AI総評プロンプトをコピー',
  });
  await expect(button).toBeEnabled();
  const buttonBox = await button.boundingBox();
  expect(buttonBox).not.toBeNull();
  expect(buttonBox!.height).toBeGreaterThanOrEqual(44);

  await shoot(page, '390');
});

test('連続してコピーしても後のクリックの表示が先のタイマーで消えない', async ({
  page,
}) => {
  await stubClipboard(page);
  await setupAssetBalanceMocks(page);
  await page.setViewportSize({ width: 1920, height: 1080 });
  await gotoAssetBalance(page);
  await expect(page.getByTestId('portfolio-pie-chart')).toBeVisible();

  const card = page.getByTestId('asset-review-prompt-card');
  const button = card.getByRole('button');
  await button.click();
  await page.waitForTimeout(2_200);
  await button.click();
  // 1回目のクリックから3秒経っても、2回目のクリックの表示は残る
  await page.waitForTimeout(1_200);
  await expect(
    card.getByRole('button', { name: 'コピーしました！' }),
  ).toBeVisible();
  await expect(
    card.getByRole('button', { name: 'AI総評プロンプトをコピー' }),
  ).toBeVisible({ timeout: 6_000 });
});

test('クリップボードへの書き込みが拒否されたとき失敗表示になる', async ({
  page,
}) => {
  await stubClipboard(page, 'reject');
  await setupAssetBalanceMocks(page);
  await page.setViewportSize({ width: 1920, height: 1080 });
  await gotoAssetBalance(page);
  await expect(page.getByTestId('portfolio-pie-chart')).toBeVisible();

  const card = page.getByTestId('asset-review-prompt-card');
  await card
    .getByRole('button', { name: 'AI総評プロンプトをコピー' })
    .click();
  await expect(
    card.getByRole('button', { name: 'コピーに失敗しました' }),
  ).toBeVisible();

  const written = await page.evaluate(
    () => (window as unknown as { __clipboardWrites: string[] }).__clipboardWrites,
  );
  expect(written).toEqual([EXPECTED_PROMPT]);

  // 3秒でラベルが元に戻る
  await expect(
    card.getByRole('button', { name: 'AI総評プロンプトをコピー' }),
  ).toBeVisible({ timeout: 6_000 });
});

test('Clipboard API がない環境では失敗表示になる', async ({ page }) => {
  await stubClipboard(page, 'missing');
  await setupAssetBalanceMocks(page);
  await page.setViewportSize({ width: 1920, height: 1080 });
  await gotoAssetBalance(page);
  await expect(page.getByTestId('portfolio-pie-chart')).toBeVisible();

  const card = page.getByTestId('asset-review-prompt-card');
  await card
    .getByRole('button', { name: 'AI総評プロンプトをコピー' })
    .click();
  await expect(
    card.getByRole('button', { name: 'コピーに失敗しました' }),
  ).toBeVisible();
});

test('保有データが0件のときコピーボタンは無効になる', async ({ page }) => {
  await stubClipboard(page);
  await setupAssetBalanceMocks(page, []);
  await page.setViewportSize({ width: 1920, height: 1080 });
  await gotoAssetBalance(page);
  await expect(page.getByText('資産管理データがありません')).toBeVisible();

  const card = page.getByTestId('asset-review-prompt-card');
  await expect(card).toBeVisible();
  await expect(
    card.getByRole('button', { name: 'AI総評プロンプトをコピー' }),
  ).toBeDisabled();
});
