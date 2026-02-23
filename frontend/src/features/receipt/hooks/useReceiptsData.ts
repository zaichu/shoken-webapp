import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useCallback } from 'react';
import { dividendApi, domesticStockApi, mutualfundApi } from '../api/receiptApi';
import {
  parseDividendCsvItem,
  parseDomesticStockCsvItem,
  parseMutualfundCsvItem,
  transformDBDividend,
  transformDBDomesticStock,
  transformDBMutualfund,
} from '../parsers';
import { receiptQueryKeys } from '../queryKeys';
import { type ReceiptsType } from '@/pages/receiptsReducer';
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';

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
 */
export function useReceiptsData(isAuthenticated: boolean): UseReceiptsDataResult {
  const queryClient = useQueryClient();

  const dividendQuery = useQuery({
    queryKey: receiptQueryKeys.dividend,
    queryFn: () =>
      dividendApi.list().then(items =>
        items.map(d => transformDBDividend(d as unknown as Record<string, unknown>))
      ),
    enabled: isAuthenticated,
  });

  const domesticstockQuery = useQuery({
    queryKey: receiptQueryKeys.domesticstock,
    queryFn: () =>
      domesticStockApi.list().then(items =>
        items.map(d => transformDBDomesticStock(d as unknown as Record<string, unknown>))
      ),
    enabled: isAuthenticated,
  });

  const mutualfundQuery = useQuery({
    queryKey: receiptQueryKeys.mutualfund,
    queryFn: () =>
      mutualfundApi.list().then(items =>
        items.map(d => transformDBMutualfund(d as unknown as Record<string, unknown>))
      ),
    enabled: isAuthenticated,
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
    onSuccess: (_, { type, onSuccess }) => {
      queryClient.invalidateQueries({ queryKey: receiptQueryKeys[type] });
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
    onSuccess: (_, type) => {
      queryClient.setQueryData(receiptQueryKeys[type], []);
    },
  });

  const clearCache = useCallback(() => {
    queryClient.setQueryData(receiptQueryKeys.dividend, []);
    queryClient.setQueryData(receiptQueryKeys.domesticstock, []);
    queryClient.setQueryData(receiptQueryKeys.mutualfund, []);
  }, [queryClient]);

  const firstError = dividendQuery.error ?? domesticstockQuery.error ?? mutualfundQuery.error;

  return {
    dividendData: dividendQuery.data ?? [],
    domesticstockData: domesticstockQuery.data ?? [],
    mutualfundData: mutualfundQuery.data ?? [],
    dbLoading:
      dividendQuery.isFetching ||
      domesticstockQuery.isFetching ||
      mutualfundQuery.isFetching,
    dbError: firstError
      ? getDisplayErrorMessage(firstError, 'データ取得に失敗しました')
      : null,
    saving: bulkCreateMutation.isPending,
    deleting: deleteAllMutation.isPending,
    bulkCreate: bulkCreateMutation.mutate,
    deleteAll: deleteAllMutation.mutate,
    clearCache,
  };
}
