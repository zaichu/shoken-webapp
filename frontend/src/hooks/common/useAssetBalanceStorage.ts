import { useState, useCallback } from 'react';
import { useLocalStorage } from './useLocalStorage';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';

export interface UseAssetBalanceStorageReturn {
  assetBalanceStorageData: AssetBalanceData[];
  isLoading: boolean;
  saveAssetBalance: (data: AssetBalanceData[]) => void;
  clearAssetBalance: () => void;
  getAssetBalanceByCode: (code: string) => AssetBalanceData | undefined;
  getTotalMarketValue: () => number;
  lastUpdated: string | null;
}

export function useAssetBalanceStorage(): UseAssetBalanceStorageReturn {
  const [assetBalanceStorageData, setAssetBalanceStorageData] = useLocalStorage<AssetBalanceData[]>('assetBalances', []);
  const [lastUpdated, setLastUpdated] = useLocalStorage<string | null>('assetBalancesLastUpdated', null);
  const [isLoading, setIsLoading] = useState(false);

  // 資産データを保存
  const saveAssetBalance = useCallback((data: AssetBalanceData[]): void => {
    setIsLoading(true);
    try {
      setAssetBalanceStorageData(data);
      setLastUpdated(new Date().toLocaleString('ja-JP'));
    } finally {
      setIsLoading(false);
    }
  }, [setAssetBalanceStorageData, setLastUpdated]);

  // 資産データをクリア
  const clearAssetBalance = useCallback((): void => {
    setIsLoading(true);
    try {
      setAssetBalanceStorageData([]);
      setLastUpdated(null);
    } finally {
      setIsLoading(false);
    }
  }, [setAssetBalanceStorageData, setLastUpdated]);

  // 銘柄コードで資産を取得
  const getAssetBalanceByCode = useCallback((code: string): AssetBalanceData | undefined => {
    return assetBalanceStorageData.find(balance => balance && balance.security_code === code);
  }, [assetBalanceStorageData]);

  // 全資産の市場価値合計を計算
  const getTotalMarketValue = useCallback((): number => {
    return assetBalanceStorageData.reduce((total, balance) => total + (balance?.market_value || 0), 0);
  }, [assetBalanceStorageData]);

  return {
    assetBalanceStorageData,
    isLoading,
    saveAssetBalance,
    clearAssetBalance,
    getAssetBalanceByCode,
    getTotalMarketValue,
    lastUpdated,
  };
}
