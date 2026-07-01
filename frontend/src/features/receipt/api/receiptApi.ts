import { apiClient } from '@/lib/api/client';
import { uploadCsvFile, previewCsvFile } from '@/lib/api/csvHelpers';
import type { paths } from '@/generated/api';

const API_PATHS = {
  dividendList: '/api/v1/dividends',
  dividendPreview: '/api/v1/dividend-import-validations',
  dividendImport: '/api/v1/dividend-imports',
  domesticStockList: '/api/v1/domestic-stock-transactions',
  domesticStockPreview: '/api/v1/domestic-stock-import-validations',
  domesticStockImport: '/api/v1/domestic-stock-imports',
  mutualfundList: '/api/v1/mutual-fund-transactions',
  mutualfundPreview: '/api/v1/mutual-fund-import-validations',
  mutualfundImport: '/api/v1/mutual-fund-imports',
} as const satisfies Record<string, keyof paths>;

type DividendListResponse = paths['/api/v1/dividends']['get']['responses'][200]['content']['application/json'];
type DividendDeleteResponse = paths['/api/v1/dividends']['delete']['responses'][200]['content']['application/json'];
type DomesticStockListResponse = paths['/api/v1/domestic-stock-transactions']['get']['responses'][200]['content']['application/json'];
type DomesticStockDeleteResponse = paths['/api/v1/domestic-stock-transactions']['delete']['responses'][200]['content']['application/json'];
type MutualfundListResponse = paths['/api/v1/mutual-fund-transactions']['get']['responses'][200]['content']['application/json'];
type MutualfundDeleteResponse = paths['/api/v1/mutual-fund-transactions']['delete']['responses'][200]['content']['application/json'];

type ListQueryParams = NonNullable<paths['/api/v1/dividends']['get']['parameters']['query']>;
const listRequestConfig = (params?: ListQueryParams) =>
  params ? { params, withCredentials: true } : { withCredentials: true };

// 配当金API
export const dividendApi = {
  list: (params?: ListQueryParams) =>
    apiClient.get<DividendListResponse>(API_PATHS.dividendList, listRequestConfig(params)),

  previewCsv: (file: File) => previewCsvFile(API_PATHS.dividendPreview, file),

  uploadCsv: (file: File) => uploadCsvFile(API_PATHS.dividendImport, file),

  deleteAll: async () =>
    apiClient.delete<DividendDeleteResponse>(API_PATHS.dividendList, { withCredentials: true }),
};

// 国内株式API
export const domesticStockApi = {
  list: (params?: ListQueryParams) =>
    apiClient.get<DomesticStockListResponse>(API_PATHS.domesticStockList, listRequestConfig(params)),

  previewCsv: (file: File) => previewCsvFile(API_PATHS.domesticStockPreview, file),

  uploadCsv: (file: File) => uploadCsvFile(API_PATHS.domesticStockImport, file),

  deleteAll: async () =>
    apiClient.delete<DomesticStockDeleteResponse>(API_PATHS.domesticStockList, { withCredentials: true }),
};

// 投資信託API
export const mutualfundApi = {
  list: (params?: ListQueryParams) =>
    apiClient.get<MutualfundListResponse>(API_PATHS.mutualfundList, listRequestConfig(params)),

  previewCsv: (file: File) => previewCsvFile(API_PATHS.mutualfundPreview, file),

  uploadCsv: (file: File) => uploadCsvFile(API_PATHS.mutualfundImport, file),

  deleteAll: async () =>
    apiClient.delete<MutualfundDeleteResponse>(API_PATHS.mutualfundList, { withCredentials: true }),
};
