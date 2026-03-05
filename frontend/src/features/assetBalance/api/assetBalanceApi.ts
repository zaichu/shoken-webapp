import { apiClient } from '@/lib/api/client';
import { uploadCsvFile, previewCsvFile } from '@/lib/api/csvHelpers';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';

export const assetBalanceApi = {
  list: () =>
    apiClient.get<AssetBalanceData[]>('/asset-balances', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/asset-balances/csv/preview', file),

  uploadCsv: (file: File) => uploadCsvFile('/asset-balances/csv', file),

  deleteAll: async () =>
    apiClient.delete('/asset-balances/all', { withCredentials: true }),
};
