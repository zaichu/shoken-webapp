import { apiClient } from '@/lib/api/client';
import { uploadCsvFile, previewCsvFile } from '@/lib/api/csvHelpers';
import { DividendData } from '@/lib/interfaces/dividend';
import { DomesticStockData } from '@/lib/interfaces/domesticStock';
import { MutualfundData } from '@/lib/interfaces/mutualfund';

// 配当金API
export const dividendApi = {
  list: () =>
    apiClient.get<DividendData[]>('/dividends', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/dividends/csv/preview', file),

  uploadCsv: (file: File) => uploadCsvFile('/dividends/csv', file),

  deleteAll: async () =>
    apiClient.delete('/dividends/all', { withCredentials: true }),
};

// 国内株式API
export const domesticStockApi = {
  list: () =>
    apiClient.get<DomesticStockData[]>('/domestic-stocks', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/domestic-stocks/csv/preview', file),

  uploadCsv: (file: File) => uploadCsvFile('/domestic-stocks/csv', file),

  deleteAll: async () =>
    apiClient.delete('/domestic-stocks/all', { withCredentials: true }),
};

// 投資信託API
export const mutualfundApi = {
  list: () =>
    apiClient.get<MutualfundData[]>('/mutualfunds', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/mutualfunds/csv/preview', file),

  uploadCsv: (file: File) => uploadCsvFile('/mutualfunds/csv', file),

  deleteAll: async () =>
    apiClient.delete('/mutualfunds/all', { withCredentials: true }),
};
