import { apiClient } from '@/lib/api/client';
import { DividendData } from '@/lib/interfaces/dividend';
import { DomesticStockData } from '@/lib/interfaces/domesticStock';
import { MutualfundData } from '@/lib/interfaces/mutualfund';
import type { components } from '@/generated/api';

// APIレスポンス型（OpenAPI スキーマから生成）
type CsvUploadResponse = components['schemas']['CsvUploadResponse'];

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

  deleteAll: async () =>
    apiClient.delete('/dividends/all', { withCredentials: true }),
};

// 国内株式API
export const domesticStockApi = {
  list: () =>
    apiClient.get<DomesticStockData[]>('/domestic-stocks', { withCredentials: true }),

  uploadCsv: (file: File) => uploadCsvFile('/domestic-stocks/csv', file),

  deleteAll: async () =>
    apiClient.delete('/domestic-stocks/all', { withCredentials: true }),
};

// 投資信託API
export const mutualfundApi = {
  list: () =>
    apiClient.get<MutualfundData[]>('/mutualfunds', { withCredentials: true }),

  uploadCsv: (file: File) => uploadCsvFile('/mutualfunds/csv', file),

  deleteAll: async () =>
    apiClient.delete('/mutualfunds/all', { withCredentials: true }),
};

