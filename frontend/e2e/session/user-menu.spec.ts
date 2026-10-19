import type { Page } from '@playwright/test';
import { expect, test } from '../support/test';

const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

async function mockAuthorizedAuth(page: Page) {
  await page.route(/\/api\/v1\/session$/, (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(MOCK_USER),
    }),
  );
}

test('メニューボタンでユーザーメニューが開閉し、aria-expanded が連動する', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.goto('/');

  const trigger = page.getByRole('button', { name: /テストユーザーのアカウントメニュー/ });
  const menu = page.getByRole('menu', { name: 'ユーザーメニュー' });

  await expect(trigger).toHaveAttribute('aria-expanded', 'false');
  await expect(trigger).toHaveAttribute('aria-haspopup', 'menu');
  await expect(trigger).toHaveAttribute('aria-controls', 'user-menu');
  await expect(menu).toHaveCount(0);

  await trigger.click();
  await expect(trigger).toHaveAttribute('aria-expanded', 'true');
  await expect(menu).toBeVisible();
  await expect(menu.getByRole('menuitem', { name: '設定' })).toBeVisible();
  await expect(menu.getByRole('menuitem', { name: 'データをエクスポート' })).toBeVisible();
  await expect(menu.getByRole('menuitem', { name: 'ログアウト' })).toBeVisible();
  await expect(menu.getByRole('menuitem', { name: 'アカウント削除' })).toHaveCount(0);

  await trigger.click();
  await expect(trigger).toHaveAttribute('aria-expanded', 'false');
  await expect(menu).toHaveCount(0);
});

test('ユーザーメニューが外側クリックと Escape で閉じる', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.goto('/');

  const trigger = page.getByRole('button', { name: /テストユーザーのアカウントメニュー/ });
  const menu = page.getByRole('menu', { name: 'ユーザーメニュー' });

  await trigger.click();
  await expect(menu).toBeVisible();
  await page.getByText('© 2026 証券Web').click();
  await expect(menu).toHaveCount(0);

  await trigger.click();
  await expect(menu).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(menu).toHaveCount(0);
});

test('キーボード操作でメニュー項目を移動できる', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.goto('/');

  const trigger = page.getByRole('button', { name: /テストユーザーのアカウントメニュー/ });
  const menu = page.getByRole('menu', { name: 'ユーザーメニュー' });

  await trigger.click();
  await expect(menu).toBeVisible();

  // 開いたとき先頭項目（設定）にフォーカスがある
  await expect(page.getByRole('menuitem', { name: '設定' })).toBeFocused();

  // ArrowDown でデータをエクスポート → ログアウトの順に移動
  await page.keyboard.press('ArrowDown');
  await expect(page.getByRole('menuitem', { name: 'データをエクスポート' })).toBeFocused();
  await page.keyboard.press('ArrowDown');
  await expect(page.getByRole('menuitem', { name: 'ログアウト' })).toBeFocused();

  // ArrowUp で設定に戻る
  await page.keyboard.press('ArrowUp');
  await page.keyboard.press('ArrowUp');
  await expect(page.getByRole('menuitem', { name: '設定' })).toBeFocused();

  // Escape で閉じてトリガーにフォーカスが戻る
  await page.keyboard.press('Escape');
  await expect(menu).toHaveCount(0);
  await expect(trigger).toBeFocused();
});

test('メニューから設定ページへ遷移できる', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.goto('/');

  const trigger = page.getByRole('button', { name: /テストユーザーのアカウントメニュー/ });
  await trigger.click();

  await page.getByRole('menuitem', { name: '設定' }).click();

  await expect(page).toHaveURL(/\/settings/);
  await expect(page.getByRole('heading', { name: '設定' })).toBeVisible();
});

