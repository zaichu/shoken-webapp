import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect } from 'react';
import { dividendApi, domesticStockApi, mutualfundApi } from '../api/receiptApi';
import {
  parseDividendCsvItem,
  parseDomesticStockCsvItem,
  parseMutualfundCsvItem,
  transformDBDividend,
  transformDBDomesticStock,
  transformDBMutualfund,
} from '../parsers';
import { receiptQueryKeys, clearReceiptsCache } from '../queryKeys';
import { type ReceiptsType } from '@/pages/receiptsReducer';
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';
import { useAuth } from '@/features/auth/hooks/useAuth';

export interface UseReceiptsDataResult {
  dividendData: ReturnType<typeof transformDBDividend>[];
  domesticstockData: ReturnType<typeof transformDBDomesticStock>[];
  mutualfundData: ReturnType<typeof transformDBMutualfund>[];
  dbLoading: boolean;
  dbError: string | null;
  saving: boolean;
  deleting: boolean;
  bulkCreate: (args: BulkCreateArgs) => void;
  deleteAll: (type: ReceiptsType) => void;
  clearCache: () => void;
}

interface BulkCreateArgs {
  type: ReceiptsType;
  csvData: Record<string, unknown>[];
  onSuccess?: () => void;
}

/**
 * 明細データの取得・保存・削除を TanStack Query で管理するフック
 * ユーザー固有キーでキャッシュを分離し、未認証時はフェッチせず空を返す
 */
export function useReceiptsData(): UseReceiptsDataResult {
  const { isAuthenticated, isLoading: authLoading, onLogout, user } = useAuth();
  const userId = user?.id ?? '';
  const queryClient = useQueryClient();

  // ログアウト時：プレフィックスマッチで全ユーザーキャッシュをクリア
  useEffect(() => {
    return onLogout(() => clearReceiptsCache(queryClient));
  }, [onLogout, queryClient]);

  const dividendQuery = useQuery({
    queryKey: receiptQueryKeys.dividend(userId),
    queryFn: () =>
      dividendApi.list().then(items =>
        items.map(d => transformDBDividend(d as unknown as Record<string, unknown>))
      ),
    enabled: isAuthenticated && !authLoading && !!userId,
  });

  const domesticstockQuery = useQuery({
    queryKey: receiptQueryKeys.domesticstock(userId),
    queryFn: () =>
      domesticStockApi.list().then(items =>
        items.map(d => transformDBDomesticStock(d as unknown as Record<string, unknown>))
      ),
    enabled: isAuthenticated && !authLoading && !!userId,
  });

  const mutualfundQuery = useQuery({
    queryKey: receiptQueryKeys.mutualfund(userId),
    queryFn: () =>
      mutualfundApi.list().then(items =>
        items.map(d => transformDBMutualfund(d as unknown as Record<string, unknown>))
      ),
    enabled: isAuthenticated && !authLoading && !!userId,
  });

  const bulkCreateMutation = useMutation({
    mutationFn: ({ type, csvData }: BulkCreateArgs) => {
      switch (type) {
        case 'dividend':
          return dividendApi.bulkCreate(csvData.map(parseDividendCsvItem));
        case 'domesticstock':
          return domesticStockApi.bulkCreate(csvData.map(parseDomesticStockCsvItem));
        case 'mutualfund':
          return mutualfundApi.bulkCreate(csvData.map(parseMutualfundCsvItem));
      }
    },
    // mutate 呼び出し時点の userId をスナップショット（ログアウト→再ログイン中の上書き防止）
    onMutate: () => ({ snapshotUserId: userId }),
    onSuccess: (_, { type, onSuccess }, context) => {
      queryClient.invalidateQueries({ queryKey: receiptQueryKeys[type](context?.snapshotUserId ?? userId) });
      onSuccess?.();
    },
  });

  const deleteAllMutation = useMutation({
    mutationFn: (type: ReceiptsType) => {
      switch (type) {
        case 'dividend':
          return dividendApi.deleteAll();
        case 'domesticstock':
          return domesticStockApi.deleteAll();
        case 'mutualfund':
          return mutualfundApi.deleteAll();
      }
    },
    // mutate 呼び出し時点の userId をスナップショット（ログアウト→再ログイン中の上書き防止）
    onMutate: () => ({ snapshotUserId: userId }),
    onSuccess: (_, type, context) => {
      const key = receiptQueryKeys[type](context?.snapshotUserId ?? userId);
      // ログアウト等で clearReceiptsCache が先行しキャッシュが消えている場合は再生成しない
      if (queryClient.getQueryState(key) !== undefined) {
        queryClient.setQueryData(key, []);
      }
    },
  });

  const clearCache = useCallback(() => {
    clearReceiptsCache(queryClient);
  }, [queryClient]);

  const queryError = dividendQuery.error ?? domesticstockQuery.error ?? mutualfundQuery.error;
  const mutationError = bulkCreateMutation.error ?? deleteAllMutation.error;

  return {
    dividendData: dividendQuery.data ?? [],
    domesticstockData: domesticstockQuery.data ?? [],
    mutualfundData: mutualfundQuery.data ?? [],
    dbLoading:
      dividendQuery.isFetching ||
      domesticstockQuery.isFetching ||
      mutualfundQuery.isFetching,
    dbError: queryError
      ? getDisplayErrorMessage(queryError, 'データ取得に失敗しました')
      : mutationError
        ? getDisplayErrorMessage(mutationError, '操作に失敗しました')
        : null,
    saving: bulkCreateMutation.isPending,
    deleting: deleteAllMutation.isPending,
    bulkCreate: bulkCreateMutation.mutate,
    deleteAll: deleteAllMutation.mutate,
    clearCache,
  };
}
