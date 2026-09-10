import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
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

type ReceiptDataByType = {
  dividend: ReturnType<typeof transformDBDividend>[];
  domesticstock: ReturnType<typeof transformDBDomesticStock>[];
  mutualfund: ReturnType<typeof transformDBMutualfund>[];
};

type ReceiptSummaryByType = {
  dividend: DividendApiSummary | undefined;
  domesticstock: DomesticStockApiSummary | undefined;
  mutualfund: MutualfundApiSummary | undefined;
};

interface UseReceiptsDataResult {
  data: ReceiptDataByType;
  // 検索条件全体（DB 側）の集計。1000件超のユーザーでも正しい合計を表示するために使用する
  summaries: ReceiptSummaryByType;
  dbLoading: boolean;
  // タブごとの取得中フラグ。選択中タブの表示が他タブの取得完了を待たないようにするために使用する
  loadingByTab: Record<ReceiptsType, boolean>;
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
 *
 * activeTab を渡すと選択中タブを優先取得する。非表示タブは選択中タブの取得完了後に
 * バックグラウンドで取得するため、タブ件数バッジは遅れて表示される。
 * activeTab を省略した場合は従来通り3タブとも同時に取得する。
 */
export function useReceiptsData(activeTab?: ReceiptsType): UseReceiptsDataResult {
  const { isAuthenticated, isLoading: authLoading, onLogout, user } = useAuth();
  const userId = user?.id ?? '';
  const queryClient = useQueryClient();
  const enabled = isAuthenticated && !authLoading && !!userId;

  // ログアウト時：プレフィックスマッチで全ユーザーキャッシュをクリア
  useEffect(() => {
    return onLogout(() => clearReceiptsCache(queryClient));
  }, [onLogout, queryClient]);

  // 訪問済みタブは有効化したままにし、タブ切替で取得中クエリを中断させない
  const visitedTabsRef = useRef<Set<ReceiptsType>>(new Set(activeTab ? [activeTab] : []));
  useEffect(() => {
    if (activeTab !== undefined) {
      visitedTabsRef.current.add(activeTab);
    }
  }, [activeTab]);

  // 選択中タブの初回取得が完了したら非表示タブをバックグラウンド取得する
  const [activeTabSettled, setActiveTabSettled] = useState(false);
  // ユーザー切替時はバックグラウンド取得フラグをリセットする
  useEffect(() => {
    setActiveTabSettled(false);
  }, [userId]);

  const isTabEnabled = (tab: ReceiptsType): boolean => {
    if (!enabled || activeTab === undefined) {
      return enabled;
    }
    return (
      tab === activeTab || visitedTabsRef.current.has(tab) || activeTabSettled
    );
  };

  const dividendQuery = useReceiptListQuery(
    {
      queryKey: receiptQueryKeys.dividend(userId),
      list: dividendApi.list,
      transform: transformDBDividend,
    },
    isTabEnabled('dividend'),
  );

  const domesticstockQuery = useReceiptListQuery(
    {
      queryKey: receiptQueryKeys.domesticstock(userId),
      list: domesticStockApi.list,
      transform: transformDBDomesticStock,
    },
    isTabEnabled('domesticstock'),
  );

  const mutualfundQuery = useReceiptListQuery(
    {
      queryKey: receiptQueryKeys.mutualfund(userId),
      list: mutualfundApi.list,
      transform: transformDBMutualfund,
    },
    isTabEnabled('mutualfund'),
  );

  const queriesByTab = {
    dividend: dividendQuery,
    domesticstock: domesticstockQuery,
    mutualfund: mutualfundQuery,
  };

  // 選択中タブの取得完了を検知してバックグラウンド取得を開始する
  const activeQueryFetched = activeTab === undefined ? true : queriesByTab[activeTab].isFetched;
  useEffect(() => {
    if (activeQueryFetched) {
      setActiveTabSettled(true);
    }
  }, [activeQueryFetched]);

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
    data: {
      dividend: dividendQuery.data?.items ?? [],
      domesticstock: domesticstockQuery.data?.items ?? [],
      mutualfund: mutualfundQuery.data?.items ?? [],
    },
    summaries: {
      dividend: dividendQuery.data?.summary,
      domesticstock: domesticstockQuery.data?.summary,
      mutualfund: mutualfundQuery.data?.summary,
    },
    dbLoading:
      dividendQuery.isFetching ||
      domesticstockQuery.isFetching ||
      mutualfundQuery.isFetching,
    loadingByTab: {
      dividend: dividendQuery.isFetching,
      domesticstock: domesticstockQuery.isFetching,
      mutualfund: mutualfundQuery.isFetching,
    },
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
