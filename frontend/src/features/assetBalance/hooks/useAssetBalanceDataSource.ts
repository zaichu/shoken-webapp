import { useState, useEffect, useCallback, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { assetBalanceApi } from '@/features/assetBalance/api/assetBalanceApi';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';
import { assetBalanceQueryKeys, clearAssetBalanceCache } from '../queryKeys';
import type { CsvUploadResult } from '@/lib/csvImport';

interface UseAssetBalanceDataSourceResult {
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
}

/**
 * 保有銘柄データソースを TanStack Query で管理するフック
 * ファイル選択時にバックエンドプレビューAPIを呼び出し、行データを取得する
 */
export function useAssetBalanceDataSource(): UseAssetBalanceDataSourceResult {
  const { isAuthenticated, isLoading: authLoading, onLogout, user } = useAuth();
  const queryClient = useQueryClient();
  const userId = user?.id ?? '';

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

  useEffect(() => {
    return onLogout(() => {
      clearAssetBalanceCache(queryClient);
      setRawFile(null);
      setPreviewRows([]);
      setLastSavedResult(null);
    });
  }, [onLogout, queryClient]);

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

  const dbData = useMemo(() => dbQuery.data ?? [], [dbQuery.data]);
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
  };
}
