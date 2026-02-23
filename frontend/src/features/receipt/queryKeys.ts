import { ReceiptsType } from '@/pages/receiptsReducer';

export const receiptQueryKeys = {
  dividend: ['receipts', 'dividend'] as const,
  domesticstock: ['receipts', 'domesticstock'] as const,
  mutualfund: ['receipts', 'mutualfund'] as const,
} satisfies Record<ReceiptsType, readonly string[]>;
