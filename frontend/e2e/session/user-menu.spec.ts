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

  // ArrowDown でログアウトへ移動
  await page.keyboard.press('ArrowDown');
  await expect(page.getByRole('menuitem', { name: 'ログアウト' })).toBeFocused();

  // ArrowUp で設定に戻る
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

  await page.getByRole('button', { name: 'アカウントを削除' }).click();

  const dialog = page.getByRole('dialog', { name: 'アカウント削除の確認' });
  await expect(dialog).toBeVisible();

  await dialog.getByRole('button', { name: 'キャンセル' }).click();
  await expect(dialog).toHaveCount(0);
  expect(calls).toHaveLength(0);

  await page.getByRole('button', { name: '削除…' }).first().click();
  await dialog.getByRole('button', { name: '削除する' }).click();

  await expect.poll(() => calls.join(',')).toBe('confirm,delete');
  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'ログイン' })).toBeVisible();
});

test('設定ページにアカウント・データ・危険な操作のセクションが表示される', async ({ page }) => {
  await mockAuthorizedAuth(page);
  await page.goto('/settings');

  await expect(page.getByRole('heading', { name: '設定' })).toBeVisible();

  // アカウントセクション
  await expect(page.getByRole('heading', { name: 'アカウント' })).toBeVisible();
  await expect(page.getByText('テストユーザー')).toBeVisible();
  await expect(page.getByText('test@example.com')).toBeVisible();
  await expect(page.getByText(/Google でログイン中|メールでログイン中/)).toBeVisible();
  await expect(page.getByText('登録日')).toBeVisible();

  // データセクション
  await expect(page.getByRole('heading', { name: 'データ' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'CSV管理へ →' })).toBeVisible();

  // 危険な操作セクション
  await expect(page.getByRole('heading', { name: '危険な操作' })).toBeVisible();
  await expect(page.getByText('保存済みデータをすべて削除')).toBeVisible();
  await expect(page.getByText('アカウントを削除')).toBeVisible();

  // 表示・セキュリティセクションは段階2なので表示されない
  await expect(page.getByRole('heading', { name: '表示' })).toHaveCount(0);
  await expect(page.getByRole('heading', { name: 'セキュリティ' })).toHaveCount(0);
});