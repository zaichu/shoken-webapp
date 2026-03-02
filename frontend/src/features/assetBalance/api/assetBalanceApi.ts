import { apiClient } from '@/lib/api/client';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import type { components } from '@/generated/api';

type CsvPreviewResponse = components['schemas']['CsvPreviewResponse'];
type CsvUploadResponse = components['schemas']['CsvUploadResponse'];

const uploadCsvFile = (path: string, file: File) => {
  const formData = new FormData();
  formData.append('file', file);
  return apiClient.post<CsvUploadResponse>(path, formData, { withCredentials: true });
};

const previewCsvFile = (path: string, file: File) => {
  const formData = new FormData();
  formData.append('file', file);
  return apiClient.post<CsvPreviewResponse>(path, formData, { withCredentials: true });
};

export const assetBalanceApi = {
  list: () =>
    apiClient.get<AssetBalanceData[]>('/asset-balances', { withCredentials: true }),

  previewCsv: (file: File) => previewCsvFile('/asset-balances/csv/preview', file),

  uploadCsv: (file: File) => uploadCsvFile('/asset-balances/csv', file),

  deleteAll: async () =>
    apiClient.delete('/asset-balances/all', { withCredentials: true }),
};
