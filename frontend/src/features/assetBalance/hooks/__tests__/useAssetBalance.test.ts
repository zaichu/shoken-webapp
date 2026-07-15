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
import type { AssetBalanceApiData } from '@/types/api';

// ────────────────────────────────────────────────────────
// モック
// ────────────────────────────────────────────────────────

vi.mock('@/features/assetBalance/api/assetBalanceApi', () => ({
  assetBalanceApi: { list: vi.fn().mockResolvedValue({ data: [], total: 0, page: 1, per_page: 1 }) },
}));
const emptyPage = { data: [], total: 0, page: 1, per_page: 1 };

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

function makeAssetBalanceData(overrides: Partial<AssetBalanceApiData> = {}): AssetBalanceApiData {
  return {
    id: 'asset-balance-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '7203',
    security_name: 'トヨタ自動車',
    shares: 100,
    executing_shares: 0,
    average_purchase_price: 2000,
    total_purchase_amount: 200000,
    current_price: 2100,
    daily_change: 10,
    market_value: 210000,
    profit_loss_rate: 5,
    ...overrides,
  };
}

// ────────────────────────────────────────────────────────
// テスト
// ────────────────────────────────────────────────────────

describe('useAssetBalance', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));
    vi.mocked(assetBalanceApiModule.assetBalanceApi.list).mockResolvedValue(emptyPage);
  });

  it('onLogout コールバック実行で assetBalance キャッシュが除去される', async () => {
    let capturedCallback: LogoutCallback | null = null;
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ onLogoutCapture: (cb) => { capturedCallback = cb; } })
    );
    vi.mocked(assetBalanceApiModule.assetBalanceApi.list).mockResolvedValue({
      data: [{ security_code: '7203', security_name: 'トヨタ自動車', shares: 100 } as never],
      total: 1, page: 1, per_page: 1,
    });

    renderHook(() => useAssetBalance({ securityCode: '7203' }), { wrapper: makeWrapper(qc) });

    // キャッシュにデータが入るまで待つ
    await waitFor(() => {
      expect(qc.getQueryData(assetBalanceQueryKeys.lookup('user-1', '7203'))).toBeDefined();
    });
    await waitFor(() => expect(capturedCallback).not.toBeNull());

    act(() => {
      capturedCallback!();
      vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: false }));
    });

    await waitFor(() => {
      expect(qc.getQueryData(assetBalanceQueryKeys.lookup('user-1', '7203'))).toBeUndefined();
    });
  });

  it('getAssetBalanceByCode は空文字コードに undefined を返す', () => {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    const { result } = renderHook(() => useAssetBalance({ securityCode: '7203' }), { wrapper: makeWrapper(qc) });

    expect(result.current.getAssetBalanceByCode('')).toBeUndefined();
  });

  it('getAssetBalanceByCode は正規化後に一致する銘柄を返す', async () => {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    vi.mocked(assetBalanceApiModule.assetBalanceApi.list).mockResolvedValue({
      data: [makeAssetBalanceData({ security_code: '7203' })],
      total: 1, page: 1, per_page: 1,
    });

    const { result } = renderHook(() => useAssetBalance({ securityCode: '7203' }), { wrapper: makeWrapper(qc) });

    await waitFor(() => {
      expect(result.current.assetBalanceData).toHaveLength(1);
    });

    expect(result.current.getAssetBalanceByCode(' 7203 ')).toMatchObject({
      security_code: '7203',
      security_name: 'トヨタ自動車',
    });
  });

  it('未認証時はassetBalanceDataが空配列を返す', () => {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: false }));

    const { result } = renderHook(() => useAssetBalance({ securityCode: '7203' }), { wrapper: makeWrapper(qc) });

    expect(result.current.assetBalanceData).toEqual([]);
  });

  it('securityCode 指定時は銘柄コードで1回だけAPI取得する', async () => {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    vi.mocked(assetBalanceApiModule.assetBalanceApi.list).mockResolvedValue({
      data: [makeAssetBalanceData({ security_code: '7203' })],
      total: 1,
      page: 1,
      per_page: 1,
    });

    const { result } = renderHook(() => useAssetBalance({ securityCode: ' 7203 ' }), { wrapper: makeWrapper(qc) });

    await waitFor(() => {
      expect(result.current.assetBalanceData).toHaveLength(1);
    });

    expect(assetBalanceApiModule.assetBalanceApi.list).toHaveBeenCalledTimes(1);
    expect(assetBalanceApiModule.assetBalanceApi.list).toHaveBeenCalledWith({
      page: 1,
      per_page: 1,
      security_code: '7203',
    });
    expect(result.current.getAssetBalanceByCode('7203')).toMatchObject({ security_code: '7203' });
  });

  it('securityCode が空文字の場合はAPIを呼ばない', () => {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    renderHook(() => useAssetBalance({ securityCode: '' }), { wrapper: makeWrapper(qc) });

    expect(assetBalanceApiModule.assetBalanceApi.list).not.toHaveBeenCalled();
  });

  it('refetch は該当銘柄の lookup クエリを invalidate する', async () => {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });
    const invalidateSpy = vi.spyOn(qc, 'invalidateQueries');

    const { result } = renderHook(() => useAssetBalance({ securityCode: '7203' }), { wrapper: makeWrapper(qc) });

    await act(async () => {
      await result.current.refetch();
    });

    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: assetBalanceQueryKeys.lookup('user-1', '7203'),
    });
  });
});
