/**
 * useReceiptsData: 認証境界・キャッシュ境界テスト
 *
 * ReceiptsPage を経由しない単独利用として、
 * 未認証時のフェッチ抑止とログアウト時のキャッシュ削除を検証する
 */
import { renderHook, act, waitFor } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { useReceiptsData } from '../useReceiptsData';
import { receiptQueryKeys } from '../../queryKeys';
import * as receiptApiModule from '@/features/receipt/api/receiptApi';
import * as authHook from '@/features/auth/hooks/useAuth';

// ────────────────────────────────────────────────────────
// モック
// ────────────────────────────────────────────────────────

vi.mock('@/features/receipt/api/receiptApi', () => ({
  dividendApi: {
    list: vi.fn().mockResolvedValue({ data: [], total: 0, page: 1, per_page: 1000 }),
    deleteAll: vi.fn().mockResolvedValue({}),
    uploadCsv: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0, errors: [] }),
    previewCsv: vi.fn().mockResolvedValue({ total_rows: 0, valid_rows: 0, errors: [], rows: [] }),
  },
  domesticStockApi: {
    list: vi.fn().mockResolvedValue({ data: [], total: 0, page: 1, per_page: 1000 }),
    deleteAll: vi.fn().mockResolvedValue({}),
    uploadCsv: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0, errors: [] }),
    previewCsv: vi.fn().mockResolvedValue({ total_rows: 0, valid_rows: 0, errors: [], rows: [] }),
  },
  mutualfundApi: {
    list: vi.fn().mockResolvedValue({ data: [], total: 0, page: 1, per_page: 1000 }),
    deleteAll: vi.fn().mockResolvedValue({}),
    uploadCsv: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0, errors: [] }),
    previewCsv: vi.fn().mockResolvedValue({ total_rows: 0, valid_rows: 0, errors: [], rows: [] }),
  },
}));

