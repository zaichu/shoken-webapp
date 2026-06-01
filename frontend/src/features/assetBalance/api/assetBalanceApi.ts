import { apiClient } from '@/lib/api/client';
import { uploadCsvFile, previewCsvFile } from '@/lib/api/csvHelpers';
import type { AssetBalanceApiData } from '@/types/api';

export const assetBalanceApi = {
  list: () =>
    apiClient.get<AssetBalanceApiData[]>('/api/v1/asset-balances', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/api/v1/asset-balance-import-validations', file),

  uploadCsv: (file: File) => uploadCsvFile('/api/v1/asset-balance-imports', file),

  deleteAll: async () =>
    apiClient.delete('/api/v1/asset-balances', { withCredentials: true }),
};
