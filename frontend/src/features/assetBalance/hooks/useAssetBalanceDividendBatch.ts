import type { DividendStatus } from '@/features/dividendPerShare/api/dividendPerShareApi';
import { useDividendBatch } from '@/features/dividendPerShare/hooks/useDividendBatch';
import type { AssetBalanceData } from '@/types/api';

export interface UseAssetBalanceDividendBatchResult {
  dividendPerShareMap: Map<string, number>;
  dividendStatusMap: Map<string, DividendStatus> | undefined;
}

/**
 * 表示中の保有銘柄コードに対応する1株配当をバッチ取得するフック
 * 保存中・読み込み中は問い合わせ対象を空にして無駄なリクエストを避ける
 */
export function useAssetBalanceDividendBatch(
  assetBalanceData: AssetBalanceData[],
  isAuthenticated: boolean,
  skip: boolean
): UseAssetBalanceDividendBatchResult {
  const securityCodes = skip ? [] : assetBalanceData.map((item) => item.security_code);
  const { dividendPerShareMap, dividendStatusMap } = useDividendBatch(securityCodes, isAuthenticated);

  return {
    dividendPerShareMap: dividendPerShareMap as Map<string, number>,
    dividendStatusMap: dividendStatusMap as Map<string, DividendStatus> | undefined,
  };
}
