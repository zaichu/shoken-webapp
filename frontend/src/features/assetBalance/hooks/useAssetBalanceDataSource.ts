import { useState, useEffect, useCallback, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { assetBalanceApi } from '@/features/assetBalance/api/assetBalanceApi';
import { useCSVReader, CSVReaderHook } from '@/hooks/useCSVReader';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';
import { assetBalanceQueryKeys, clearAssetBalanceCache } from '../queryKeys';

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
 * ユーザー固有クエリキーでキャッシュを分離する
 */
export function useAssetBalanceDataSource(
  parseCsvItem: (item: Record<string, unknown>) => AssetBalanceData,
  filterCsvItem?: (item: AssetBalanceData) => boolean
): UseAssetBalanceDataSourceResult {
  const { isAuthenticated, isLoading: authLoading, onLogout, user } = useAuth();
  const queryClient = useQueryClient();
  const userId = user?.id ?? '';

  const [csvData, setCsvData] = useState<Record<string, unknown>[]>([]);
  const csvReader = useCSVReader({ skipHeaderRows: 6 });

  const dbQuery = useQuery({
    queryKey: assetBalanceQueryKeys.all(userId),
    queryFn: () => assetBalanceApi.list(),
    enabled: isAuthenticated && !authLoading && !!userId,
  });

  const bulkCreateMutation = useMutation({
    mutationFn: (items: AssetBalanceData[]) => assetBalanceApi.bulkCreate(items),
    // mutate 呼び出し時点の userId をスナップショット（ログアウト→再ログイン中の無効化キーのぶれを防止）
    onMutate: () => ({ snapshotUserId: userId }),
    onSuccess: (_, __, context) => {
      queryClient.invalidateQueries({ queryKey: assetBalanceQueryKeys.all(context?.snapshotUserId ?? userId) });
    },
  });

  const deleteAllMutation = useMutation({
    mutationFn: () => assetBalanceApi.deleteAll(),
    // mutate 呼び出し時点の userId をスナップショット（ログアウト→再ログイン中の上書き防止）
    onMutate: () => ({ snapshotUserId: userId }),
    onSuccess: (_, __, context) => {
      const key = assetBalanceQueryKeys.all(context?.snapshotUserId ?? userId);
      // ログアウト等で clearAssetBalanceCache が先行しキャッシュが消えている場合は再生成しない
      if (queryClient.getQueryState(key) !== undefined) {
        queryClient.setQueryData(key, []);
      }
    },
  });

  // ログアウト時：プレフィックスマッチで全ユーザーキャッシュをクリア（他画面からのログアウトにも対応）
  useEffect(() => {
    return onLogout(() => {
      clearAssetBalanceCache(queryClient);
      setCsvData([]);
      csvReader.reset();
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
