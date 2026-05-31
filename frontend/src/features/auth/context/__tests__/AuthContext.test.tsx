import { useRef, useState } from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AuthProvider } from '../AuthContext';
import { locationAssigner } from '../locationAssigner';
import { useAuth } from '../../hooks/useAuth';
import { useIdleTimer } from '../../hooks/useIdleTimer';
import { apiClient } from '@/lib/api/client';

vi.mock('../../hooks/useIdleTimer', () => ({
  useIdleTimer: vi.fn(),
}));

const { mockGet } = vi.hoisted(() => ({
  mockGet: vi.fn(),
}));

vi.mock('@/lib/api/client', () => ({
  apiClient: {
    post: vi.fn(),
    delete: vi.fn(),
  },
  createApiClient: () => ({
    get: mockGet,
  }),
}));

const mockUser = {
  id: '1',
  email: 'test@example.com',
  name: 'テストユーザー',
};

function TestConsumer({ onLogoutCallback }: { onLogoutCallback?: () => void }) {
  const { user, isLoading, isAuthenticated, login, logout, deleteAccount, onLogout } = useAuth();
  const cleanupRef = useRef<(() => void) | null>(null);
  const [error, setError] = useState('none');

  const handleLogout = async () => {
    try {
      await logout();
      setError('none');
    } catch (caughtError) {
      setError(caughtError instanceof Error ? caughtError.message : String(caughtError));
    }
  };

  const handleDeleteAccount = async () => {
    try {
      await deleteAccount();
      setError('none');
    } catch (caughtError) {
      setError(caughtError instanceof Error ? caughtError.message : String(caughtError));
    }
  };

  return (
    <div>
      <span data-testid="loading">{String(isLoading)}</span>
      <span data-testid="authenticated">{String(isAuthenticated)}</span>
      <span data-testid="user-name">{user?.name ?? 'none'}</span>
      <span data-testid="error">{error}</span>
      <button type="button" onClick={() => login()}>
        login
      </button>
      <button type="button" onClick={handleLogout}>
        logout
      </button>
      <button type="button" onClick={handleDeleteAccount}>
        delete-account
      </button>
      <button
        type="button"
        onClick={() => {
          cleanupRef.current = onLogout(() => onLogoutCallback?.());
        }}
      >
        register-callback
      </button>
      <button type="button" onClick={() => cleanupRef.current?.()}>
        unregister-callback
      </button>
    </div>
  );
}

async function renderAuthProvider({
  authenticated = true,
  onLogoutCallback,
}: {
  authenticated?: boolean;
  onLogoutCallback?: () => void;
} = {}) {
  if (authenticated) {
    mockGet.mockResolvedValueOnce(mockUser);
  } else {
    mockGet.mockRejectedValueOnce(new Error('Unauthorized'));
  }

  const user = userEvent.setup();

  render(
    <AuthProvider>
      <TestConsumer onLogoutCallback={onLogoutCallback} />
    </AuthProvider>
  );

  await waitFor(() => {
    expect(screen.getByTestId('loading').textContent).toBe('false');
  });

  return { user };
}

