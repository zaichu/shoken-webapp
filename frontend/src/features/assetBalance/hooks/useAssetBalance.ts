import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useMemo } from 'react';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { assetBalanceApi } from '@/features/assetBalance/api/assetBalanceApi';
import { logError } from '@/lib/utils/errorHandler';
import { normalizeSecurityCode } from '@/lib/utils/formatters';
import { assetBalanceQueryKeys } from '../queryKeys';

export interface UseAssetBalanceReturn {
  assetBalanceData: AssetBalanceData[];
  isLoading: boolean;
  getAssetBalanceByCode: (code: string) => AssetBalanceData | undefined;
  getTotalMarketValue: () => number;
  refetch: () => Promise<void>;
}

interface UseAssetBalanceOptions {
  enabled?: boolean;
}

/**
 * 保有銘柄データを Query キャッシュから取得するフック
 */
export function useAssetBalance(options: UseAssetBalanceOptions = {}): UseAssetBalanceReturn {
  const { enabled = true } = options;
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: assetBalanceQueryKeys.all,
    queryFn: async () => {
      try {
        return await assetBalanceApi.list();
      } catch (error) {
        logError('保有銘柄データ取得', error);
        return [] as AssetBalanceData[];
      }
    },
    enabled,
  });

  const assetBalanceData = useMemo(() => query.data ?? [], [query.data]);

  const getAssetBalanceByCode = useCallback(
    (code: string): AssetBalanceData | undefined => {
      const normalizedCode = normalizeSecurityCode(code);
      if (!normalizedCode) return undefined;
      return assetBalanceData.find(balance =>
        balance && normalizeSecurityCode(balance.security_code) === normalizedCode
      );
    },
    [assetBalanceData]
  );

  const getTotalMarketValue = useCallback((): number => {
    return assetBalanceData.reduce((total, balance) => total + (balance?.market_value || 0), 0);
  }, [assetBalanceData]);

  const refetch = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: assetBalanceQueryKeys.all });
  }, [queryClient]);

  return {
    assetBalanceData,
    isLoading: query.isLoading,
    getAssetBalanceByCode,
    getTotalMarketValue,
    refetch,
  };
}
