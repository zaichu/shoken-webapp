import { useState, useEffect, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { AssetBalanceData } from '@/types/api';
import { assetBalanceApi } from '@/features/assetBalance/api/assetBalanceApi';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';
import { assetBalanceQueryKeys, clearAssetBalanceCache } from '../queryKeys';
import type { CsvUploadResult } from '@/lib/csvImport';

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
    queryKey: assetBalanceQueryKeys.all(userId),
    queryFn: () => assetBalanceApi.list(),
    enabled: isAuthenticated && !authLoading && !!userId,
  });

  const previewMutation = useMutation({
    mutationFn: (file: File) => assetBalanceApi.previewCsv(file),
    onSuccess: (data) => {
      setPreviewRows(data.rows as unknown as AssetBalanceData[]);
    },
  });

  const uploadCsvMutation = useMutation({
    mutationFn: (file: File) => assetBalanceApi.uploadCsv(file),
    onMutate: () => ({ snapshotUserId: userId }),
    onSuccess: (data, _, context) => {
      setRawFile(null);
      setPreviewRows([]);
      setLastSavedResult(data ?? null);
      queryClient.invalidateQueries({ queryKey: assetBalanceQueryKeys.all(context?.snapshotUserId ?? userId) });
    },
  });

  const deleteAllMutation = useMutation({
    mutationFn: () => assetBalanceApi.deleteAll(),
    onMutate: () => ({ snapshotUserId: userId }),
    onSuccess: (_, __, context) => {
      const key = assetBalanceQueryKeys.all(context?.snapshotUserId ?? userId);
      if (queryClient.getQueryState(key) !== undefined) {
        queryClient.setQueryData(key, []);
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

  const dbData = dbQuery.data ?? [];
  const queryError = dbQuery.error;
  const mutationError = uploadCsvMutation.error ?? deleteAllMutation.error ?? previewMutation.error;
  const error = queryError
    ? getDisplayErrorMessage(queryError, 'データ取得に失敗しました')
    : mutationError
      ? getDisplayErrorMessage(mutationError, '操作に失敗しました')
      : null;

  return {
    dbData,
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
