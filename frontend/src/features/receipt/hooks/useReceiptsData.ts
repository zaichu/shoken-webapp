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

// previewCsv/uploadCsv/deleteAll は3ドメインとも同一シグネチャのため、type キーで引ける lookup map にまとめる
const receiptApiByType = {
  dividend: dividendApi,
  domesticstock: domesticStockApi,
  mutualfund: mutualfundApi,
} satisfies Record<ReceiptsType, Pick<typeof dividendApi, 'previewCsv' | 'uploadCsv' | 'deleteAll'>>;

interface ReceiptListQueryConfig<TDbItem, TItem, TSummary> {
  queryKey: readonly unknown[];
  list: (params: typeof RECEIPT_LIST_PARAMS) => Promise<{ data: TDbItem[]; summary?: TSummary }>;
  transform: (item: TDbItem) => TItem;
}

/** 明細一覧の取得ロジック（クエリキー・APIモジュール・変換関数以外は3ドメイン共通） */
function useReceiptListQuery<TDbItem, TItem, TSummary>(
  config: ReceiptListQueryConfig<TDbItem, TItem, TSummary>,
  enabled: boolean,
) {
  return useQuery({
    queryKey: config.queryKey,
    queryFn: () =>
      config.list(RECEIPT_LIST_PARAMS).then((response) => ({
        items: response.data.map(config.transform),
        summary: response.summary,
      })),
    enabled,
  });
}

/**
 * 明細データの取得・保存・削除を TanStack Query で管理するフック
 * ユーザー固有キーでキャッシュを分離し、未認証時はフェッチせず空を返す
 */
export function useReceiptsData(): UseReceiptsDataResult {
  const { isAuthenticated, isLoading: authLoading, onLogout, user } = useAuth();
  const userId = user?.id ?? '';
  const queryClient = useQueryClient();
  const enabled = isAuthenticated && !authLoading && !!userId;

  // ログアウト時：プレフィックスマッチで全ユーザーキャッシュをクリア
  useEffect(() => {
    return onLogout(() => clearReceiptsCache(queryClient));
  }, [onLogout, queryClient]);

  const dividendQuery = useReceiptListQuery(
    {
      queryKey: receiptQueryKeys.dividend(userId),
      list: dividendApi.list,
      transform: transformDBDividend,
    },
    enabled,
  );

  const domesticstockQuery = useReceiptListQuery(
    {
      queryKey: receiptQueryKeys.domesticstock(userId),
      list: domesticStockApi.list,
      transform: transformDBDomesticStock,
    },
    enabled,
  );

  const mutualfundQuery = useReceiptListQuery(
    {
      queryKey: receiptQueryKeys.mutualfund(userId),
      list: mutualfundApi.list,
      transform: transformDBMutualfund,
    },
    enabled,
  );

  const previewCsvMutation = useMutation({
    mutationFn: ({ type, file }: PreviewCsvArgs) => receiptApiByType[type].previewCsv(file),
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
    mutationFn: ({ type, file }: UploadCsvArgs) => receiptApiByType[type].uploadCsv(file),
    // mutate 呼び出し時点の userId をスナップショット（ログアウト→再ログイン中の上書き防止）
    onMutate: () => ({ snapshotUserId: userId }),
    onSuccess: (data, { type, onSuccess }, context) => {
      queryClient.invalidateQueries({ queryKey: receiptQueryKeys[type](context?.snapshotUserId ?? userId) });
      onSuccess?.(data);
    },
  });

  const deleteAllMutation = useMutation({
    mutationFn: (type: ReceiptsType) => receiptApiByType[type].deleteAll(),
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
