import { useCallback } from 'react';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { useLocalStorage } from '@/hooks/common/useLocalStorage';

const ASSET_BALANCE_STORAGE_KEY = 'asset-balance-data';

/**
 * 保有株データをローカルストレージで管理するカスタムフック
 */
export const useAssetBalanceStorage = () => {
  const [storedAssetBalance, setStoredAssetBalance] = useLocalStorage<AssetBalanceData[]>(ASSET_BALANCE_STORAGE_KEY, []);
  const [lastUpdated, setLastUpdated] = useLocalStorage<string | null>('asset-balance-last-updated', null);

  /**
   * 保有株データを保存する
   */
  const saveAssetBalance = useCallback((assetBalance: AssetBalanceData[]) => {
    setStoredAssetBalance(assetBalance);
    setLastUpdated(new Date().toISOString());
  }, [setStoredAssetBalance, setLastUpdated]);

  /**
   * 保有株データをクリアする
   */
  const clearAssetBalance = useCallback(() => {
    setStoredAssetBalance([]);
    setLastUpdated(null);
  }, [setStoredAssetBalance, setLastUpdated]);

  /**
   * 特定の銘柄の保有株情報を取得する
   */
  const getAssetBalanceByCode = useCallback((security_code: string): AssetBalanceData | undefined => {
    return storedAssetBalance.find(assetBalance => assetBalance.security_code === security_code);
  }, [storedAssetBalance]);

  return {
    assetBalanceStorageData: storedAssetBalance,
    lastUpdated,
    saveAssetBalance,
    clearAssetBalance,
    getAssetBalanceByCode,
  };
};
