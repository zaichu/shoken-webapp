import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AuthProvider } from '../AuthContext';
import { useAuth } from '../../hooks/useAuth';

// アイドルタイマーのモック
vi.mock('../../hooks/useIdleTimer', () => ({
  useIdleTimer: vi.fn(),
}));

// vi.hoisted でモック関数をホイスティング対応にする
const { mockGet } = vi.hoisted(() => ({
  mockGet: vi.fn(),
}));

// APIクライアントのモック
vi.mock('@/lib/api/client', () => ({
  apiClient: {
    post: vi.fn(),
    delete: vi.fn(),
  },
  createApiClient: () => ({
    get: mockGet,
  }),
}));

// テスト用コンシューマーコンポーネント
function TestConsumer() {
  const { user, isLoading, isAuthenticated } = useAuth();
  return (
    <div>
      <span data-testid="loading">{String(isLoading)}</span>
      <span data-testid="authenticated">{String(isAuthenticated)}</span>
      <span data-testid="user-name">{user?.name ?? 'none'}</span>
    </div>
  );
}

describe('AuthProvider', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('認証成功時にユーザー情報がセットされる', async () => {
    const mockUser = { id: '1', email: 'test@example.com', name: 'テストユーザー' };
    mockGet.mockResolvedValueOnce(mockUser);

    render(
      <AuthProvider>
        <TestConsumer />
      </AuthProvider>
    );

    // 初期状態: ローディング中
    expect(screen.getByTestId('loading').textContent).toBe('true');

    await waitFor(() => {
      expect(screen.getByTestId('loading').textContent).toBe('false');
    });

    expect(screen.getByTestId('authenticated').textContent).toBe('true');
    expect(screen.getByTestId('user-name').textContent).toBe('テストユーザー');
    expect(mockGet).toHaveBeenCalledTimes(1);
    expect(mockGet).toHaveBeenCalledWith('/auth/me', expect.objectContaining({
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
    // signalのabortを検知するためのモック
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

    // リクエスト発行を待つ
    await waitFor(() => {
      expect(mockGet).toHaveBeenCalled();
    });

    // アンマウント時にabortされる
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

    // createApiClientで生成されたクライアントのgetがsignal付きで呼ばれていること
    expect(mockGet).toHaveBeenCalledWith('/auth/me', expect.objectContaining({
      withCredentials: true,
      signal: expect.any(AbortSignal),
    }));
  });
});
