/**
 * AssetBalancePage の認証境界・キャッシュ境界テスト
 *
 * 対象:
 * - 未認証時: API フェッチ抑制・ログインプロンプト表示
 * - 認証済み: DB データ取得・全件削除ボタン表示
 * - ログアウト: onLogout コールバックでキャッシュ除去
 */
import React from 'react';
import { render, screen, waitFor, act } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

import { AssetBalancePage } from '../AssetBalance';
import { assetBalanceQueryKeys } from '@/features/assetBalance/queryKeys';

// ────────────────────────────────────────────────────────
// モック定義
// ────────────────────────────────────────────────────────

vi.mock('@/components/templates/Layout', () => ({
  Layout: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));
vi.mock('@/components/atoms/PageHeader', () => ({
  PageHeader: ({ title }: { title: string }) => <h1>{title}</h1>,
}));
vi.mock('@/components/atoms/Alert', () => ({
  Alert: ({ children }: { children: React.ReactNode }) => <div role="alert">{children}</div>,
}));
vi.mock('@/components/atoms/Spinner', () => ({
  Spinner: () => <span role="status">loading</span>,
}));
vi.mock('@/components/atoms/Button', () => ({
  Button: ({ children, onClick }: { children: React.ReactNode; onClick?: () => void }) => (
    <button onClick={onClick}>{children}</button>
  ),
}));
vi.mock('@/components/molecules/CSVFileInput', () => ({
  CSVFileInput: () => <div data-testid="csv-file-input" />,
}));
vi.mock('@/components/molecules/ConfirmDeleteModal/ConfirmDeleteModal', () => ({
  ConfirmDeleteModal: ({ isOpen, onConfirm }: { isOpen: boolean; onConfirm: () => void }) =>
    isOpen ? (
      <button data-testid="confirm-delete" onClick={onConfirm}>削除する</button>
    ) : null,
}));
vi.mock('@/components/organisms/AssetPortfolioSummary', () => ({
  AssetPortfolioSummary: ({ assetBalanceData }: { assetBalanceData: unknown[] }) => (
    <div data-testid="portfolio-summary">{assetBalanceData.length}</div>
  ),
}));
vi.mock('@/components/organisms/SearchCard/SearchCard', () => ({
  SearchCard: () => <div data-testid="search-card" />,
}));

// AssetBalance API
import * as assetBalanceApiModule from '@/features/assetBalance/api/assetBalanceApi';
vi.mock('@/features/assetBalance/api/assetBalanceApi', () => ({
  assetBalanceApi: {
    list: vi.fn().mockResolvedValue([]),
    bulkCreate: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0 }),
    deleteAll: vi.fn().mockResolvedValue({}),
  },
}));

vi.mock('@/features/jquants/hooks/useDividendBatch', () => ({
  useDividendBatch: vi.fn(() => ({
    dividendPerShareMap: new Map(),
    dividendStatusMap: new Map(),
    fetchedCount: 0,
    totalCount: 0,
  })),
}));

vi.mock('@/hooks/useCSVReader', () => ({
  useCSVReader: () => ({
    parseCSV: vi.fn().mockResolvedValue([]),
    isLoading: false,
    error: null,
    fileName: null,
    resetError: vi.fn(),
    reset: vi.fn(),
  }),
}));

import * as authHook from '@/features/auth/hooks/useAuth';
vi.mock('@/features/auth/hooks/useAuth');

// ────────────────────────────────────────────────────────
// ユーティリティ
// ────────────────────────────────────────────────────────

type LogoutCallback = () => void;
type UseAuthReturn = ReturnType<typeof authHook.useAuth>;

function makeAuthMock(opts: {
  isAuthenticated?: boolean;
  authLoading?: boolean;
  userId?: string;
  onLogoutCapture?: (cb: LogoutCallback) => void;
}): UseAuthReturn {
  const { isAuthenticated = false, authLoading = false, userId = 'user-1', onLogoutCapture } = opts;
  return {
    user: isAuthenticated ? { id: userId, email: 'test@example.com' } : null,
    setUser: vi.fn(),
    login: vi.fn(),
    logout: vi.fn().mockResolvedValue(undefined),
    deleteAccount: vi.fn().mockResolvedValue(undefined),
    isAuthenticated,
    isLoading: authLoading,
    onLogout: (cb: LogoutCallback) => {
      onLogoutCapture?.(cb);
      return () => {};
    },
  };
}

function makeQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: Infinity },
    },
  });
}

function renderWithQuery(ui: React.ReactElement, qc?: QueryClient) {
  const client = qc ?? makeQueryClient();
  return render(<QueryClientProvider client={client}>{ui}</QueryClientProvider>);
}

// DBデータの型に合わせたモック行
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const mockDbRow: any = {
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

// ────────────────────────────────────────────────────────
// テスト
// ────────────────────────────────────────────────────────

describe('AssetBalancePage 認証境界・キャッシュ境界', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(assetBalanceApiModule.assetBalanceApi.list).mockResolvedValue([]);
  });

  it('未認証時: ログインプロンプトが表示され API フェッチが行われない', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));

    await act(async () => { renderWithQuery(<AssetBalancePage />); });

    await waitFor(() => {
      expect(screen.getByText(/ログインが必要です/)).toBeInTheDocument();
    });
    expect(assetBalanceApiModule.assetBalanceApi.list).not.toHaveBeenCalled();
  });

  it('認証済み・DBデータあり: 全件削除ボタンが表示される', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ isAuthenticated: true, userId: 'user-1' })
    );
    vi.mocked(assetBalanceApiModule.assetBalanceApi.list).mockResolvedValue([mockDbRow]);

    await act(async () => { renderWithQuery(<AssetBalancePage />); });

    await waitFor(() => {
      expect(screen.getByText(/全件削除/)).toBeInTheDocument();
    });
  });

  it('ログアウト: onLogout コールバック実行でキャッシュが除去される', async () => {
    let capturedCallback: LogoutCallback | null = null;
    const qc = makeQueryClient();

    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({
        isAuthenticated: true,
        userId: 'user-1',
        onLogoutCapture: (cb) => { capturedCallback = cb; },
      })
    );
    vi.mocked(assetBalanceApiModule.assetBalanceApi.list).mockResolvedValue([mockDbRow]);

    await act(async () => { renderWithQuery(<AssetBalancePage />, qc); });

    await waitFor(() => expect(capturedCallback).not.toBeNull());
    await waitFor(() => {
      expect(screen.getByText(/全件削除/)).toBeInTheDocument();
    });

    act(() => {
      // ログアウト: コールバック実行と同時に認証状態を false へ（実際のフローと同順）
      capturedCallback!();
      vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: false }));
    });

    await waitFor(() => {
      expect(screen.queryByText(/全件削除/)).not.toBeInTheDocument();
    });
    expect(qc.getQueryData(assetBalanceQueryKeys.all('user-1'))).toBeUndefined();
  });
});
