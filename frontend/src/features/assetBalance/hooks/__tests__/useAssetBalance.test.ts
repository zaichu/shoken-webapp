/**
 * useAssetBalance: ログアウト時のキャッシュクリアテスト
 *
 * AssetBalancePage を経由しない単独利用経路（DividendInfo など）でも
 * onLogout コールバックが発火した際にキャッシュが除去されることを検証する
 */
import { renderHook, act, waitFor } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { useAssetBalance } from '../useAssetBalance';
import { assetBalanceQueryKeys } from '../../queryKeys';
import * as assetBalanceApiModule from '@/features/assetBalance/api/assetBalanceApi';
import * as authHook from '@/features/auth/hooks/useAuth';

// ────────────────────────────────────────────────────────
// モック
// ────────────────────────────────────────────────────────

vi.mock('@/features/assetBalance/api/assetBalanceApi', () => ({
  assetBalanceApi: { list: vi.fn().mockResolvedValue([]) },
}));

vi.mock('@/features/auth/hooks/useAuth');

type LogoutCallback = () => void;
type UseAuthReturn = ReturnType<typeof authHook.useAuth>;

function makeAuthMock(opts: {
  isAuthenticated?: boolean;
  userId?: string;
  onLogoutCapture?: (cb: LogoutCallback) => void;
}): UseAuthReturn {
  const { isAuthenticated = true, userId = 'user-1', onLogoutCapture } = opts;
  return {
    user: isAuthenticated ? { id: userId, email: 'test@example.com' } : null,
    setUser: vi.fn(),
    login: vi.fn(),
    logout: vi.fn().mockResolvedValue(undefined),
    deleteAccount: vi.fn().mockResolvedValue(undefined),
    isAuthenticated,
    isLoading: false,
    onLogout: (cb: LogoutCallback) => {
      onLogoutCapture?.(cb);
      return () => {};
    },
  };
}

function makeWrapper(qc: QueryClient) {
  function Wrapper({ children }: { children: React.ReactNode }) {
    return React.createElement(QueryClientProvider, { client: qc }, children);
  }
  return Wrapper;
}

// ────────────────────────────────────────────────────────
// テスト
// ────────────────────────────────────────────────────────

describe('useAssetBalance: ログアウト時キャッシュクリア', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('onLogout コールバック実行で assetBalance キャッシュが除去される', async () => {
    let capturedCallback: LogoutCallback | null = null;
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ onLogoutCapture: (cb) => { capturedCallback = cb; } })
    );
    vi.mocked(assetBalanceApiModule.assetBalanceApi.list).mockResolvedValue([
      { security_code: '7203', security_name: 'トヨタ自動車', shares: 100 } as never,
    ]);

    renderHook(() => useAssetBalance(), { wrapper: makeWrapper(qc) });

    // キャッシュにデータが入るまで待つ
    await waitFor(() => {
      expect(qc.getQueryData(assetBalanceQueryKeys.all('user-1'))).toBeDefined();
    });
    await waitFor(() => expect(capturedCallback).not.toBeNull());

    act(() => {
      capturedCallback!();
      vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: false }));
    });

    await waitFor(() => {
      expect(qc.getQueryData(assetBalanceQueryKeys.all('user-1'))).toBeUndefined();
    });
  });
});
