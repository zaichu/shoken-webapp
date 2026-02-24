import type { QueryClient } from '@tanstack/react-query';

// ユーザーごとにキャッシュを分離し、別ユーザーへのデータ漏洩を防ぐ
export const receiptQueryKeys = {
  // ログアウト時の cancelQueries/removeQueries に使うプレフィックスキー
  prefix: ['receipts'] as const,
  dividend: (userId: string) => ['receipts', userId, 'dividend'] as const,
  domesticstock: (userId: string) => ['receipts', userId, 'domesticstock'] as const,
  mutualfund: (userId: string) => ['receipts', userId, 'mutualfund'] as const,
};

/** ログアウト時の receipts キャッシュ全削除（プレフィックスマッチ） */
export function clearReceiptsCache(queryClient: QueryClient): void {
  queryClient.cancelQueries({ queryKey: receiptQueryKeys.prefix });
  queryClient.removeQueries({ queryKey: receiptQueryKeys.prefix });
}
