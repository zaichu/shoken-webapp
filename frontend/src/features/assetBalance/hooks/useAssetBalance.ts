import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo } from 'react';
import type { AssetBalanceData } from '@/types/api';
import { assetBalanceApi } from '@/features/assetBalance/api/assetBalanceApi';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { normalizeSecurityCode } from '@/lib/utils/formatters';
import { assetBalanceQueryKeys, clearAssetBalanceCache } from '../queryKeys';

interface UseAssetBalanceReturn {
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
 * ユーザー固有キーでキャッシュを分離し、未認証時はフェッチせず空配列を返す
 */
export function useAssetBalance(options: UseAssetBalanceOptions = {}): UseAssetBalanceReturn {
  const { enabled = true } = options;
  const { isAuthenticated, onLogout, user } = useAuth();
  const queryClient = useQueryClient();
  const userId = user?.id ?? '';

  const query = useQuery({
    queryKey: assetBalanceQueryKeys.all(userId),
    queryFn: () => assetBalanceApi.list(),
    enabled: isAuthenticated && !!userId && enabled,
  });

  // ログアウト時：このフックがマウントされている経路でも確実にキャッシュを除去する
  useEffect(() => {
    return onLogout(() => clearAssetBalanceCache(queryClient));
  }, [onLogout, queryClient]);

  // 未認証時はキャッシュに残存データがあっても空を返す
  const assetBalanceData = useMemo(
    () => (isAuthenticated ? (query.data ?? []) : []),
    [isAuthenticated, query.data]
  );

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
    await queryClient.invalidateQueries({ queryKey: assetBalanceQueryKeys.all(userId) });
  }, [queryClient, userId]);

  return {
    assetBalanceData,
    isLoading: query.isLoading,
    getAssetBalanceByCode,
    getTotalMarketValue,
    refetch,
  };
}
