import { apiClient } from '@/lib/api/client';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';

interface BulkCreateResponse {
  inserted: number;
  skipped: number;
}

const safeNumber = (value: unknown): number => {
  const num = Number(value);
  return Number.isFinite(num) ? num : 0;
};

export const assetBalanceApi = {
  list: () =>
    apiClient.get<AssetBalanceData[]>('/asset-balances', { withCredentials: true }),

  bulkCreate: async (items: AssetBalanceData[]) => {
    const payload = items.map(item => ({
      security_code: item.security_code,
      security_name: item.security_name,
      shares: safeNumber(item.shares),
      executing_shares: safeNumber(item.executing_shares),
      average_purchase_price: safeNumber(item.average_purchase_price),
      total_purchase_amount: safeNumber(item.total_purchase_amount),
      current_price: safeNumber(item.current_price),
      daily_change: safeNumber(item.daily_change),
      market_value: safeNumber(item.market_value),
      profit_loss_rate: safeNumber(item.profit_loss_rate),
    }));
    return apiClient.post<BulkCreateResponse>('/asset-balances/bulk', { items: payload }, { withCredentials: true });
  },

  deleteAll: async () =>
    apiClient.delete('/asset-balances/all', { withCredentials: true }),
};
