import React from 'react';
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
    expect(mockGet).toHaveBeenCalledWith('/auth/me', { withCredentials: true });
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

  it('checkSessionが重複して呼ばれない', async () => {
    // 遅延レスポンスをシミュレート
    mockGet.mockImplementation(() =>
      new Promise(resolve => setTimeout(() => resolve({ id: '1', email: 'test@example.com', name: 'ユーザー' }), 50))
    );

    const { unmount } = render(
      <AuthProvider>
        <TestConsumer />
      </AuthProvider>
    );

    unmount();

    // 再マウント（StrictModeでの二重レンダリングを模倣）
    render(
      <React.StrictMode>
        <AuthProvider>
          <TestConsumer />
        </AuthProvider>
      </React.StrictMode>
    );

    await waitFor(() => {
      expect(screen.getByTestId('loading').textContent).toBe('false');
    });

    // StrictModeでも過剰な呼び出しが起きないことを確認
    // unmount後は新インスタンスなので再マウント時に1回呼ばれる
    expect(mockGet.mock.calls.length).toBeGreaterThanOrEqual(1);
    expect(mockGet.mock.calls.length).toBeLessThanOrEqual(2);
  });

  it('認証確認専用クライアントが使用される', async () => {
    mockGet.mockResolvedValueOnce({ id: '1', email: 'test@example.com' });

    render(
      <AuthProvider>
        <TestConsumer />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('loading').textContent).toBe('false');
    });

    // createApiClientで生成されたクライアントのgetが呼ばれていること
    expect(mockGet).toHaveBeenCalledWith('/auth/me', { withCredentials: true });
  });
});
