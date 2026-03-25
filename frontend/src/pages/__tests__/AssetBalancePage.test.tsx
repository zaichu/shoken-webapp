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
import userEvent from '@testing-library/user-event';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

import { AssetBalancePage } from '../AssetBalance';
import { assetBalanceQueryKeys } from '@/features/assetBalance/queryKeys';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';

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
  CSVFileInput: ({ onFileSelect }: { onFileSelect: (file: File) => void }) => (
    <button data-testid="csv-file-input" onClick={() => onFileSelect(new File(['dummy'], 'asset.csv'))}>
      CSV読込
    </button>
  ),
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
    previewCsv: vi.fn().mockResolvedValue({ total_rows: 0, valid_rows: 0, errors: [], rows: [] }),
    uploadCsv: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0, errors: [] }),
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
const mockDbRow: AssetBalanceData = {
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

  it('認証済み時: workspace は広めの right rail レイアウトで表示される', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ isAuthenticated: true, userId: 'user-1' })
    );
    vi.mocked(assetBalanceApiModule.assetBalanceApi.list).mockResolvedValue([mockDbRow]);

    await act(async () => { renderWithQuery(<AssetBalancePage />); });

    await waitFor(() => {
      expect(screen.getByTestId('assetbalance-workspace')).toBeInTheDocument();
    });

    expect(screen.getByTestId('assetbalance-workspace').className).toContain('lg:grid-cols-[minmax(0,1fr)_22rem]');
    expect(screen.getByTestId('assetbalance-workspace').className).toContain('xl:grid-cols-[minmax(0,1fr)_24rem]');
  });

  it('保存後: right rail に軽い confirmation strip が表示される', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ isAuthenticated: true, userId: 'user-1' })
    );
    vi.mocked(assetBalanceApiModule.assetBalanceApi.list).mockResolvedValue([]);
    vi.mocked(assetBalanceApiModule.assetBalanceApi.previewCsv).mockResolvedValue({
      total_rows: 2,
      valid_rows: 2,
      errors: [],
      rows: [mockDbRow, { ...mockDbRow, security_code: '6758' }],
    } as never);
    vi.mocked(assetBalanceApiModule.assetBalanceApi.uploadCsv).mockResolvedValue({
      inserted: 2,
      skipped: 1,
      errors: [],
    } as never);

    await act(async () => { renderWithQuery(<AssetBalancePage />); });

    const user = userEvent.setup();
    await user.click(screen.getByTestId('csv-file-input'));

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /全件置換で保存/ })).toBeInTheDocument();
    });

    await user.click(screen.getByRole('button', { name: /全件置換で保存/ }));

    await waitFor(() => {
      expect(screen.getByText('保存しました')).toBeInTheDocument();
    });

    expect(screen.getByText('2件反映')).toBeInTheDocument();
    expect(screen.getByText('全件置換')).toBeInTheDocument();
    expect(screen.getByText('1件スキップ')).toBeInTheDocument();
    expect(screen.queryByText(/2件保存しました/)).not.toBeInTheDocument();
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
