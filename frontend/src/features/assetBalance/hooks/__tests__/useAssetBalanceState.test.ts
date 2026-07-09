import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { AssetBalanceData } from '@/types/api';
import { useAssetBalanceState } from '../useAssetBalanceState';
import * as authHook from '@/features/auth/hooks/useAuth';
import * as dataSourceHook from '../useAssetBalanceDataSource';

vi.mock('@/features/auth/hooks/useAuth');
vi.mock('../useAssetBalanceDataSource');
vi.mock('@/features/dividendPerShare/hooks/useDividendBatch', () => ({
  useDividendBatch: vi.fn(() => ({
    dividendPerShareMap: new Map(),
    dividendStatusMap: new Map(),
  })),
}));

type LogoutCallback = () => void;
type UseAuthReturn = ReturnType<typeof authHook.useAuth>;
type UseAssetBalanceDataSourceReturn = ReturnType<typeof dataSourceHook.useAssetBalanceDataSourceCore>;

function makeAuthMock(opts: {
  isAuthenticated?: boolean;
  authLoading?: boolean;
  onLogoutCapture?: (cb: LogoutCallback) => void;
}): UseAuthReturn {
  const { isAuthenticated = true, authLoading = false, onLogoutCapture } = opts;

  return {
    user: isAuthenticated ? { id: 'user-1', email: 'test@example.com' } : null,
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

function makeRow(overrides: Partial<AssetBalanceData> = {}): AssetBalanceData {
  return {
    id: 'asset-balance-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
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
    ...overrides,
  };
}

function makeDataSourceMock(
  overrides: Partial<UseAssetBalanceDataSourceReturn> = {}
): UseAssetBalanceDataSourceReturn {
  const dbData = overrides.dbData ?? [];

  return {
    dbData,
    dbTotal: overrides.dbTotal ?? dbData.length,
    summary: undefined,
    facets: undefined,
    assetBalanceListLimit: 1000,
    previewRows: [],
    loading: false,
    error: null,
    saving: false,
    deleting: false,
    previewing: false,
    lastSavedResult: null,
    hasCsvFile: false,
    hasDbData: false,
    csvFileName: null,
    handleFileSelect: vi.fn(),
    handleSaveToDB: vi.fn(),
    handleDeleteAll: vi.fn().mockResolvedValue(undefined),
    resetState: vi.fn(),
    ...overrides,
  };
}

describe('useAssetBalanceState', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(makeDataSourceMock());
  });

  it('プレビュー行を優先し、rail props と検索結果を組み立てる', () => {
    const previewRows = [
      makeRow(),
      makeRow({
        id: 'asset-balance-2',
        security_code: '6758',
        security_name: 'ソニーグループ',
      }),
    ];

    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        dbData: [makeRow({ security_code: '9984', security_name: 'ソフトバンクグループ' })],
        previewRows,
        hasCsvFile: true,
        hasDbData: true,
        csvFileName: 'assetbalance.csv',
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    expect(result.current.assetBalanceData).toEqual(previewRows);
    expect(result.current.filteredData).toEqual(previewRows);
    expect(result.current.utilityRailProps.actionRailProps.selectedFileName).toBe('assetbalance.csv');
    expect(result.current.utilityRailProps.actionRailProps.saveLabel).toBe('2件 全件置換で保存');
    expect(result.current.utilityRailProps.actionRailProps.deleteLabel).toBe('全件削除 (1件)');
    expect(result.current.utilityRailProps.searchCardProps.visible).toBe(true);
    expect(result.current.utilityRailProps.reviewPromptCardProps.assetBalanceData).toEqual(previewRows);

    act(() => {
      result.current.utilityRailProps.searchCardProps.onSearch('6758');
    });

    expect(result.current.utilityRailProps.searchCardProps.value).toBe('6758');
    expect(result.current.filteredData).toEqual([previewRows[1]]);
    expect(result.current.utilityRailProps.reviewPromptCardProps.assetBalanceData).toEqual(previewRows);
  });

  it('削除確認を開き、confirmDeleteAll で閉じて削除処理を実行する', async () => {
    const handleDeleteAll = vi.fn().mockResolvedValue(undefined);
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        dbData: [makeRow()],
        hasDbData: true,
        handleDeleteAll,
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    act(() => {
      result.current.utilityRailProps.actionRailProps.onDeleteRequest();
    });

    expect(result.current.showDeleteConfirm).toBe(true);

    await act(async () => {
      await result.current.confirmDeleteAll();
    });

    expect(handleDeleteAll).toHaveBeenCalledTimes(1);
    expect(result.current.showDeleteConfirm).toBe(false);
  });

  it('ログアウト時に検索条件と削除確認状態をリセットする', () => {
    let logoutCallback: LogoutCallback | null = null;

    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({
        onLogoutCapture: (cb) => {
          logoutCallback = cb;
        },
      })
    );
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        dbData: [makeRow()],
        hasDbData: true,
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    act(() => {
      result.current.utilityRailProps.searchCardProps.onSearch('7203');
      result.current.utilityRailProps.actionRailProps.onDeleteRequest();
    });

    expect(result.current.utilityRailProps.searchCardProps.value).toBe('7203');
    expect(result.current.showDeleteConfirm).toBe(true);

    act(() => {
      logoutCallback?.();
    });

    expect(result.current.utilityRailProps.searchCardProps.value).toBe('');
    expect(result.current.showDeleteConfirm).toBe(false);
  });

  it('facetsがある場合は検索候補にfacets由来のsecuritiesを使う', () => {
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        dbData: [makeRow()],
        hasDbData: true,
        facets: {
          securities: [{ value: '9984', label: '9984: ソフトバンクグループ', count: 1 }],
        },
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    expect(result.current.utilityRailProps.searchCardProps.categories.securities).toEqual([
      { value: '9984', label: '9984: ソフトバンクグループ' },
    ]);
  });

  it('facetsがない場合は表示データからcreateSearchOptionsで検索候補を作る', () => {
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        dbData: [makeRow()],
        hasDbData: true,
        facets: undefined,
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    expect(result.current.utilityRailProps.searchCardProps.categories.securities).toEqual([
      { value: '7203', label: '7203: トヨタ自動車' },
    ]);
  });

  it('CSVプレビュー中はfacetsがあってもcreateSearchOptions由来の候補を使う', () => {
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        previewRows: [makeRow({ security_code: '4568', security_name: 'プレビュー銘柄' })],
        hasCsvFile: true,
        facets: {
          securities: [{ value: '9984', label: '9984: ソフトバンクグループ', count: 1 }],
        },
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    expect(result.current.utilityRailProps.searchCardProps.categories.securities).toEqual([
      { value: '4568', label: '4568: プレビュー銘柄' },
    ]);
  });

  it('summaryがある場合はportfolioSummaryとして返す（未検索・非プレビュー時）', () => {
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        dbData: [makeRow()],
        hasDbData: true,
        summary: {
          total_purchase_amount: 250000,
          total_market_value: 260000,
          total_daily_change: 50,
        },
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    expect(result.current.portfolioSummary).toEqual({
      total_purchase_amount: 250000,
      total_market_value: 260000,
      total_daily_change: 50,
    });
  });

  it('検索中はsummaryがあってもportfolioSummaryはundefinedになる', () => {
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        dbData: [makeRow()],
        hasDbData: true,
        summary: {
          total_purchase_amount: 250000,
          total_market_value: 260000,
          total_daily_change: 50,
        },
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    act(() => {
      result.current.utilityRailProps.searchCardProps.onSearch('7203');
    });

    expect(result.current.portfolioSummary).toBeUndefined();
  });

  it('CSVプレビュー中はsummaryがあってもportfolioSummaryはundefinedになる', () => {
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        previewRows: [makeRow()],
        hasCsvFile: true,
        summary: {
          total_purchase_amount: 250000,
          total_market_value: 260000,
          total_daily_change: 50,
        },
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    expect(result.current.portfolioSummary).toBeUndefined();
  });

  it('deleteLabel と dbDataCount は API total を使う', () => {
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        dbData: [makeRow()],
        hasDbData: true,
        dbTotal: 1001,
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    expect(result.current.utilityRailProps.actionRailProps.deleteLabel).toBe('全件削除 (1001件)');
    expect(result.current.dbDataCount).toBe(1001);
  });

  it('dbTotalが上限を超える場合は件数上限の警告を表示する', () => {
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        dbData: [makeRow()],
        hasDbData: true,
        dbTotal: 1001,
        assetBalanceListLimit: 1000,
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    expect(result.current.utilityRailProps.warning).toBe(
      '一覧は最大1000件まで表示しています。未表示の銘柄がある可能性があります。'
    );
  });

  it('dbTotalが上限以下の場合は警告を表示しない', () => {
    vi.mocked(dataSourceHook.useAssetBalanceDataSourceCore).mockReturnValue(
      makeDataSourceMock({
        dbData: [makeRow()],
        hasDbData: true,
        dbTotal: 1,
        assetBalanceListLimit: 1000,
      })
    );

    const { result } = renderHook(() => useAssetBalanceState());

    expect(result.current.utilityRailProps.warning).toBeNull();
  });
});