test('ログアウトが実行されると未認証表示になる', async ({ page }) => {
  await mockAuthorizedAuth(page);
  const logoutCalls: string[] = [];
  await page.route(/\/api\/v1\/session$/, async (route, request) => {
    if (request.method() === 'DELETE') {
      logoutCalls.push('logout');
      await route.fulfill({ status: 200, contentType: 'application/json', body: '{"message":"ok"}' });
    } else {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(MOCK_USER),
      });
    }
  });
  await page.goto('/');

  const trigger = page.getByRole('button', { name: /テストユーザーのアカウントメニュー/ });
  await trigger.click();
  await page.getByRole('menuitem', { name: 'ログアウト' }).click();

  await expect.poll(() => logoutCalls.join(',')).toBe('logout');
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('設定ページの危険な操作からアカウント削除確認ダイアログが開く', async ({ page }) => {
  await mockAuthorizedAuth(page);
  const calls: string[] = [];
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    calls.push('confirm');
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.route(/\/api\/v1\/account$/, async (route) => {
    calls.push('delete');
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.goto('/settings');

  // アカウント削除ボタンは aria-label "アカウントを削除" を持つ
  await page.getByRole('button', { name: 'アカウントを削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await expect(dialog).toBeVisible();

  await dialog.getByRole('button', { name: 'キャンセル' }).click();
  await expect(dialog).toHaveCount(0);
  expect(calls).toHaveLength(0);

  await page.getByRole('button', { name: 'アカウントを削除' }).click();
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect.poll(() => calls.join(',')).toBe('confirm,delete');
  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('設定ページで確認APIが失敗したときダイアログにエラーが出て再試行で削除を完了できる', async ({
  page,
}) => {
  await mockAuthorizedAuth(page);
  const calls: string[] = [];
  let confirmFails = true;
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    calls.push('confirm');
    await route.fulfill(
      confirmFails
        ? {
            status: 500,
            contentType: 'application/json',
            body: '{"error":"server error"}',
          }
        : {
            status: 200,
            contentType: 'application/json',
            body: '{"message":"ok"}',
          },
    );
  });
  await page.route(/\/api\/v1\/account$/, async (route) => {
    calls.push('delete');
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.goto('/settings');

  await page.getByRole('button', { name: 'アカウントを削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect.poll(() => calls.length).toBe(1);
  await page.waitForTimeout(1500);
  expect(calls).toEqual(['confirm']);
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('alert')).toContainText(
    'サーバーエラーが発生しました。しばらくしてから再度お試しください',
  );
  await expect(page.getByRole('button', { name: 'アカウントを削除' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'ログイン' })).toHaveCount(0);

  confirmFails = false;
  await dialog.getByRole('button', { name: '削除する' }).click();
  await expect.poll(() => calls.join(',')).toBe('confirm,confirm,delete');
  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('設定ページで削除APIが500を返したときはダイアログにエラーが出て再試行で削除を完了できる', async ({
  page,
}) => {
  await mockAuthorizedAuth(page);
  const calls: string[] = [];
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    calls.push('confirm');
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  let deleteFails = true;
  await page.route(/\/api\/v1\/account$/, async (route) => {
    calls.push('delete');
    await route.fulfill(
      deleteFails
        ? {
            status: 500,
            contentType: 'application/json',
            body: '{"error":{"code":"INTERNAL","message":"db error"}}',
          }
        : {
            status: 200,
            contentType: 'application/json',
            body: '{"message":"ok"}',
          },
    );
  });
  await page.goto('/settings');

  await page.getByRole('button', { name: 'アカウントを削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect.poll(() => calls.join(',')).toBe('confirm,delete');
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('alert')).toContainText(
    'サーバーエラーが発生しました。しばらくしてから再度お試しください',
  );
  await expect(page.getByRole('button', { name: 'アカウントを削除' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'ログイン' })).toHaveCount(0);

  deleteFails = false;
  await dialog.getByRole('button', { name: '削除する' }).click();
  await expect.poll(() => calls.join(',')).toBe('confirm,delete,confirm,delete');
  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('設定ページの削除処理中はキャンセル・Escape・背景クリックでダイアログを閉じられない', async ({
  page,
}) => {
  await mockAuthorizedAuth(page);
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 1000));
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.route(/\/api\/v1\/account$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.goto('/settings');

  await page.getByRole('button', { name: 'アカウントを削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect(dialog.getByRole('button', { name: 'キャンセル' })).toBeDisabled();
  await expect(dialog.getByRole('button', { name: '閉じる' })).toBeDisabled();
  await page.keyboard.press('Escape');
  await page.mouse.click(10, 10);
  await expect(dialog).toBeVisible();

  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('削除APIの応答が失われてセッションも失効したときは結果を確定せず認証切れを表示する', async ({
  page,
}) => {
  let sessionDeleted = false;
  await page.route(/\/api\/v1\/session$/, async (route) => {
    await route.fulfill({
      status: sessionDeleted ? 401 : 200,
      contentType: 'application/json',
      body: sessionDeleted ? '' : JSON.stringify(MOCK_USER),
    });
  });
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  let deleteAttempts = 0;
  await page.route(/\/api\/v1\/account$/, async (route) => {
    deleteAttempts += 1;
    sessionDeleted = true;
    await route.abort();
  });
  await page.goto('/settings');

  await page.getByRole('button', { name: 'アカウントを削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('alert')).toContainText('ログインが必要です');
  expect(deleteAttempts).toBe(1);
});

test('削除要求がサーバーに届かなかった一時的な失敗では再送して削除を完了する', async ({
  page,
}) => {
  await mockAuthorizedAuth(page);
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  let deleteAttempts = 0;
  await page.route(/\/api\/v1\/account$/, async (route) => {
    deleteAttempts += 1;
    if (deleteAttempts === 1) {
      await route.abort();
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.goto('/settings');

  await page.getByRole('button', { name: 'アカウントを削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  expect(deleteAttempts).toBe(2);
});

test('セッション失効で確認APIが401を返したとき削除APIは呼ばれずダイアログは開いたままになる', async ({
  page,
}) => {
  let sessionAlive = true;
  await page.route(/\/api\/v1\/session$/, async (route) => {
    await route.fulfill({
      status: sessionAlive ? 200 : 401,
      contentType: 'application/json',
      body: sessionAlive ? JSON.stringify(MOCK_USER) : '',
    });
  });
  let deleteAttempts = 0;
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    sessionAlive = false;
    await route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: '',
    });
  });
  await page.route(/\/api\/v1\/account$/, async (route) => {
    deleteAttempts += 1;
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.goto('/settings');

  await page.getByRole('button', { name: 'アカウントを削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('alert')).toContainText('ログインが必要です');
  expect(deleteAttempts).toBe(0);
});

test('削除APIが401を返したときはダイアログが開いたままになる', async ({ page }) => {
  let sessionAlive = true;
  await page.route(/\/api\/v1\/session$/, async (route) => {
    await route.fulfill({
      status: sessionAlive ? 200 : 401,
      contentType: 'application/json',
      body: sessionAlive ? JSON.stringify(MOCK_USER) : '',
    });
  });
  await page.route(/\/api\/v1\/account-deletion-confirmations$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"message":"ok"}',
    });
  });
  await page.route(/\/api\/v1\/account$/, async (route) => {
    sessionAlive = false;
    await route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: '',
    });
  });
  await page.goto('/settings');

  await page.getByRole('button', { name: 'アカウントを削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('alert')).toContainText('ログインが必要です');
});

test('設定ページにアカウント・表示・データ・セキュリティ・危険な操作のセクションが表示される', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.goto('/settings');

  await expect(page.getByRole('heading', { name: '設定' })).toBeVisible();

  // アカウントセクション（ヘッダーのユーザーメニューと重複しないようセクション内に絞る）
  const accountSection = page.locator('#account');
  await expect(accountSection.getByRole('heading', { name: 'アカウント' })).toBeVisible();
  await expect(accountSection.getByText('テストユーザー')).toBeVisible();
  await expect(accountSection.getByText('test@example.com')).toBeVisible();
  await expect(accountSection.getByText('Google でログイン中')).toBeVisible();
  // 登録日は現状 API から取得できないため非表示（段階2で対応予定）
  await expect(accountSection.getByText('登録日')).toHaveCount(0);

  // データセクション
  await expect(page.getByRole('heading', { name: 'データ' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'CSV管理へ →' })).toBeVisible();

  // 危険な操作セクション
  await expect(page.getByRole('heading', { name: '危険な操作' })).toBeVisible();
  await expect(page.getByText('取引明細：配当金を削除')).toBeVisible();
  await expect(page.getByText('取引明細：国内株式を削除')).toBeVisible();
  await expect(page.getByText('取引明細：投資信託を削除')).toBeVisible();
  await expect(page.getByText('資産管理データを削除')).toBeVisible();
  await expect(page.getByText('アカウントを削除')).toBeVisible();

  // 削除リンクの href に正しい kind パラメータが含まれることを確認
  // aria-label で特定する（表示テキストは「削除…」だが aria-label は「配当金データを削除」等）
  await expect(page.getByRole('link', { name: '配当金データを削除' })).toHaveAttribute('href', '/receipts/csv/delete?kind=dividend');
  await expect(page.getByRole('link', { name: '国内株式データを削除' })).toHaveAttribute('href', '/receipts/csv/delete?kind=stock');
  await expect(page.getByRole('link', { name: '投資信託データを削除' })).toHaveAttribute('href', '/receipts/csv/delete?kind=fund');
  await expect(page.getByRole('link', { name: '資産管理データを削除' })).toHaveAttribute('href', '/assetbalance/csv/delete');

  // 表示・セキュリティセクション
  await expect(page.getByRole('heading', { name: '表示' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'セキュリティ' })).toBeVisible();
});
