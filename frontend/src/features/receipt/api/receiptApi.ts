import { apiClient } from '@/lib/api/client';
import { uploadCsvFile, previewCsvFile } from '@/lib/api/csvHelpers';
import type {
  DividendApiData,
  DomesticStockApiData,
  MutualfundApiData,
} from '@/features/receipt/types';

// 配当金API
export const dividendApi = {
  list: () =>
    apiClient.get<DividendApiData[]>('/api/v1/dividends', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/api/v1/dividend-import-validations', file),

  uploadCsv: (file: File) => uploadCsvFile('/api/v1/dividend-imports', file),

  deleteAll: async () =>
    apiClient.delete('/api/v1/dividends', { withCredentials: true }),
};

// 国内株式API
export const domesticStockApi = {
  list: () =>
    apiClient.get<DomesticStockApiData[]>('/api/v1/domestic-stock-transactions', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/api/v1/domestic-stock-import-validations', file),

  uploadCsv: (file: File) => uploadCsvFile('/api/v1/domestic-stock-imports', file),

  deleteAll: async () =>
    apiClient.delete('/api/v1/domestic-stock-transactions', { withCredentials: true }),
};

// 投資信託API
export const mutualfundApi = {
  list: () =>
    apiClient.get<MutualfundApiData[]>('/api/v1/mutual-fund-transactions', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/api/v1/mutual-fund-import-validations', file),

  uploadCsv: (file: File) => uploadCsvFile('/api/v1/mutual-fund-imports', file),

  deleteAll: async () =>
    apiClient.delete('/api/v1/mutual-fund-transactions', { withCredentials: true }),
};