vi.mock('@/features/receipt/parsers', () => ({
  transformDBDividend: vi.fn((item: unknown) => item),
  transformDBDomesticStock: vi.fn((item: unknown) => item),
  transformDBMutualfund: vi.fn((item: unknown) => item),
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

// 3ドメインの API モジュールは同一シグネチャのためテーブルで切り替える。
// 実装側も receiptApiByType ルックアップ1本で同一コードパス（useReceiptsData.ts:79-82）。
const apiMocksByType = {
  dividend: receiptApiModule.dividendApi,
  domesticstock: receiptApiModule.domesticStockApi,
  mutualfund: receiptApiModule.mutualfundApi,
};

describe('useReceiptsData: 認証境界・キャッシュ境界', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(receiptApiModule.dividendApi.list).mockResolvedValue({ data: [], total: 0, page: 1, per_page: 200 } as never);
    vi.mocked(receiptApiModule.dividendApi.deleteAll).mockResolvedValue({} as never);
    vi.mocked(receiptApiModule.dividendApi.uploadCsv).mockResolvedValue({
      inserted: 0,
      skipped: 0,
      errors: [],
    } as never);
    vi.mocked(receiptApiModule.dividendApi.previewCsv).mockResolvedValue({
      total_rows: 0,
      valid_rows: 0,
      errors: [],
      rows: [],
    } as never);

    vi.mocked(receiptApiModule.domesticStockApi.list).mockResolvedValue({ data: [], total: 0, page: 1, per_page: 200 } as never);
    vi.mocked(receiptApiModule.domesticStockApi.deleteAll).mockResolvedValue({} as never);
    vi.mocked(receiptApiModule.domesticStockApi.uploadCsv).mockResolvedValue({
      inserted: 0,
      skipped: 0,
      errors: [],
    } as never);
    vi.mocked(receiptApiModule.domesticStockApi.previewCsv).mockResolvedValue({
      total_rows: 0,
      valid_rows: 0,
      errors: [],
      rows: [],
    } as never);

    vi.mocked(receiptApiModule.mutualfundApi.list).mockResolvedValue({ data: [], total: 0, page: 1, per_page: 200 } as never);
    vi.mocked(receiptApiModule.mutualfundApi.deleteAll).mockResolvedValue({} as never);
    vi.mocked(receiptApiModule.mutualfundApi.uploadCsv).mockResolvedValue({
      inserted: 0,
      skipped: 0,
      errors: [],
    } as never);
    vi.mocked(receiptApiModule.mutualfundApi.previewCsv).mockResolvedValue({
      total_rows: 0,
      valid_rows: 0,
      errors: [],
      rows: [],
    } as never);
  });

  it('未認証時: API フェッチが行われない', async () => {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: false }));

    renderHook(() => useReceiptsData(), { wrapper: makeWrapper(qc) });

    // enabled: false のため queryFn は実行されない（非同期キューを flush しても呼ばれない）
    await act(async () => { await Promise.resolve(); });
    expect(receiptApiModule.dividendApi.list).not.toHaveBeenCalled();
    expect(receiptApiModule.domesticStockApi.list).not.toHaveBeenCalled();
    expect(receiptApiModule.mutualfundApi.list).not.toHaveBeenCalled();
  });

  it('deleteAll 実行中ログアウト: mutation 完了後もキャッシュが再生成されない', async () => {
    // deleteAll の完了を手動制御するための Promise
    let resolveDeleteAll!: () => void;
    vi.mocked(receiptApiModule.dividendApi.deleteAll).mockReturnValue(
      new Promise<void>((resolve) => { resolveDeleteAll = resolve; }) as never
    );

    const capturedCallbacks: LogoutCallback[] = [];
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ onLogoutCapture: (cb) => { capturedCallbacks.push(cb); } })
    );
    vi.mocked(receiptApiModule.dividendApi.list).mockResolvedValue({
      data: [{ id: '1', payment_date: '2023-01-01' } as never],
      total: 1, page: 1, per_page: 200,
    } as never);

    const { result } = renderHook(() => useReceiptsData(), { wrapper: makeWrapper(qc) });

    // データがキャッシュに入り、onLogout が登録されるまで待つ
    await waitFor(() => {
      expect(qc.getQueryData(receiptQueryKeys.dividend('user-1'))).toBeDefined();
    });
    await waitFor(() => expect(capturedCallbacks.length).toBeGreaterThan(0));

    // deleteAll を開始（まだ完了しない）
    act(() => { result.current.deleteAll('dividend'); });

    // ログアウト実行（キャッシュをクリア）
    act(() => {
      capturedCallbacks.forEach(cb => cb());
      vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: false }));
    });

    // キャッシュがクリアされたことを確認
    await waitFor(() => {
      expect(qc.getQueryData(receiptQueryKeys.dividend('user-1'))).toBeUndefined();
    });

    // deleteAll を完了させる（onSuccess が setQueryData を試みる）
    await act(async () => { resolveDeleteAll(); });

    // getQueryState ガードにより キャッシュが再生成されていないことを確認
    expect(qc.getQueryData(receiptQueryKeys.dividend('user-1'))).toBeUndefined();
  });

  it('onLogout コールバック実行で receipts キャッシュが除去される', async () => {
    let capturedCallback: LogoutCallback | null = null;
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ onLogoutCapture: (cb) => { capturedCallback = cb; } })
    );
    vi.mocked(receiptApiModule.dividendApi.list).mockResolvedValue({
      data: [{ id: '1', payment_date: '2023-01-01' } as never],
      total: 1, page: 1, per_page: 200,
    } as never);

    renderHook(() => useReceiptsData(), { wrapper: makeWrapper(qc) });

    // キャッシュにデータが入るまで待つ
    await waitFor(() => {
      expect(qc.getQueryData(receiptQueryKeys.dividend('user-1'))).toBeDefined();
    });
    await waitFor(() => expect(capturedCallback).not.toBeNull());

    act(() => {
      capturedCallback!();
      vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: false }));
    });

    await waitFor(() => {
      expect(qc.getQueryData(receiptQueryKeys.dividend('user-1'))).toBeUndefined();
    });
    expect(qc.getQueryData(receiptQueryKeys.domesticstock('user-1'))).toBeUndefined();
    expect(qc.getQueryData(receiptQueryKeys.mutualfund('user-1'))).toBeUndefined();
  });

  // uploadCsv は3ドメインとも同一コードパスのためパラメータ化する
  it.each(['dividend', 'domesticstock', 'mutualfund'] as const)(
    'uploadCsv mutation: %s で invalidate と onSuccess が実行される',
    async (type) => {
      const qc = new QueryClient({
        defaultOptions: { queries: { retry: false, staleTime: Infinity } },
      });
      const invalidateQueriesSpy = vi.spyOn(qc, 'invalidateQueries');
      const onSuccess = vi.fn();
      const api = apiMocksByType[type];

      vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: true }));
      vi.mocked(api.uploadCsv).mockResolvedValue({
        inserted: 2,
        skipped: 1,
        errors: [],
      } as never);

      const { result } = renderHook(() => useReceiptsData(), { wrapper: makeWrapper(qc) });

      await waitFor(() => {
        expect(qc.getQueryData(receiptQueryKeys[type]('user-1'))).toEqual({ items: [], summary: undefined });
      });

      act(() => {
        result.current.uploadCsv({
          type,
          file: new File([''], `${type}.csv`),
          onSuccess,
        });
      });

      await waitFor(() => {
        expect(vi.mocked(api.uploadCsv)).toHaveBeenCalledTimes(1);
      });
      await waitFor(() => {
        expect(onSuccess).toHaveBeenCalledWith({ inserted: 2, skipped: 1, errors: [] });
      });
      await waitFor(() => {
        expect(invalidateQueriesSpy).toHaveBeenCalledWith({
          queryKey: receiptQueryKeys[type]('user-1'),
        });
      });
      await waitFor(() => {
        expect(vi.mocked(api.list)).toHaveBeenCalledTimes(2);
      });
    }
  );

  // total > per_page の場合も3ドメインとも同一コードパスのためパラメータ化する
  it.each(['dividend', 'domesticstock', 'mutualfund'] as const)(
    '%s: total が per_page を超えても API 検索では次ページを取得しない',
    async (type) => {
      const qc = new QueryClient({
        defaultOptions: { queries: { retry: false, staleTime: Infinity } },
      });

      vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: true }));
      const api = apiMocksByType[type];
      vi.mocked(api.list).mockImplementation(
        ({ page = 1 }: { page?: number; per_page?: number } = {}) => {
          if (page === 1) {
            return Promise.resolve({
              data: Array.from({ length: 1000 }, (_, index) =>
                ({ id: `${type}-${index + 1}` }) as never
              ),
              total: 1001,
              page: 1,
              per_page: 1000,
            } as never);
          }
          return Promise.resolve({
            data: [{ id: `${type}-1001` } as never],
            total: 1001,
            page: 2,
            per_page: 1000,
          } as never);
        }
      );

      const { result } = renderHook(() => useReceiptsData(), { wrapper: makeWrapper(qc) });

      await waitFor(() => {
        expect(result.current.data[type]).toHaveLength(1000);
      });
      expect(vi.mocked(api.list)).toHaveBeenCalledTimes(1);
      expect(vi.mocked(api.list)).toHaveBeenNthCalledWith(1, { per_page: 1000, page: 1, include_summary: true });
      expect(vi.mocked(api.list)).not.toHaveBeenCalledWith({ per_page: 1000, page: 2 });
    }
  );

  it('deleteAll mutation: domesticstock と mutualfund を削除できる', async () => {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: true }));

    const { result } = renderHook(() => useReceiptsData(), { wrapper: makeWrapper(qc) });

    act(() => {
      result.current.deleteAll('domesticstock');
    });
    await waitFor(() => {
      expect(receiptApiModule.domesticStockApi.deleteAll).toHaveBeenCalledTimes(1);
    });

    act(() => {
      result.current.deleteAll('mutualfund');
    });
    await waitFor(() => {
      expect(receiptApiModule.mutualfundApi.deleteAll).toHaveBeenCalledTimes(1);
    });
  });

  // previewCsv も3ドメインとも同一コードパスのためパラメータ化する
  it.each(['dividend', 'domesticstock', 'mutualfund'] as const)(
    'previewCsv mutation: %s で整形済み結果を返す',
    async (type) => {
      const qc = new QueryClient({
        defaultOptions: { queries: { retry: false, staleTime: Infinity } },
      });
      const onSuccess = vi.fn();
      const file = new File([''], `${type}.csv`);
      const api = apiMocksByType[type];

      vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: true }));
      vi.mocked(api.previewCsv).mockResolvedValue({
        total_rows: 2,
        valid_rows: 1,
        errors: [{ row: 2, message: 'invalid' }],
        rows: [{ id: `${type}-1` }],
      } as never);

      const { result } = renderHook(() => useReceiptsData(), { wrapper: makeWrapper(qc) });

      act(() => {
        result.current.previewCsv({ type, file, onSuccess });
      });

      await waitFor(() => {
        expect(vi.mocked(api.previewCsv)).toHaveBeenCalledWith(file);
      });
      await waitFor(() => {
        expect(onSuccess).toHaveBeenCalledWith({
          totalRows: 2,
          validRows: 1,
          errors: [{ row: 2, message: 'invalid' }],
          rows: [{ id: `${type}-1` }],
        });
      });
    }
  );

  it('previewCsv mutation: エラー時に onError が呼ばれる', async () => {    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });
    const onError = vi.fn();

    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: true }));
    vi.mocked(receiptApiModule.dividendApi.previewCsv).mockRejectedValue(new Error('解析失敗'));

    const { result } = renderHook(() => useReceiptsData(), { wrapper: makeWrapper(qc) });

    act(() => {
      result.current.previewCsv({
        type: 'dividend',
        file: new File([''], 'test.csv'),
        onError,
      });
    });

    await waitFor(() => {
      expect(onError).toHaveBeenCalledWith('解析失敗');
    });
  });

  it('uploadCsv mutation エラー時: dbError に反映される', async () => {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: true }));
    vi.mocked(receiptApiModule.dividendApi.uploadCsv).mockRejectedValue(new Error('アップロード失敗'));

    const { result } = renderHook(() => useReceiptsData(), { wrapper: makeWrapper(qc) });

    act(() => {
      result.current.uploadCsv({
        type: 'dividend',
        file: new File([''], 'test.csv'),
      });
    });

    await waitFor(() => {
      expect(result.current.dbError).toBe('アップロード失敗');
    });
  });
});

