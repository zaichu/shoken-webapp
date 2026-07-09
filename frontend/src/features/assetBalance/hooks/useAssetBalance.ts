import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
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
  securityCode?: string;
}

/**
 * 保有銘柄データを Query キャッシュから取得するフック
 * ユーザー固有キーでキャッシュを分離し、未認証時はフェッチせず空配列を返す
 */
export function useAssetBalance(options: UseAssetBalanceOptions = {}): UseAssetBalanceReturn {
  const { enabled = true, securityCode } = options;
  const { isAuthenticated, onLogout, user } = useAuth();
  const queryClient = useQueryClient();
  const userId = user?.id ?? '';
  const normalizedSecurityCode = normalizeSecurityCode(securityCode ?? '');
  const queryKey = normalizedSecurityCode
    ? assetBalanceQueryKeys.lookup(userId, normalizedSecurityCode)
    : assetBalanceQueryKeys.all(userId);

  const query = useQuery({
    queryKey,
    queryFn: async () => {
      const response = await assetBalanceApi.list(
        normalizedSecurityCode
          ? { page: 1, per_page: 1, security_code: normalizedSecurityCode }
          : { page: 1, per_page: 1000 }
      );
      return response.data;
    },
    enabled: isAuthenticated && !!userId && enabled,
  });

  // ログアウト時：このフックがマウントされている経路でも確実にキャッシュを除去する
  useEffect(() => {
    return onLogout(() => clearAssetBalanceCache(queryClient));
  }, [onLogout, queryClient]);

  // 未認証時はキャッシュに残存データがあっても空を返す
  const assetBalanceData = (isAuthenticated ? query.data : undefined) ?? [];

  const getAssetBalanceByCode = (code: string): AssetBalanceData | undefined => {
    const normalizedCode = normalizeSecurityCode(code);
    if (!normalizedCode) return undefined;
    return assetBalanceData.find(balance =>
      balance && normalizeSecurityCode(balance.security_code) === normalizedCode
    );
  };

  const getTotalMarketValue = (): number => {
    return assetBalanceData.reduce((total, balance) => total + (balance?.market_value || 0), 0);
  };

  const refetch = async () => {
    await queryClient.invalidateQueries({ queryKey });
  };

  return {
    assetBalanceData,
    isLoading: query.isLoading,
    getAssetBalanceByCode,
    getTotalMarketValue,
    refetch,
  };
}
