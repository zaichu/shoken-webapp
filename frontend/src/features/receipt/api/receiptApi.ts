import { apiClient } from '@/lib/api/client';
import { DividendData } from '@/lib/interfaces/dividend';
import { DomesticStockData } from '@/lib/interfaces/domesticStock';
import { MutualfundData } from '@/lib/interfaces/mutualfund';
import type { components } from '@/generated/api';

// APIレスポンス型（OpenAPI スキーマから生成）
type BulkCreateResponse = components['schemas']['BulkCreateResponse'];
type CsvUploadResponse = components['schemas']['CsvUploadResponse'];

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

// CSV ファイルを multipart/form-data で送信
const uploadCsvFile = (path: string, file: File) => {
  const formData = new FormData();
  formData.append('file', file);
  return apiClient.post<CsvUploadResponse>(path, formData, { withCredentials: true });
};

// 配当金API
export const dividendApi = {
  list: () =>
    apiClient.get<DividendData[]>('/dividends', { withCredentials: true }),

  uploadCsv: (file: File) => uploadCsvFile('/dividends/csv', file),

  bulkCreate: async (items: DividendData[]) => {
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
    return apiClient.post<BulkCreateResponse>('/dividends/bulk', { items: payload }, { withCredentials: true });
  },

  deleteAll: async () =>
    apiClient.delete('/dividends/all', { withCredentials: true }),
};

// 国内株式API
export const domesticStockApi = {
  list: () =>
    apiClient.get<DomesticStockData[]>('/domestic-stocks', { withCredentials: true }),

  uploadCsv: (file: File) => uploadCsvFile('/domestic-stocks/csv', file),

  bulkCreate: async (items: DomesticStockData[]) => {
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
    return apiClient.post<BulkCreateResponse>('/domestic-stocks/bulk', { items: payload }, { withCredentials: true });
  },

  deleteAll: async () =>
    apiClient.delete('/domestic-stocks/all', { withCredentials: true }),
};

// 投資信託API
export const mutualfundApi = {
  list: () =>
    apiClient.get<MutualfundData[]>('/mutualfunds', { withCredentials: true }),

  uploadCsv: (file: File) => uploadCsvFile('/mutualfunds/csv', file),

  bulkCreate: async (items: MutualfundData[]) => {
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
    return apiClient.post<BulkCreateResponse>('/mutualfunds/bulk', { items: payload }, { withCredentials: true });
  },

  deleteAll: async () =>
    apiClient.delete('/mutualfunds/all', { withCredentials: true }),
};

