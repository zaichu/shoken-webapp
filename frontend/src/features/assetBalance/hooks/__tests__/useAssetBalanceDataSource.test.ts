/**
 * useAssetBalanceDataSource: 認証競合・キャッシュ境界テスト
 *
 * - deleteAll 実行中ログアウトが発生しても、mutation 完了後にキャッシュが再生成されないことを検証
 * - uploadCsv 実行中ユーザー変更が発生しても、snapshotUserId キーで invalidateQueries されることを検証
 */
import { renderHook, act, waitFor } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { useAssetBalanceDataSource } from '../useAssetBalanceDataSource';
import { assetBalanceQueryKeys } from '../../queryKeys';
import * as assetBalanceApiModule from '@/features/assetBalance/api/assetBalanceApi';
import * as authHook from '@/features/auth/hooks/useAuth';

// ────────────────────────────────────────────────────────
// モック
// ────────────────────────────────────────────────────────

vi.mock('@/features/assetBalance/api/assetBalanceApi', () => ({
  assetBalanceApi: {
    list: vi.fn().mockResolvedValue([]),
    previewCsv: vi.fn().mockResolvedValue({ total_rows: 0, valid_rows: 0, errors: [], rows: [] }),
    uploadCsv: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0, errors: [] }),
    deleteAll: vi.fn().mockResolvedValue({}),
  },
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

describe('useAssetBalanceDataSource: 基本動作', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));
  });

  it('未認証時はhandleSaveToDBが早期リターンする', async () => {
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: false }));

    const { result } = renderHook(() => useAssetBalanceDataSource(), { wrapper: makeWrapper(qc) });

    act(() => { result.current.handleSaveToDB(); });

    expect(assetBalanceApiModule.assetBalanceApi.uploadCsv).not.toHaveBeenCalled();
  });

  it('未認証時はhandleDeleteAllが早期リターンする', async () => {
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: false }));

    const { result } = renderHook(() => useAssetBalanceDataSource(), { wrapper: makeWrapper(qc) });

    await act(async () => { await result.current.handleDeleteAll(); });

    expect(assetBalanceApiModule.assetBalanceApi.deleteAll).not.toHaveBeenCalled();
  });

  it('rawFileなしではhandleSaveToDBが早期リターンする', () => {
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });

    const { result } = renderHook(() => useAssetBalanceDataSource(), { wrapper: makeWrapper(qc) });

    // rawFile=null（初期状態）でhanleSaveToDB呼び出し
    act(() => { result.current.handleSaveToDB(); });

    expect(assetBalanceApiModule.assetBalanceApi.uploadCsv).not.toHaveBeenCalled();
  });

  it('previewMutationエラー時にerrorが設定される', async () => {
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    vi.mocked(assetBalanceApiModule.assetBalanceApi.previewCsv).mockRejectedValueOnce(new Error('プレビュー失敗'));

    const { result } = renderHook(() => useAssetBalanceDataSource(), { wrapper: makeWrapper(qc) });

    await act(async () => {
      result.current.handleFileSelect(new File([], 'test.csv'));
    });

    await waitFor(() => {
      expect(result.current.error).toBeTruthy();
    });
  });

  it('deleteAll完了後にcacheキーが存在しない場合setQueryDataを呼ばない', async () => {
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false, staleTime: Infinity } } });
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));
    // DBデータを読み込まない（queryStateがundefined）ままdeleteAllを呼ぶ
    vi.mocked(assetBalanceApiModule.assetBalanceApi.deleteAll).mockResolvedValue({} as never);

    const { result } = renderHook(() => useAssetBalanceDataSource(), { wrapper: makeWrapper(qc) });

    // クエリが実行される前にキャッシュを削除
    qc.removeQueries();

    await act(async () => { await result.current.handleDeleteAll(); });

    // エラーなく完了することを確認
    expect(result.current.error).toBeNull();
  });
});

