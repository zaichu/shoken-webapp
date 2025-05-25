import { useCallback } from 'react';
import { HoldingsData } from '@/lib/interfaces/holdings';
import { useLocalStorage } from '@/hooks/common/useLocalStorage';

const HOLDINGS_STORAGE_KEY = 'holdings-data';

/**
 * 保有株データをローカルストレージで管理するカスタムフック
 */
export const useHoldingsStorage = () => {
  const [storedHoldings, setStoredHoldings] = useLocalStorage<HoldingsData[]>(HOLDINGS_STORAGE_KEY, []);
  const [lastUpdated, setLastUpdated] = useLocalStorage<string | null>('holdings-last-updated', null);

  /**
   * 保有株データを保存する
   */
  const saveHoldings = useCallback((holdings: HoldingsData[]) => {
    setStoredHoldings(holdings);
    setLastUpdated(new Date().toISOString());
  }, [setStoredHoldings, setLastUpdated]);

  /**
   * 保有株データをクリアする
   */
  const clearHoldings = useCallback(() => {
    setStoredHoldings([]);
    setLastUpdated(null);
  }, [setStoredHoldings, setLastUpdated]);

  /**
   * 特定の銘柄の保有株情報を取得する
   */
  const getHoldingByCode = useCallback((security_code: string): HoldingsData | undefined => {
    return storedHoldings.find(holding => holding.security_code === security_code);
  }, [storedHoldings]);

  return {
    holdings: storedHoldings,
    lastUpdated,
    saveHoldings,
    clearHoldings,
    getHoldingByCode,
  };
};
