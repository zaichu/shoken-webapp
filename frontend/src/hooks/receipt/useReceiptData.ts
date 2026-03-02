import { useMemo } from 'react';

/**
 * 受取データの集計用カスタムフック
 */
export function useReceiptCalculations<T, C>(
  data: T[],
  calculateFunction: (data: T[]) => C
): C {
  return useMemo(() => calculateFunction(data), [data, calculateFunction]);
}