describe('useAssetBalanceDataSource: キャッシュ境界', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('deleteAll 実行中ログアウト: mutation 完了後もキャッシュが再生成されない', async () => {
    // deleteAll の完了を手動制御するための Promise
    let resolveDeleteAll!: () => void;
    vi.mocked(assetBalanceApiModule.assetBalanceApi.deleteAll).mockReturnValue(
      new Promise<void>((resolve) => { resolveDeleteAll = resolve; }) as never
    );

    const capturedCallbacks: LogoutCallback[] = [];
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });

    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ onLogoutCapture: (cb) => { capturedCallbacks.push(cb); } })
    );
    vi.mocked(assetBalanceApiModule.assetBalanceApi.list).mockResolvedValue({
      data: [{ security_code: '7203', security_name: 'トヨタ自動車', shares: 100 } as never],
      total: 1, page: 1, per_page: 200,
    });

    const { result } = renderHook(
      () => useAssetBalanceDataSource(),
      { wrapper: makeWrapper(qc) }
    );

    // データがキャッシュに入り、onLogout が登録されるまで待つ
    await waitFor(() => {
      expect(qc.getQueryData(assetBalanceQueryKeys.all('user-1'))).toBeDefined();
    });
    await waitFor(() => expect(capturedCallbacks.length).toBeGreaterThan(0));

    // deleteAll を開始（まだ完了しない）
    act(() => { void result.current.handleDeleteAll(); });

    // ログアウト実行（キャッシュをクリア）
    act(() => {
      capturedCallbacks.forEach(cb => cb());
      vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: false }));
    });

    // キャッシュがクリアされたことを確認
    await waitFor(() => {
      expect(qc.getQueryData(assetBalanceQueryKeys.all('user-1'))).toBeUndefined();
    });

    // deleteAll を完了させる（onSuccess が setQueryData を試みる）
    await act(async () => { resolveDeleteAll(); });

    // getQueryState ガードにより キャッシュが再生成されていないことを確認
    expect(qc.getQueryData(assetBalanceQueryKeys.all('user-1'))).toBeUndefined();
  });

  it('uploadCsv 実行中ユーザー変更: snapshotUserId キーで invalidateQueries される', async () => {
    // uploadCsv の完了を手動制御するための Promise
    let resolveUploadCsv!: () => void;
    vi.mocked(assetBalanceApiModule.assetBalanceApi.uploadCsv).mockReturnValue(
      new Promise<void>((resolve) => { resolveUploadCsv = resolve; }) as never
    );

    // previewCsv が非空データを返すように設定して rawFile を確保する
    vi.mocked(assetBalanceApiModule.assetBalanceApi.previewCsv).mockResolvedValue({
      total_rows: 1,
      valid_rows: 1,
      errors: [],
      rows: [{ security_code: '7203', security_name: 'トヨタ自動車', shares: 100 } as never],
    });

    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });
    const invalidateSpy = vi.spyOn(qc, 'invalidateQueries');

    // mockImplementation で可変状態から返すことで rerender 後の userId を切り替えられる
    let currentAuthMock = makeAuthMock({ userId: 'user-1' });
    vi.mocked(authHook.useAuth).mockImplementation(() => currentAuthMock);

    const { result, rerender } = renderHook(
      () => useAssetBalanceDataSource(),
      { wrapper: makeWrapper(qc) }
    );

    // handleFileSelect で rawFile を設定し、プレビューを取得する
    await act(async () => {
      await result.current.handleFileSelect(new File([], 'test.csv'));
    });

    // rawFile が設定されるまで待つ
    await waitFor(() => expect(result.current.hasCsvFile).toBe(true));

    // uploadCsv を開始（まだ完了しない）
    act(() => { void result.current.handleSaveToDB(); });

    // user-2 に切り替えて rerender し、フック内の userId を更新する（再ログイン相当）
    act(() => {
      currentAuthMock = makeAuthMock({ userId: 'user-2' });
      rerender();
    });

    // uploadCsv を完了させる（onSuccess が snapshotUserId で動作するか確認）
    await act(async () => { resolveUploadCsv(); });

    // snapshotUserId（user-1）キーで invalidateQueries が呼ばれたことを確認
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: assetBalanceQueryKeys.all('user-1') });
    // user-2 キーでは呼ばれていないことを確認
    expect(invalidateSpy).not.toHaveBeenCalledWith({ queryKey: assetBalanceQueryKeys.all('user-2') });
  });
});
