import { useState, useEffect, useCallback } from 'react';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { assetBalanceApi } from '@/features/receipt/api/receiptApi';
import { logError } from '@/lib/utils/errorHandler';

export interface UseAssetBalanceReturn {
  assetBalanceData: AssetBalanceData[];
  isLoading: boolean;
  getAssetBalanceByCode: (code: string) => AssetBalanceData | undefined;
  getTotalMarketValue: () => number;
  refetch: () => Promise<void>;
}

/**
 * 保有銘柄データをDBから取得するフック
 */
export function useAssetBalance(): UseAssetBalanceReturn {
  const [assetBalanceData, setAssetBalanceData] = useState<AssetBalanceData[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // DBから保有銘柄データを取得
  const fetchAssetBalances = useCallback(async () => {
    setIsLoading(true);
    try {
      const data = await assetBalanceApi.list();
      setAssetBalanceData(data || []);
    } catch (error) {
      logError('保有銘柄データ取得', error);
      setAssetBalanceData([]);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 初回読み込み
  useEffect(() => {
    fetchAssetBalances();
  }, [fetchAssetBalances]);

  // 銘柄コードで資産を取得
  const getAssetBalanceByCode = useCallback(
    (code: string): AssetBalanceData | undefined => {
      return assetBalanceData.find(balance => balance && balance.security_code === code);
    },
    [assetBalanceData]
  );

  // 全資産の市場価値合計を計算
  const getTotalMarketValue = useCallback((): number => {
    return assetBalanceData.reduce((total, balance) => total + (balance?.market_value || 0), 0);
  }, [assetBalanceData]);

  return {
    assetBalanceData,
    isLoading,
    getAssetBalanceByCode,
    getTotalMarketValue,
    refetch: fetchAssetBalances,
  };
}
