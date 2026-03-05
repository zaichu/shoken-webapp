import { apiClient } from '@/lib/api/client';
import type { components } from '@/generated/api';

type CsvUploadResponse = components['schemas']['CsvUploadResponse'];
type CsvPreviewResponse = components['schemas']['CsvPreviewResponse'];

export const uploadCsvFile = (path: string, file: File) => {
  const formData = new FormData();
  formData.append('file', file);
  return apiClient.post<CsvUploadResponse>(path, formData, { withCredentials: true });
};

export const previewCsvFile = (path: string, file: File) => {
  const formData = new FormData();
  formData.append('file', file);
  return apiClient.post<CsvPreviewResponse>(path, formData, { withCredentials: true });
};
