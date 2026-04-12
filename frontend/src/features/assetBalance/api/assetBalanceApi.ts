import { apiClient } from '@/lib/api/client';
import { uploadCsvFile, previewCsvFile } from '@/lib/api/csvHelpers';
import type { AssetBalanceApiData } from '@/types/api';

export const assetBalanceApi = {
  list: () =>
    apiClient.get<AssetBalanceApiData[]>('/asset-balances', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/asset-balances/csv/preview', file),

  uploadCsv: (file: File) => uploadCsvFile('/asset-balances/csv', file),

  deleteAll: async () =>
    apiClient.delete('/asset-balances', { withCredentials: true }),
};
