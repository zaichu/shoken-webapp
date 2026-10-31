import { expect, test, json, MOCK_USER } from '../support/test';

const entries = Array.from({ length: 64 }, (_, index) => ({
  occurred_at: new Date(Date.UTC(2026, 9, 31, 0, 12) - index * 3600000).toISOString(),
  email: index === 0 ? 'long.email.for.administrator.login.history.display.check@example.com' : `user${index}@example.com`,
  result: index % 2 === 0 ? 'success' : 'failure',
  country: 'JP', region: index % 2 === 0 ? 'Tokyo' : 'Osaka', browser: 'chrome', os: 'linux',
}));

for (const permission of [undefined, false]) {
  test(`管理者権限が ${permission} のときメニューと画面を出さない`, async ({ page }) => {
    await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json({ ...MOCK_USER, is_admin: permission })));
    let calls = 0;
    await page.route(/\/api\/v1\/admin\/logins(?:\?.*)?$/, (route) => {
      calls += 1;
      return route.fulfill(json({}, 403));
    });
    await page.goto('/settings');
    await expect(page.getByRole('heading', { name: '設定', exact: true })).toBeVisible();
    await page.getByRole('button', { name: /のアカウントメニュー/ }).filter({ visible: true }).click();
    await expect(page.getByRole('menuitem', { name: 'ログイン一覧(管理者)' })).toHaveCount(0);
    await page.goto('/admin/logins');
    await expect(page.getByRole('heading', { name: 'ページが見つかりません', exact: true })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'ログイン一覧(管理者)' })).toHaveCount(0);
    expect(calls).toBe(0);
  });
}

test('管理者メニューから開き、検索・並べ替え・ページ送りを API に渡す', async ({ page }) => {
  await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json({ ...MOCK_USER, is_admin: true })));
  await page.route(/\/api\/v1\/admin\/logins(?:\?.*)?$/, (route) => {
    const params = new URL(route.request().url()).searchParams;
    const number = Number(params.get('page'));
    const data = params.get('q') ? [] : entries.slice((number - 1) * 50, number * 50);
    return route.fulfill(json({ data, page: number, per_page: 50, total: params.get('q') ? 0 : entries.length }));
  });
  await page.goto('/settings');
  await expect(page.getByRole('heading', { name: '設定', exact: true })).toBeVisible();
  await page.getByRole('button', { name: /のアカウントメニュー/ }).filter({ visible: true }).click();
  await page.getByRole('menuitem', { name: 'ログイン一覧(管理者)' }).click();
  const table = page.getByTestId('admin-login-table');
  await expect(table.locator('tbody tr')).toHaveCount(50);
  const pageTwo = page.waitForRequest((request) => request.url().includes('/admin/logins?') && new URL(request.url()).searchParams.get('page') === '2');
  await page.getByRole('button', { name: '次へ', exact: true }).click();
  await pageTwo;
  await expect(table.locator('tbody tr')).toHaveCount(14);
  await expect(page.getByRole('button', { name: '次へ', exact: true })).toBeDisabled();
  const sorted = page.waitForRequest((request) => {
    const url = new URL(request.url());
    return url.pathname.endsWith('/admin/logins') && url.searchParams.get('sort_by') === 'email' && url.searchParams.get('page') === '1';
  });
  await table.getByRole('button', { name: 'メール', exact: true }).click();
  await sorted;
  await expect(table.locator('tbody tr')).toHaveCount(50);
  await page.getByRole('searchbox', { name: 'メール・場所で検索', exact: true }).fill('no-matching-login');
  await expect(page.getByTestId('empty-state')).toBeVisible();
  await page.getByRole('button', { name: 'すべてクリア', exact: true }).click();
  await expect(table.locator('tbody tr')).toHaveCount(50);
});

test('一覧 API が 403 を返したら管理画面と管理者メニューを閉じる', async ({ page }) => {
  await page.route(/\/api\/v1\/session$/, (route) => route.fulfill(json({ ...MOCK_USER, is_admin: true })));
  await page.route(/\/api\/v1\/admin\/logins(?:\?.*)?$/, (route) => route.fulfill(json({ error: { code: 'FORBIDDEN', message: '管理者権限が必要です' } }, 403)));
  await page.goto('/admin/logins');
  await expect(page.getByRole('heading', { name: 'ページが見つかりません', exact: true })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'ログイン一覧(管理者)' })).toHaveCount(0);
});