describe('useReceiptsData: 選択中タブの優先取得', () => {
  function deferred<T>() {
    let resolve!: (value: T) => void;
    let reject!: (reason?: unknown) => void;
    const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
    return { promise, resolve, reject };
  }

  function makeEmptyList() {
    return { data: [], total: 0, page: 1, per_page: 200 };
  }

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(receiptApiModule.dividendApi.list).mockResolvedValue(makeEmptyList() as never);
    vi.mocked(receiptApiModule.domesticStockApi.list).mockResolvedValue(makeEmptyList() as never);
    vi.mocked(receiptApiModule.mutualfundApi.list).mockResolvedValue(makeEmptyList() as never);
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: true }));
  });

  function makeTestQueryClient() {
    return new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });
  }

  it('選択中タブのみ先行取得し、完了後に他タブをバックグラウンド取得する', async () => {
    const dividendGate = deferred<unknown>();
    vi.mocked(receiptApiModule.dividendApi.list).mockReturnValue(dividendGate.promise as never);

    const { result } = renderHook(() => useReceiptsData('dividend'), {
      wrapper: makeWrapper(makeTestQueryClient()),
    });

    // 選択中タブの取得は開始される
    await waitFor(() => {
      expect(receiptApiModule.dividendApi.list).toHaveBeenCalledTimes(1);
    });
    // 他タブは選択中タブの完了まで取得されない
    expect(receiptApiModule.domesticStockApi.list).not.toHaveBeenCalled();
    expect(receiptApiModule.mutualfundApi.list).not.toHaveBeenCalled();
    expect(result.current.loadingByTab.dividend).toBe(true);
    expect(result.current.dbLoading).toBe(true);

    // 選択中タブの取得が完了すると他タブがバックグラウンド取得される
    await act(async () => {
      dividendGate.resolve(makeEmptyList());
    });
    await waitFor(() => {
      expect(receiptApiModule.domesticStockApi.list).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(receiptApiModule.mutualfundApi.list).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(result.current.loadingByTab.dividend).toBe(false);
    });
    expect(result.current.dbLoading).toBe(false);
  });

  it('タブ切替で未取得タブの取得が開始される', async () => {
    const dividendGate = deferred<unknown>();
    vi.mocked(receiptApiModule.dividendApi.list).mockReturnValue(dividendGate.promise as never);

    const { result, rerender } = renderHook(
      ({ tab }: { tab: 'dividend' | 'domesticstock' | 'mutualfund' }) => useReceiptsData(tab),
      {
        wrapper: makeWrapper(makeTestQueryClient()),
        initialProps: { tab: 'dividend' as 'dividend' | 'domesticstock' | 'mutualfund' },
      },
    );

    await waitFor(() => {
      expect(receiptApiModule.dividendApi.list).toHaveBeenCalledTimes(1);
    });
    // dividend 完了前は他タブを取得しない
    expect(receiptApiModule.domesticStockApi.list).not.toHaveBeenCalled();

    // 未取得タブへ切り替えるとそのタブの取得が開始される
    rerender({ tab: 'domesticstock' });
    await waitFor(() => {
      expect(receiptApiModule.domesticStockApi.list).toHaveBeenCalledTimes(1);
    });
    expect(result.current.loadingByTab.domesticstock).toBe(true);
    // 切替元の取得は中断されず継続する
    expect(receiptApiModule.dividendApi.list).toHaveBeenCalledTimes(1);

    await act(async () => {
      dividendGate.resolve(makeEmptyList());
    });
    await waitFor(() => {
      expect(result.current.loadingByTab.dividend).toBe(false);
    });
    await waitFor(() => {
      expect(result.current.loadingByTab.domesticstock).toBe(false);
    });
  });

  it('activeTab 省略時は従来通り3タブとも同時に取得する', async () => {
    renderHook(() => useReceiptsData(), {
      wrapper: makeWrapper(makeTestQueryClient()),
    });

    await waitFor(() => {
      expect(receiptApiModule.dividendApi.list).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(receiptApiModule.domesticStockApi.list).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(receiptApiModule.mutualfundApi.list).toHaveBeenCalledTimes(1);
    });
  });
});
