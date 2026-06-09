import { apiClient } from '@/lib/api/client';
import { uploadCsvFile, previewCsvFile } from '@/lib/api/csvHelpers';
import type { paths } from '@/generated/api';

const API_PATHS = {
  assetBalanceList: '/api/v1/asset-balances',
  assetBalancePreview: '/api/v1/asset-balance-import-validations',
  assetBalanceImport: '/api/v1/asset-balance-imports',
} as const satisfies Record<string, keyof paths>;

type AssetBalanceListResponse = paths['/api/v1/asset-balances']['get']['responses'][200]['content']['application/json'];
type AssetBalanceDeleteResponse = paths['/api/v1/asset-balances']['delete']['responses'][200]['content']['application/json'];

export const assetBalanceApi = {
  list: () =>
    apiClient.get<AssetBalanceListResponse>(API_PATHS.assetBalanceList, { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile(API_PATHS.assetBalancePreview, file),

  uploadCsv: (file: File) => uploadCsvFile(API_PATHS.assetBalanceImport, file),

  deleteAll: async () =>
    apiClient.delete<AssetBalanceDeleteResponse>(API_PATHS.assetBalanceList, { withCredentials: true }),
};
