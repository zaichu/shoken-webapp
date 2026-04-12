import { apiClient } from '@/lib/api/client';
import { uploadCsvFile, previewCsvFile } from '@/lib/api/csvHelpers';
import type { DividendApiData } from '@/lib/interfaces/dividend';
import type { DomesticStockApiData } from '@/lib/interfaces/domesticStock';
import type { MutualfundApiData } from '@/lib/interfaces/mutualfund';

// 配当金API
export const dividendApi = {
  list: () =>
    apiClient.get<DividendApiData[]>('/dividends', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/dividends/csv/preview', file),

  uploadCsv: (file: File) => uploadCsvFile('/dividends/csv', file),

  deleteAll: async () =>
    apiClient.delete('/dividends', { withCredentials: true }),
};

// 国内株式API
export const domesticStockApi = {
  list: () =>
    apiClient.get<DomesticStockApiData[]>('/domestic-stocks', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/domestic-stocks/csv/preview', file),

  uploadCsv: (file: File) => uploadCsvFile('/domestic-stocks/csv', file),

  deleteAll: async () =>
    apiClient.delete('/domestic-stocks', { withCredentials: true }),
};

// 投資信託API
export const mutualfundApi = {
  list: () =>
    apiClient.get<MutualfundApiData[]>('/mutualfunds', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/mutualfunds/csv/preview', file),

  uploadCsv: (file: File) => uploadCsvFile('/mutualfunds/csv', file),

  deleteAll: async () =>
    apiClient.delete('/mutualfunds', { withCredentials: true }),
};
