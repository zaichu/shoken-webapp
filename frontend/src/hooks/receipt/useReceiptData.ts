/**
 * 受取データ用の共通カスタムフック
 * CSVデータの変換とソートを行う
 */
export function useReceiptData<T>(
  csvData: Record<string, unknown>[],
  parseItem: (item: Record<string, unknown>) => T,
  sortFunction: (data: T[]) => T[]
): T[] {
  const parsedData = csvData.map(parseItem);
  return sortFunction(parsedData);
}

/**
 * 受取データの集計用カスタムフック
 */
export function useReceiptCalculations<T, C>(
  data: T[],
  calculateFunction: (data: T[]) => C
): C {
  return calculateFunction(data);
}
