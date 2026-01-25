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

// 数値を安全に変換（NaN, null, undefinedを0に変換）
const safeNumber = (value: unknown): number => {
  const num = Number(value);
  return Number.isFinite(num) ? num : 0;
};

// 配当金API
export const dividendApi = {
  list: () =>
    apiClient.get<DividendData[]>('/dividends', { withCredentials: true }),

  bulkCreate: async (items: DividendData[]) => {
    console.log('[dividendApi.bulkCreate] 開始:', items.length, '件');
    const payload = items.map(item => ({
      settlement_date: formatDate(item.settlement_date as Date),
      product: item.product,
      account: item.account,
      security_code: item.security_code,
      security_name: item.security_name,
      unit_price: safeNumber(item.unit_price),
      shares: safeNumber(item.shares),
      dividends_before_tax: safeNumber(item.dividends_before_tax),
      taxes: safeNumber(item.taxes),
      net_amount_received: safeNumber(item.net_amount_received),
    }));
    try {
      const response = await apiClient.post<BulkCreateResponse>('/dividends/bulk', { items: payload }, { withCredentials: true });
      console.log('[dividendApi.bulkCreate] 成功');
      return response;
    } catch (error) {
      console.error('[dividendApi.bulkCreate] 失敗:', error);
      throw error;
    }
  },

  deleteAll: async () => {
    console.log('[dividendApi.deleteAll] 開始');
    try {
      const response = await apiClient.delete('/dividends/all', { withCredentials: true });
      console.log('[dividendApi.deleteAll] 成功');
      return response;
    } catch (error) {
      console.error('[dividendApi.deleteAll] 失敗:', error);
      throw error;
    }
  },
};

// 国内株式API
export const domesticStockApi = {
  list: () =>
    apiClient.get<DomesticStockData[]>('/domestic-stocks', { withCredentials: true }),

  bulkCreate: async (items: DomesticStockData[]) => {
    console.log('[domesticStockApi.bulkCreate] 開始:', items.length, '件');
    const payload = items.map(item => ({
      trade_date: formatDate(item.trade_date as Date),
      settlement_date: formatDate(item.settlement_date as Date),
      security_code: item.security_code,
      security_name: item.security_name,
      account: item.account,
      shares: safeNumber(item.shares),
      asked_price: safeNumber(item.asked_price),
      proceeds: safeNumber(item.proceeds),
      purchase_price: safeNumber(item.purchase_price),
      realized_profit_and_loss: safeNumber(item.realized_profit_and_loss),
      taxes: safeNumber(item.taxes),
      realized_profit_and_loss_after_tax: safeNumber(item.realized_profit_and_loss_after_tax),
    }));
    try {
      const response = await apiClient.post<BulkCreateResponse>('/domestic-stocks/bulk', { items: payload }, { withCredentials: true });
      console.log('[domesticStockApi.bulkCreate] 成功');
      return response;
    } catch (error) {
      console.error('[domesticStockApi.bulkCreate] 失敗:', error);
      throw error;
    }
  },

  deleteAll: async () => {
    console.log('[domesticStockApi.deleteAll] 開始');
    try {
      const response = await apiClient.delete('/domestic-stocks/all', { withCredentials: true });
      console.log('[domesticStockApi.deleteAll] 成功');
      return response;
    } catch (error) {
      console.error('[domesticStockApi.deleteAll] 失敗:', error);
      throw error;
    }
  },
};

// 投資信託API
export const mutualfundApi = {
  list: () =>
    apiClient.get<MutualfundData[]>('/mutualfunds', { withCredentials: true }),

  bulkCreate: async (items: MutualfundData[]) => {
    console.log('[mutualfundApi.bulkCreate] 開始:', items.length, '件');
    const payload = items.map(item => ({
      trade_date: formatDate(item.trade_date as Date),
      settlement_date: formatDate(item.settlement_date as Date),
      fund_name: item.fund_name,
      dividends: item.dividends || null,
      account: item.account,
      shares: safeNumber(item.shares),
      exchange_rate: safeNumber(item.exchange_rate),
      cancellation_unit_price_yen: safeNumber(item.cancellation_unit_price_yen),
      cancellation_amount_yen: safeNumber(item.cancellation_amount_yen),
      average_acquisition_price_yen: safeNumber(item.average_acquisition_price_yen),
      realized_profit_and_loss: safeNumber(item.realized_profit_and_loss),
      taxes: safeNumber(item.taxes),
      realized_profit_and_loss_after_tax: safeNumber(item.realized_profit_and_loss_after_tax),
    }));
    try {
      const response = await apiClient.post<BulkCreateResponse>('/mutualfunds/bulk', { items: payload }, { withCredentials: true });
      console.log('[mutualfundApi.bulkCreate] 成功');
      return response;
    } catch (error) {
      console.error('[mutualfundApi.bulkCreate] 失敗:', error);
      throw error;
    }
  },

  deleteAll: async () => {
    console.log('[mutualfundApi.deleteAll] 開始');
    try {
      const response = await apiClient.delete('/mutualfunds/all', { withCredentials: true });
      console.log('[mutualfundApi.deleteAll] 成功');
      return response;
    } catch (error) {
      console.error('[mutualfundApi.deleteAll] 失敗:', error);
      throw error;
    }
  },
};
