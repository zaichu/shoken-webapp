import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import { dividendApi, domesticStockApi, mutualfundApi } from '../api/receiptApi';
import {
  transformDBDividend,
  transformDBDomesticStock,
  transformDBMutualfund,
} from '../parsers';
import { receiptQueryKeys, clearReceiptsCache } from '../queryKeys';
import { type ReceiptsType } from '../reducer';
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';
import { useAuth } from '@/features/auth/hooks/useAuth';
import type { CsvImportError, CsvUploadResult } from '@/lib/csvImport';

interface CsvPreviewResult {
  totalRows: number;
  validRows: number;
  errors: CsvImportError[];
  rows: Record<string, unknown>[];
}

interface UseReceiptsDataResult {
  dividendData: ReturnType<typeof transformDBDividend>[];
  domesticstockData: ReturnType<typeof transformDBDomesticStock>[];
  mutualfundData: ReturnType<typeof transformDBMutualfund>[];
  dbLoading: boolean;
  dbError: string | null;
  saving: boolean;
  deleting: boolean;
  previewing: boolean;
  uploadCsv: (args: UploadCsvArgs) => void;
  previewCsv: (args: PreviewCsvArgs) => void;
  deleteAll: (type: ReceiptsType, options?: { onSuccess?: () => void }) => void;
}

interface UploadCsvArgs {
  type: ReceiptsType;
  file: File;
  onSuccess?: (result: CsvUploadResult) => void;
}

interface PreviewCsvArgs {
  type: ReceiptsType;
  file: File;
  onSuccess?: (result: CsvPreviewResult) => void;
  onError?: (error: string) => void;
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
      dividendApi.list({ per_page: 1000 }).then(({ data: dividendRows }) =>
        dividendRows.map(transformDBDividend)),
    enabled: isAuthenticated && !authLoading && !!userId,
  });

  const domesticstockQuery = useQuery({
    queryKey: receiptQueryKeys.domesticstock(userId),
    queryFn: () =>
      domesticStockApi.list({ per_page: 1000 }).then(({ data: domesticStockRows }) =>
        domesticStockRows.map(transformDBDomesticStock)),
    enabled: isAuthenticated && !authLoading && !!userId,
  });

  const mutualfundQuery = useQuery({
    queryKey: receiptQueryKeys.mutualfund(userId),
    queryFn: () =>
      mutualfundApi.list({ per_page: 1000 }).then(({ data: mutualfundRows }) =>
        mutualfundRows.map(transformDBMutualfund)),
    enabled: isAuthenticated && !authLoading && !!userId,
  });

  const previewCsvMutation = useMutation({
    mutationFn: ({ type, file }: PreviewCsvArgs) => {
      switch (type) {
        case 'dividend':
          return dividendApi.previewCsv(file);
        case 'domesticstock':
          return domesticStockApi.previewCsv(file);
        case 'mutualfund':
          return mutualfundApi.previewCsv(file);
      }
    },
    onSuccess: (previewResult, { onSuccess }) => {
      onSuccess?.({
        totalRows: previewResult.total_rows,
        validRows: previewResult.valid_rows,
        errors: previewResult.errors,
        rows: (previewResult.rows ?? []) as Record<string, unknown>[],
      });
    },
    onError: (error, { onError }) => {
      const message = error instanceof Error ? error.message : '解析に失敗しました';
      onError?.(message);
    },
  });

  const uploadCsvMutation = useMutation({
    mutationFn: ({ type, file }: UploadCsvArgs) => {
      switch (type) {
        case 'dividend':
          return dividendApi.uploadCsv(file);
        case 'domesticstock':
          return domesticStockApi.uploadCsv(file);
        case 'mutualfund':
          return mutualfundApi.uploadCsv(file);
      }
    },
    // mutate 呼び出し時点の userId をスナップショット（ログアウト→再ログイン中の上書き防止）
    onMutate: () => ({ snapshotUserId: userId }),
    onSuccess: (data, { type, onSuccess }, context) => {
      queryClient.invalidateQueries({ queryKey: receiptQueryKeys[type](context?.snapshotUserId ?? userId) });
      onSuccess?.(data);
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

  const queryError = dividendQuery.error ?? domesticstockQuery.error ?? mutualfundQuery.error;
  const mutationError = uploadCsvMutation.error ?? deleteAllMutation.error;

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
    saving: uploadCsvMutation.isPending,
    deleting: deleteAllMutation.isPending,
    previewing: previewCsvMutation.isPending,
    uploadCsv: uploadCsvMutation.mutate,
    previewCsv: previewCsvMutation.mutate,
    deleteAll: (type, options) => deleteAllMutation.mutate(type, { onSuccess: options?.onSuccess }),
  };
}