describe('AuthProvider', () => {
  beforeEach(() => {
    vi.unstubAllEnvs();
    mockGet.mockReset();
    vi.mocked(useIdleTimer).mockReset();
    vi.mocked(apiClient.post).mockReset();
    vi.mocked(apiClient.delete).mockReset();
    vi.mocked(useIdleTimer).mockReturnValue({ resetTimer: vi.fn() });
    vi.mocked(apiClient.post).mockResolvedValue(undefined);
    vi.mocked(apiClient.delete).mockResolvedValue(undefined);
  });

  it('認証成功時にユーザー情報がセットされる', async () => {
    mockGet.mockResolvedValueOnce(mockUser);

    render(
      <AuthProvider>
        <TestConsumer />
      </AuthProvider>
    );

    expect(screen.getByTestId('loading').textContent).toBe('true');

    await waitFor(() => {
      expect(screen.getByTestId('loading').textContent).toBe('false');
    });

    expect(screen.getByTestId('authenticated').textContent).toBe('true');
    expect(screen.getByTestId('user-name').textContent).toBe('テストユーザー');
    expect(mockGet).toHaveBeenCalledTimes(1);
    expect(mockGet).toHaveBeenCalledWith('/api/v1/session', expect.objectContaining({
      withCredentials: true,
      signal: expect.any(AbortSignal),
    }));
  });

  it('認証失敗時(401)にuser=nullになる', async () => {
    mockGet.mockRejectedValueOnce(new Error('Unauthorized'));

    render(
      <AuthProvider>
        <TestConsumer />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('loading').textContent).toBe('false');
    });

    expect(screen.getByTestId('authenticated').textContent).toBe('false');
    expect(screen.getByTestId('user-name').textContent).toBe('none');
    expect(mockGet).toHaveBeenCalledTimes(1);
  });

  it('アンマウント時にAbortControllerでリクエストがキャンセルされる', async () => {
    let capturedSignal: AbortSignal | undefined;
    mockGet.mockImplementation((_url: string, config?: { signal?: AbortSignal }) => {
      capturedSignal = config?.signal;
      return new Promise(resolve =>
        setTimeout(() => resolve({ id: '1', email: 'test@example.com', name: 'ユーザー' }), 100)
      );
    });

    const { unmount } = render(
      <AuthProvider>
        <TestConsumer />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(mockGet).toHaveBeenCalled();
    });

    unmount();
    expect(capturedSignal?.aborted).toBe(true);
  });

  it('認証確認専用クライアントがsignal付きで呼ばれる', async () => {
    mockGet.mockResolvedValueOnce({ id: '1', email: 'test@example.com' });

    render(
      <AuthProvider>
        <TestConsumer />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('loading').textContent).toBe('false');
    });

    expect(mockGet).toHaveBeenCalledWith('/api/v1/session', expect.objectContaining({
      withCredentials: true,
      signal: expect.any(AbortSignal),
    }));
  });

  it('logoutがapi/v1/sessionを削除しuserをnullにする', async () => {
    const { user } = await renderAuthProvider();

    await user.click(screen.getByRole('button', { name: 'logout' }));

    await waitFor(() => {
      expect(apiClient.delete).toHaveBeenCalledWith('/api/v1/session', {
        withCredentials: true,
      });
    });
    expect(screen.getByTestId('user-name').textContent).toBe('none');
    expect(screen.getByTestId('authenticated').textContent).toBe('false');
  });

  it('logoutがAPIエラーの場合はthrowしてcatch可能', async () => {
    vi.mocked(apiClient.delete).mockRejectedValueOnce(new Error('logout failed'));
    const { user } = await renderAuthProvider();

    await user.click(screen.getByRole('button', { name: 'logout' }));

    await waitFor(() => {
      expect(screen.getByTestId('error').textContent).toBe('logout failed');
    });
    expect(screen.getByTestId('user-name').textContent).toBe('none');
  });

  it('deleteAccountが確認APIを先に呼んでから削除APIを呼びuserをnullにする', async () => {
    const { user } = await renderAuthProvider();

    await user.click(screen.getByRole('button', { name: 'delete-account' }));

    await waitFor(() => {
      expect(apiClient.post).toHaveBeenCalledWith('/api/v1/account-deletion-confirmations', {}, {
        withCredentials: true,
        retry: { maxRetries: 0 },
      });
      expect(apiClient.delete).toHaveBeenCalledWith('/api/v1/account', {
        withCredentials: true,
      });
    });

    // 確認APIが削除APIより先に呼ばれることを検証
    const postCallOrder = vi.mocked(apiClient.post).mock.invocationCallOrder[0];
    const deleteCallOrder = vi.mocked(apiClient.delete).mock.invocationCallOrder[0];
    expect(postCallOrder).toBeLessThan(deleteCallOrder);

    expect(screen.getByTestId('user-name').textContent).toBe('none');
    expect(screen.getByTestId('authenticated').textContent).toBe('false');
  });

  it('deleteAccountで確認APIが失敗した場合は削除APIを呼ばずuserをnullにする', async () => {
    vi.mocked(apiClient.post).mockRejectedValueOnce(new Error('confirmation failed'));
    const { user } = await renderAuthProvider();

    await user.click(screen.getByRole('button', { name: 'delete-account' }));

    await waitFor(() => {
      expect(screen.getByTestId('error').textContent).toBe('confirmation failed');
    });
    expect(apiClient.delete).not.toHaveBeenCalled();
    expect(screen.getByTestId('user-name').textContent).toBe('none');
  });

  it('deleteAccountで削除APIが失敗した場合はthrowしてcatch可能', async () => {
    vi.mocked(apiClient.delete).mockRejectedValueOnce(new Error('delete failed'));
    const { user } = await renderAuthProvider();

    await user.click(screen.getByRole('button', { name: 'delete-account' }));

    await waitFor(() => {
      expect(screen.getByTestId('error').textContent).toBe('delete failed');
    });
    expect(apiClient.post).toHaveBeenCalledWith('/api/v1/account-deletion-confirmations', {}, {
      withCredentials: true,
      retry: { maxRetries: 0 },
    });
    expect(screen.getByTestId('user-name').textContent).toBe('none');
  });

  it('userがいる場合はアイドル時にlogoutが呼ばれる', async () => {
    await renderAuthProvider();

    await waitFor(() => {
      expect(useIdleTimer).toHaveBeenLastCalledWith(expect.objectContaining({
        timeout: expect.any(Number),
        onIdle: expect.any(Function),
        enabled: true,
      }));
    });

    const idleOptions = vi.mocked(useIdleTimer).mock.lastCall?.[0];
    idleOptions?.onIdle();

    await waitFor(() => {
      expect(apiClient.delete).toHaveBeenCalledWith('/api/v1/session', {
        withCredentials: true,
      });
    });
  });

  it('loginがwindow.location.assignを呼ぶ', async () => {
    vi.stubEnv('VITE_SHOKEN_WEBAPI_API_URL', 'https://api.example.com');
    const assignSpy = vi.spyOn(locationAssigner, 'assign').mockImplementation(vi.fn());
    const { user } = await renderAuthProvider({ authenticated: false });

    await user.click(screen.getByRole('button', { name: 'login' }));

    expect(assignSpy).toHaveBeenCalledWith('https://api.example.com/api/v1/oauth/google/authorize');

    assignSpy.mockRestore();
  });

  it('setUserを呼ぶとユーザー情報が更新される', async () => {
    mockGet.mockResolvedValueOnce(mockUser);

    function SetUserConsumer() {
      const { user, setUser } = useAuth();
      return (
        <div>
          <span data-testid="name">{user?.name ?? 'none'}</span>
          <button type="button" onClick={() => setUser({ id: '2', name: '更新ユーザー', email: 'new@example.com' })}>
            setUser
          </button>
        </div>
      );
    }

    const user = userEvent.setup();
    render(
      <AuthProvider>
        <SetUserConsumer />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('name').textContent).toBe('テストユーザー');
    });

    await user.click(screen.getByRole('button', { name: 'setUser' }));

    await waitFor(() => {
      expect(screen.getByTestId('name').textContent).toBe('更新ユーザー');
    });
  });

  it('login=successパラメータがある場合にhistory.replaceStateでURLをクリアする', async () => {
    const replaceStateSpy = vi.spyOn(window.history, 'replaceState').mockImplementation(vi.fn());
    // URLSearchParamsがlogin=successを返すようにsearch文字列を差し替え
    const originalSearch = window.location.search;
    Object.defineProperty(window, 'location', {
      configurable: true,
      get: () => ({ search: '?login=success', pathname: '/home' }),
    });

    mockGet.mockResolvedValueOnce(mockUser);

    render(
      <AuthProvider>
        <TestConsumer />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('loading').textContent).toBe('false');
    });

    expect(replaceStateSpy).toHaveBeenCalledWith({}, '', '/home');

    // 後片付け
    Object.defineProperty(window, 'location', {
      configurable: true,
      get: () => ({ search: originalSearch, pathname: '/' }),
    });
    replaceStateSpy.mockRestore();
  });

  it('userがnullの場合はアイドル時にlogoutが呼ばれない', async () => {
    await renderAuthProvider({ authenticated: false });

    await waitFor(() => {
      expect(useIdleTimer).toHaveBeenLastCalledWith(expect.objectContaining({
        enabled: false,
      }));
    });

    const idleOptions = vi.mocked(useIdleTimer).mock.lastCall?.[0];
    idleOptions?.onIdle();

    // userがnullなのでlogoutは呼ばれない
    expect(apiClient.delete).not.toHaveBeenCalled();
  });

  it('onLogoutでコールバックを登録解除できる', async () => {
    const onLogoutCallback = vi.fn();
    const { user } = await renderAuthProvider({ onLogoutCallback });

    await user.click(screen.getByRole('button', { name: 'register-callback' }));
    await user.click(screen.getByRole('button', { name: 'logout' }));

    await waitFor(() => {
      expect(onLogoutCallback).toHaveBeenCalledTimes(1);
    });

    await user.click(screen.getByRole('button', { name: 'unregister-callback' }));
    await user.click(screen.getByRole('button', { name: 'logout' }));

    await waitFor(() => {
      expect(apiClient.delete).toHaveBeenCalledTimes(2);
    });
    expect(onLogoutCallback).toHaveBeenCalledTimes(1);
  });
});
