import type { QueryClient } from '@tanstack/react-query';

// ユーザーごとにキャッシュを分離し、別ユーザーへのデータ漏洩を防ぐ
export const assetBalanceQueryKeys = {
  // ログアウト時の cancelQueries/removeQueries に使うプレフィックスキー
  prefix: ['assetBalance'] as const,
  // useAssetBalance（DividendInfo 用）が参照する全件配列キャッシュ
  all: (userId: string) => ['assetBalance', userId] as const,
  // useAssetBalanceDataSource が参照する summary/facets 付きレスポンスキャッシュ
  // all と shape が異なるため、同じキーで共有しない
  list: (userId: string) => ['assetBalance', userId, 'list'] as const,
};

/** ログアウト時の assetBalance キャッシュ全削除（プレフィックスマッチ） */
export function clearAssetBalanceCache(queryClient: QueryClient): void {
  queryClient.cancelQueries({ queryKey: assetBalanceQueryKeys.prefix });
  queryClient.removeQueries({ queryKey: assetBalanceQueryKeys.prefix });
}
