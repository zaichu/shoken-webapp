import { apiClient } from '@/lib/api/client';
import { DividendData } from '@/lib/interfaces/dividend';
import { DomesticStockData } from '@/lib/interfaces/domesticStock';
import { MutualfundData } from '@/lib/interfaces/mutualfund';

// APIレスポンス型
interface BulkCreateResponse {
  inserted: number;
  skipped: number;
}

// Date を YYYY-MM-DD 形式に変換
const formatDate = (date: Date): string => {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
};

// 配当金API
export const dividendApi = {
  list: () =>
    apiClient.get<DividendData[]>('/dividends', { withCredentials: true }),

  bulkCreate: (items: Omit<DividendData, 'id' | 'security_info' | 'created_at' | 'updated_at'>[]) => {
    const payload = items.map(item => ({
      ...item,
      settlement_date: formatDate(item.settlement_date as Date),
    }));
    return apiClient.post<BulkCreateResponse>('/dividends/bulk', { items: payload }, { withCredentials: true });
  },

  deleteAll: () =>
    apiClient.delete('/dividends/all', { withCredentials: true }),
};

// 国内株式API
export const domesticStockApi = {
  list: () =>
    apiClient.get<DomesticStockData[]>('/domestic-stocks', { withCredentials: true }),

  bulkCreate: (items: Omit<DomesticStockData, 'id' | 'security_info' | 'created_at' | 'updated_at'>[]) => {
    const payload = items.map(item => ({
      ...item,
      trade_date: formatDate(item.trade_date as Date),
    }));
    return apiClient.post<BulkCreateResponse>('/domestic-stocks/bulk', { items: payload }, { withCredentials: true });
  },

  deleteAll: () =>
    apiClient.delete('/domestic-stocks/all', { withCredentials: true }),
};

// 投資信託API
export const mutualfundApi = {
  list: () =>
    apiClient.get<MutualfundData[]>('/mutualfunds', { withCredentials: true }),

  bulkCreate: (items: Omit<MutualfundData, 'id' | 'created_at' | 'updated_at'>[]) => {
    const payload = items.map(item => ({
      ...item,
      trade_date: formatDate(item.trade_date as Date),
    }));
    return apiClient.post<BulkCreateResponse>('/mutualfunds/bulk', { items: payload }, { withCredentials: true });
  },

  deleteAll: () =>
    apiClient.delete('/mutualfunds/all', { withCredentials: true }),
};
