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
  refetch: () => Promise<void>;
}

interface UseAssetBalanceOptions {
  enabled?: boolean;
  securityCode: string;
}

/**
 * 指定銘柄コードの保有銘柄データを Query キャッシュから取得するフック
 * ユーザー固有キーでキャッシュを分離し、未認証時はフェッチせず空配列を返す
 */
export function useAssetBalance({ enabled = true, securityCode }: UseAssetBalanceOptions): UseAssetBalanceReturn {
  const { isAuthenticated, onLogout, user } = useAuth();
  const queryClient = useQueryClient();
  const userId = user?.id ?? '';
  const normalizedSecurityCode = normalizeSecurityCode(securityCode);
  const queryKey = assetBalanceQueryKeys.lookup(userId, normalizedSecurityCode);

  const query = useQuery({
    queryKey,
    queryFn: async () => {
      const response = await assetBalanceApi.list({
        page: 1,
        per_page: 1,
        security_code: normalizedSecurityCode,
      });
      return response.data;
    },
    enabled: isAuthenticated && !!userId && !!normalizedSecurityCode && enabled,
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

  const refetch = async () => {
    await queryClient.invalidateQueries({ queryKey });
  };

  return {
    assetBalanceData,
    isLoading: query.isLoading,
    getAssetBalanceByCode,
    refetch,
  };
}
