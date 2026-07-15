import { useMemo } from 'react';
import { useReceiptPageState } from './useReceiptPageState';
import { type FilterConfig } from '@/lib/utils/searchUtils';

/**
 * 受取データの集計用カスタムフック
 */
export function useReceiptCalculations<T, C>(
  data: T[],
  calculateFunction: (data: T[]) => C
): C {
  return useMemo(() => calculateFunction(data), [data, calculateFunction]);
}

/**
 * 受取明細コンポーネント共通の基盤フック
 * previewData があればそちらを優先し、ソート・フィルタ状態を一括で返す
 */
export function useReceiptBaseData<T>(
  data: T[],
  previewData: T[] | undefined,
  sort: (items: T[]) => T[],
  filterConfig: FilterConfig<T>,
) {
  const displayData = previewData && previewData.length > 0 ? previewData : data;
  const sortedData = sort(displayData);
  const { searchQuery, setSearchQuery, filteredData } = useReceiptPageState(sortedData, filterConfig);
  return { sortedData, searchQuery, setSearchQuery, filteredData };
}

/**
 * ヘッダー集計値の選択ロジック
 * 未フィルタ時は DB 側の集計（1000件キャップの影響を受けない）を優先し、
 * プレビュー中や絞り込み中はフィルタ後データから計算したクライアント集計を使う
 */
export function useReceiptHeaderSummary<TSummary, TCalc>(
  apiSummary: TSummary | undefined,
  previewData: unknown[] | undefined,
  searchQuery: string,
  clientCalculations: TCalc,
): TSummary | TCalc {
  const isPreviewMode = Boolean(previewData && previewData.length > 0);
  return apiSummary && !isPreviewMode && searchQuery === '' ? apiSummary : clientCalculations;
}
