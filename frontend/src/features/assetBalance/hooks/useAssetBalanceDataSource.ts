import { useState, useEffect, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { AssetBalanceData } from '@/types/api';
import { assetBalanceApi } from '@/features/assetBalance/api/assetBalanceApi';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';
import { assetBalanceQueryKeys, clearAssetBalanceCache } from '../queryKeys';
import type { CsvUploadResult } from '@/lib/csvImport';

type AssetBalanceListResult = Awaited<ReturnType<typeof assetBalanceApi.list>>;
export type AssetBalanceSummary = NonNullable<AssetBalanceListResult['summary']>;
export type AssetBalanceSearchFacets = NonNullable<AssetBalanceListResult['facets']>;

// 一覧取得は 1 ページに寄せて件数上限を明示する（受領明細と同等の方針）
export const ASSET_BALANCE_LIST_LIMIT = 1000;

interface AssetBalanceDataSourceAuthContext {
  isAuthenticated: boolean;
  authLoading: boolean;
  onLogout: ReturnType<typeof useAuth>['onLogout'];
  userId: string;
  registerLogoutReset?: boolean;
}

export interface UseAssetBalanceDataSourceResult {
  // データ
  dbData: AssetBalanceData[];
  dbTotal: number;
  summary: AssetBalanceSummary | undefined;
  facets: AssetBalanceSearchFacets | undefined;
  assetBalanceListLimit: number;
  previewRows: AssetBalanceData[];
  // 状態
  loading: boolean;
  error: string | null;
  saving: boolean;
  deleting: boolean;
  previewing: boolean;
  lastSavedResult: CsvUploadResult | null;
  // フラグ
  hasCsvFile: boolean;
  hasDbData: boolean;
  csvFileName: string | null;
  // アクション
  handleFileSelect: (file: File) => void;
  handleSaveToDB: () => void;
  handleDeleteAll: () => Promise<void>;
  resetState: () => void;
}

/**
 * 保有銘柄データソースを TanStack Query で管理するフック
 * ファイル選択時にバックエンドプレビューAPIを呼び出し、行データを取得する
 */
export function useAssetBalanceDataSourceCore({
  isAuthenticated,
  authLoading,
  onLogout,
  userId,
  registerLogoutReset = true,
}: AssetBalanceDataSourceAuthContext): UseAssetBalanceDataSourceResult {
  const queryClient = useQueryClient();

  const [rawFile, setRawFile] = useState<File | null>(null);
  const [previewRows, setPreviewRows] = useState<AssetBalanceData[]>([]);
  const [lastSavedResult, setLastSavedResult] = useState<CsvUploadResult | null>(null);

  const dbQuery = useQuery({
    queryKey: assetBalanceQueryKeys.list(userId),
    queryFn: () =>
      assetBalanceApi.list({
        page: 1,
        per_page: ASSET_BALANCE_LIST_LIMIT,
        include_summary: true,
        include_facets: true,
      }),
    enabled: isAuthenticated && !authLoading && !!userId,
  });

  const previewMutation = useMutation({
    mutationFn: (file: File) => assetBalanceApi.previewCsv(file),
    onSuccess: (previewResult) => {
      setPreviewRows(previewResult.rows as unknown as AssetBalanceData[]);
    },
  });

  const uploadCsvMutation = useMutation({
    mutationFn: (file: File) => assetBalanceApi.uploadCsv(file),
    onMutate: () => ({ snapshotUserId: userId }),
    onSuccess: (uploadResult, _, context) => {
      setRawFile(null);
      setPreviewRows([]);
      setLastSavedResult(uploadResult ?? null);
      const targetUserId = context?.snapshotUserId ?? userId;
      // all は useAssetBalance（DividendInfo 用）が参照するキャッシュのため合わせて無効化する
      queryClient.invalidateQueries({ queryKey: assetBalanceQueryKeys.all(targetUserId) });
      queryClient.invalidateQueries({ queryKey: assetBalanceQueryKeys.list(targetUserId) });
    },
  });

  const deleteAllMutation = useMutation({
    mutationFn: () => assetBalanceApi.deleteAll(),
    onMutate: () => ({ snapshotUserId: userId }),
    onSuccess: (_, __, context) => {
      const targetUserId = context?.snapshotUserId ?? userId;

      const listKey = assetBalanceQueryKeys.list(targetUserId);
      if (queryClient.getQueryState(listKey) !== undefined) {
        queryClient.setQueryData(listKey, {
          data: [],
          total: 0,
          page: 1,
          per_page: ASSET_BALANCE_LIST_LIMIT,
        });
      }

      const allKey = assetBalanceQueryKeys.all(targetUserId);
      if (queryClient.getQueryState(allKey) !== undefined) {
        queryClient.setQueryData(allKey, []);
      }

      setLastSavedResult(null);
    },
  });

  const resetState = useCallback(() => {
    clearAssetBalanceCache(queryClient);
    previewMutation.reset();
    uploadCsvMutation.reset();
    deleteAllMutation.reset();
    setRawFile(null);
    setPreviewRows([]);
    setLastSavedResult(null);
  }, [deleteAllMutation, previewMutation, queryClient, uploadCsvMutation]);

  useEffect(() => {
    if (!registerLogoutReset) {
      return;
    }

    return onLogout(() => {
      resetState();
    });
  }, [onLogout, registerLogoutReset, resetState]);

  const handleFileSelect = useCallback((file: File) => {
    setRawFile(file);
    setPreviewRows([]);
    setLastSavedResult(null);
    previewMutation.mutate(file);
  }, [previewMutation]);

  const handleSaveToDB = useCallback(() => {
    if (!isAuthenticated || rawFile === null || previewRows.length === 0) return;
    uploadCsvMutation.mutate(rawFile);
  }, [uploadCsvMutation, isAuthenticated, rawFile, previewRows.length]);

  const handleDeleteAll = useCallback(async () => {
    if (!isAuthenticated) return;
    await deleteAllMutation.mutateAsync();
  }, [deleteAllMutation, isAuthenticated]);

  const dbData = dbQuery.data?.data ?? [];
  const dbTotal = dbQuery.data?.total ?? dbData.length;
  const summary = dbQuery.data?.summary;
  const facets = dbQuery.data?.facets;
  const queryError = dbQuery.error;
  const mutationError = uploadCsvMutation.error ?? deleteAllMutation.error ?? previewMutation.error;
  const error = queryError
    ? getDisplayErrorMessage(queryError, 'データ取得に失敗しました')
    : mutationError
      ? getDisplayErrorMessage(mutationError, '操作に失敗しました')
      : null;

  return {
    dbData,
    dbTotal,
    summary,
    facets,
    assetBalanceListLimit: ASSET_BALANCE_LIST_LIMIT,
    previewRows,
    loading: dbQuery.isFetching,
    error,
    saving: uploadCsvMutation.isPending,
    deleting: deleteAllMutation.isPending,
    previewing: previewMutation.isPending,
    lastSavedResult,
    hasCsvFile: rawFile !== null,
    hasDbData: dbData.length > 0,
    csvFileName: rawFile?.name ?? null,
    handleFileSelect,
    handleSaveToDB,
    handleDeleteAll,
    resetState,
  };
}

export function useAssetBalanceDataSource(): UseAssetBalanceDataSourceResult {
  const { isAuthenticated, isLoading: authLoading, onLogout, user } = useAuth();

  return useAssetBalanceDataSourceCore({
    isAuthenticated,
    authLoading,
    onLogout,
    userId: user?.id ?? '',
  });
}
