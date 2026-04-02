import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { Header } from '../Header';

vi.mock('@/features/auth/hooks/useAuth', () => ({
  useAuth: vi.fn(),
}));

type AuthState = ReturnType<typeof useAuth>;

function createAuthState(overrides: Partial<AuthState> = {}): AuthState {
  return {
    user: {
      id: '1',
      name: 'テストユーザー',
      email: 'test@example.com',
      picture_url: undefined,
    },
    setUser: vi.fn(),
    login: vi.fn(),
    logout: vi.fn().mockResolvedValue(undefined),
    deleteAccount: vi.fn().mockResolvedValue(undefined),
    isAuthenticated: true,
    isLoading: false,
    onLogout: vi.fn(() => vi.fn()),
    ...overrides,
  };
}

function renderHeader(authOverrides: Partial<AuthState> = {}, route = '/') {
  const authState = createAuthState(authOverrides);
  vi.mocked(useAuth).mockReturnValue(authState);

  const user = userEvent.setup();

  render(
    <MemoryRouter initialEntries={[route]}>
      <Header />
    </MemoryRouter>
  );

  return { user, authState };
}

describe('Header', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(useAuth).mockReturnValue(createAuthState());
  });

  it('主要ナビゲーションリンクを表示する', () => {
    renderHeader();

    expect(screen.getByRole('link', { name: '銘柄検索' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: '資産管理' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: '取引明細' })).toBeInTheDocument();
  });

  it('ログイン状態でユーザーアバターを表示する', () => {
    renderHeader();

    expect(screen.getByLabelText('テストユーザー')).toBeInTheDocument();
  });

  it('未認証時にログインボタンを表示する', () => {
    renderHeader({
      user: null,
      isAuthenticated: false,
    });

    expect(screen.getByRole('button', { name: 'ログイン' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'メニュー' })).not.toBeInTheDocument();
  });

  it('ローディング中に読み込み中を表示する', () => {
    renderHeader({
      user: null,
      isAuthenticated: false,
      isLoading: true,
    });

    expect(screen.getByText('読み込み中...')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'ログイン' })).not.toBeInTheDocument();
  });

  it('メニューボタンをクリックするとドロップダウンが開く', async () => {
    const { user } = renderHeader();

    await user.click(screen.getByRole('button', { name: 'メニュー' }));

    expect(screen.getByRole('menu', { name: 'ユーザーメニュー' })).toBeInTheDocument();
    expect(screen.getByRole('menuitem', { name: 'ログアウト' })).toBeInTheDocument();
    expect(screen.getByRole('menuitem', { name: /アカウント削除/ })).toBeInTheDocument();
  });

  it('ログアウトボタンをクリックするとlogoutが呼ばれる', async () => {
    const logout = vi.fn().mockResolvedValue(undefined);
    const { user } = renderHeader({ logout });

    await user.click(screen.getByRole('button', { name: 'メニュー' }));
    await user.click(screen.getByRole('menuitem', { name: 'ログアウト' }));

    await waitFor(() => {
      expect(logout).toHaveBeenCalledTimes(1);
    });
    expect(screen.queryByRole('menu', { name: 'ユーザーメニュー' })).not.toBeInTheDocument();
  });

  it('アカウント削除ボタンをクリックすると確認モーダルが開く', async () => {
    const { user } = renderHeader();

    await user.click(screen.getByRole('button', { name: 'メニュー' }));
    await user.click(screen.getByRole('menuitem', { name: /アカウント削除/ }));

    expect(await screen.findByRole('dialog', { name: 'アカウント削除の確認' })).toBeInTheDocument();
  });

  it('削除確認モーダルの削除するでdeleteAccountが呼ばれてモーダルが閉じる', async () => {
    const deleteAccount = vi.fn().mockResolvedValue(undefined);
    const { user } = renderHeader({ deleteAccount });

    await user.click(screen.getByRole('button', { name: 'メニュー' }));
    await user.click(screen.getByRole('menuitem', { name: /アカウント削除/ }));
    await user.click(await screen.findByRole('button', { name: '削除する' }));

    await waitFor(() => {
      expect(deleteAccount).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'アカウント削除の確認' })).not.toBeInTheDocument();
    });
  });

  it('削除確認モーダルのキャンセルでモーダルが閉じる', async () => {
    const { user } = renderHeader();

    await user.click(screen.getByRole('button', { name: 'メニュー' }));
    await user.click(screen.getByRole('menuitem', { name: /アカウント削除/ }));
    await user.click(await screen.findByRole('button', { name: 'キャンセル' }));

    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'アカウント削除の確認' })).not.toBeInTheDocument();
    });
  });

  it('Escapeキーでドロップダウンが閉じる', async () => {
    const { user } = renderHeader();

    await user.click(screen.getByRole('button', { name: 'メニュー' }));
    expect(screen.getByRole('menu', { name: 'ユーザーメニュー' })).toBeInTheDocument();

    await user.keyboard('{Escape}');

    await waitFor(() => {
      expect(screen.queryByRole('menu', { name: 'ユーザーメニュー' })).not.toBeInTheDocument();
    });
  });

  it('外部クリックでドロップダウンが閉じる', async () => {
    const { user } = renderHeader();

    await user.click(screen.getByRole('button', { name: 'メニュー' }));
    expect(screen.getByRole('menu', { name: 'ユーザーメニュー' })).toBeInTheDocument();

    fireEvent.mouseDown(document.body);

    await waitFor(() => {
      expect(screen.queryByRole('menu', { name: 'ユーザーメニュー' })).not.toBeInTheDocument();
    });
  });

  it('picture_urlがある場合はアバター画像を表示する', () => {
    renderHeader({
      user: {
        id: '1',
        name: '画像ユーザー',
        email: 'image@example.com',
        picture_url: 'http://example.com/avatar.png',
      },
    });

    expect(screen.getByAltText('画像ユーザー')).toBeInTheDocument();
  });

  it('画像エラー時はイニシャルフォールバックを表示する', async () => {
    renderHeader({
      user: {
        id: '1',
        name: '画像エラー',
        email: 'image-error@example.com',
        picture_url: 'http://example.com/avatar.png',
      },
    });

    fireEvent.error(screen.getByAltText('画像エラー'));

    await waitFor(() => {
      expect(screen.queryByAltText('画像エラー')).not.toBeInTheDocument();
    });
    expect(screen.getByLabelText('画像エラー')).toHaveTextContent('画像');
  });

  it('アクティブなリンクにaria-current=pageが付く', () => {
    renderHeader({}, '/search');

    expect(screen.getByRole('link', { name: '銘柄検索' })).toHaveAttribute('aria-current', 'page');
  });

  it('ログインボタンをクリックするとloginが呼ばれる', async () => {
    const login = vi.fn();
    const { user } = renderHeader({ user: null, isAuthenticated: false, login });

    await user.click(screen.getByRole('button', { name: 'ログイン' }));

    expect(login).toHaveBeenCalledTimes(1);
  });

  it('削除確認モーダルのバックドロップクリックでモーダルが閉じる', async () => {
    const { user } = renderHeader();

    await user.click(screen.getByRole('button', { name: 'メニュー' }));
    await user.click(screen.getByRole('menuitem', { name: /アカウント削除/ }));
    const dialog = await screen.findByRole('dialog', { name: 'アカウント削除の確認' });
    expect(dialog).toBeInTheDocument();

    // バックドロップ（dialog要素自体）をクリック
    await user.click(dialog);

    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'アカウント削除の確認' })).not.toBeInTheDocument();
    });
  });

  it('削除確認モーダルでEscapeキーを押すとモーダルが閉じる', async () => {
    const { user } = renderHeader();

    await user.click(screen.getByRole('button', { name: 'メニュー' }));
    await user.click(screen.getByRole('menuitem', { name: /アカウント削除/ }));
    const dialog = await screen.findByRole('dialog', { name: 'アカウント削除の確認' });

    fireEvent.keyDown(dialog, { key: 'Escape' });

    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'アカウント削除の確認' })).not.toBeInTheDocument();
    });
  });

  it('削除確認モーダルの✕ボタンでモーダルが閉じる', async () => {
    const { user } = renderHeader();

    await user.click(screen.getByRole('button', { name: 'メニュー' }));
    await user.click(screen.getByRole('menuitem', { name: /アカウント削除/ }));
    await screen.findByRole('dialog', { name: 'アカウント削除の確認' });

    await user.click(screen.getByRole('button', { name: '閉じる' }));

    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'アカウント削除の確認' })).not.toBeInTheDocument();
    });
  });

  it('削除確認モーダル内でEscape以外のキーはstopPropagationで処理される', async () => {
    const { user } = renderHeader();

    await user.click(screen.getByRole('button', { name: 'メニュー' }));
    await user.click(screen.getByRole('menuitem', { name: /アカウント削除/ }));
    await screen.findByRole('dialog', { name: 'アカウント削除の確認' });

    // inner presentationにEnterキーを発火 → stopPropagation()パスが実行されモーダルは閉じない
    const presentation = screen.getByRole('presentation');
    fireEvent.keyDown(presentation, { key: 'Enter' });

    expect(screen.getByRole('dialog', { name: 'アカウント削除の確認' })).toBeInTheDocument();
  });

  describe('イニシャル表示', () => {
    it('スペース区切り2語の名前は実際の動作どおり2文字を表示する', () => {
      renderHeader({
        user: {
          id: '1',
          name: '山田 太郎',
          email: 'yamada@example.com',
          picture_url: undefined,
        },
      });

      expect(screen.getByLabelText('山田 太郎')).toHaveTextContent('山太');
    });

    it('1語の名前は先頭2文字を表示する', () => {
      renderHeader({
        user: {
          id: '1',
          name: '田中',
          email: 'tanaka@example.com',
          picture_url: undefined,
        },
      });

      expect(screen.getByLabelText('田中')).toHaveTextContent('田中');
    });

    it('名前なしでメールがある場合はメールの先頭2文字を表示する', () => {
      renderHeader({
        user: {
          id: '1',
          name: undefined,
          email: 'sample@example.com',
          picture_url: undefined,
        },
      });

      expect(screen.getByLabelText('ユーザー')).toHaveTextContent('SA');
    });

    it('名前もメールもない場合はUを表示する', () => {
      renderHeader({
        user: {
          id: '1',
          name: '',
          email: '',
          picture_url: undefined,
        },
      });

      expect(screen.getByLabelText('ユーザー')).toHaveTextContent('U');
    });
  });
});
