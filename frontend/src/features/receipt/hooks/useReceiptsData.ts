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
import type { components } from '@/generated/api';

export type DividendApiSummary = components['schemas']['DividendSummary'];
export type DomesticStockApiSummary = components['schemas']['DomesticStockSummary'];
export type MutualfundApiSummary = components['schemas']['MutualfundSummary'];

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
  // 検索条件全体（DB 側）の集計。1000件超のユーザーでも正しい合計を表示するために使用する
  dividendSummary: DividendApiSummary | undefined;
  domesticstockSummary: DomesticStockApiSummary | undefined;
  mutualfundSummary: MutualfundApiSummary | undefined;
  dbLoading: boolean;
  dbError: string | null;
  saving: boolean;
  deleting: boolean;
  previewing: boolean;
  receiptListLimit: number;
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

const RECEIPT_LIST_LIMIT = 1000;

const RECEIPT_LIST_PARAMS = {
  per_page: RECEIPT_LIST_LIMIT,
  page: 1,
  include_summary: true,
} as const;

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
      dividendApi.list(RECEIPT_LIST_PARAMS).then((response) => ({
        items: response.data.map(transformDBDividend),
        summary: response.summary,
      })),
    enabled: isAuthenticated && !authLoading && !!userId,
  });

  const domesticstockQuery = useQuery({
    queryKey: receiptQueryKeys.domesticstock(userId),
    queryFn: () =>
      domesticStockApi.list(RECEIPT_LIST_PARAMS).then((response) => ({
        items: response.data.map(transformDBDomesticStock),
        summary: response.summary,
      })),
    enabled: isAuthenticated && !authLoading && !!userId,
  });

  const mutualfundQuery = useQuery({
    queryKey: receiptQueryKeys.mutualfund(userId),
    queryFn: () =>
      mutualfundApi.list(RECEIPT_LIST_PARAMS).then((response) => ({
        items: response.data.map(transformDBMutualfund),
        summary: response.summary,
      })),
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
        queryClient.setQueryData(key, { items: [], summary: undefined });
      }
    },
  });

  const queryError = dividendQuery.error ?? domesticstockQuery.error ?? mutualfundQuery.error;
  const mutationError = uploadCsvMutation.error ?? deleteAllMutation.error;

  return {
    dividendData: dividendQuery.data?.items ?? [],
    domesticstockData: domesticstockQuery.data?.items ?? [],
    mutualfundData: mutualfundQuery.data?.items ?? [],
    dividendSummary: dividendQuery.data?.summary,
    domesticstockSummary: domesticstockQuery.data?.summary,
    mutualfundSummary: mutualfundQuery.data?.summary,
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
    receiptListLimit: RECEIPT_LIST_LIMIT,
    uploadCsv: uploadCsvMutation.mutate,
    previewCsv: previewCsvMutation.mutate,
    deleteAll: (type, options) => deleteAllMutation.mutate(type, { onSuccess: options?.onSuccess }),
  };
}
