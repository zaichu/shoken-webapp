import { useState, useEffect, useCallback, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { assetBalanceApi } from '@/features/assetBalance/api/assetBalanceApi';
import { useCSVReader, CSVReaderHook } from '@/hooks/useCSVReader';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';
import { assetBalanceQueryKeys } from '../queryKeys';

export interface UseAssetBalanceDataSourceResult {
  // データ
  dbData: AssetBalanceData[];
  csvData: Record<string, unknown>[];
  // 状態
  loading: boolean;
  error: string | null;
  saving: boolean;
  deleting: boolean;
  // CSV関連
  csvReader: CSVReaderHook;
  // フラグ
  hasCsvData: boolean;
  hasDbData: boolean;
  // アクション
  handleFileSelect: (file: File) => Promise<void>;
  handleSaveToDB: () => Promise<void>;
  handleDeleteAll: () => Promise<void>;
}

/**
 * 保有銘柄データソースを TanStack Query で管理するフック
 */
export function useAssetBalanceDataSource(
  parseCsvItem: (item: Record<string, unknown>) => AssetBalanceData,
  filterCsvItem?: (item: AssetBalanceData) => boolean
): UseAssetBalanceDataSourceResult {
  const { isAuthenticated, isLoading: authLoading, onLogout } = useAuth();
  const queryClient = useQueryClient();

  const [csvData, setCsvData] = useState<Record<string, unknown>[]>([]);
  const csvReader = useCSVReader({ skipHeaderRows: 6 });

  const dbQuery = useQuery({
    queryKey: assetBalanceQueryKeys.all,
    queryFn: () => assetBalanceApi.list(),
    enabled: isAuthenticated && !authLoading,
  });

  const bulkCreateMutation = useMutation({
    mutationFn: (items: AssetBalanceData[]) => assetBalanceApi.bulkCreate(items),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: assetBalanceQueryKeys.all });
    },
  });

  const deleteAllMutation = useMutation({
    mutationFn: () => assetBalanceApi.deleteAll(),
    onSuccess: () => {
      queryClient.setQueryData(assetBalanceQueryKeys.all, []);
    },
  });

  // ログアウト時にキャッシュと CSV をクリア
  useEffect(() => {
    return onLogout(() => {
      setCsvData([]);
      csvReader.reset();
      queryClient.setQueryData(assetBalanceQueryKeys.all, []);
    });
  }, [onLogout, csvReader, queryClient]);

  const handleFileSelect = useCallback(async (file: File) => {
    try {
      const data = await csvReader.parseCSV(file);
      setCsvData(data);
      if (csvReader.error) csvReader.resetError();
    } catch {
      // CSVパースエラーは csvReader.error に反映される
    }
  }, [csvReader]);

  const handleSaveToDB = useCallback(async () => {
    if (!isAuthenticated || csvData.length === 0) return;
    let items = csvData.map(parseCsvItem);
    if (filterCsvItem) items = items.filter(filterCsvItem);
    try {
      await bulkCreateMutation.mutateAsync(items);
      setCsvData([]);
      csvReader.reset();
    } catch {
      // エラーは bulkCreateMutation.error に反映される
    }
  }, [bulkCreateMutation, csvData, csvReader, filterCsvItem, isAuthenticated, parseCsvItem]);

  const handleDeleteAll = useCallback(async () => {
    if (!isAuthenticated) return;
    try {
      await deleteAllMutation.mutateAsync();
    } catch {
      // エラーは deleteAllMutation.error に反映される
    }
  }, [deleteAllMutation, isAuthenticated]);

  const dbData = useMemo(() => dbQuery.data ?? [], [dbQuery.data]);
  const queryError = dbQuery.error;
  const mutationError = bulkCreateMutation.error ?? deleteAllMutation.error;
  const error = queryError
    ? getDisplayErrorMessage(queryError, 'データ取得に失敗しました')
    : mutationError
      ? getDisplayErrorMessage(mutationError, '操作に失敗しました')
      : null;

  const hasCsvData = csvData.length > 0;
  const hasDbData = useMemo(() => dbData.length > 0, [dbData]);

  return {
    dbData,
    csvData,
    loading: dbQuery.isFetching,
    error,
    saving: bulkCreateMutation.isPending,
    deleting: deleteAllMutation.isPending,
    csvReader,
    hasCsvData,
    hasDbData,
    handleFileSelect,
    handleSaveToDB,
    handleDeleteAll,
  };
}
