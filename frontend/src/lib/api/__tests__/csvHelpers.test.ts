import { beforeEach, describe, expect, it, vi } from 'vitest';

import { apiClient } from '@/lib/api/client';
import { previewCsvFile, uploadCsvFile } from '../csvHelpers';

vi.mock('@/lib/api/client', () => ({
  apiClient: {
    post: vi.fn(),
  },
}));

describe('csvHelpers', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('uploadCsvFile', () => {
    it('path と file を渡すと FormData を含めて apiClient.post を呼ぶ', () => {
      const file = new File(['id,name\n1,test'], 'asset-balances.csv', { type: 'text/csv' });

      uploadCsvFile('/csv/upload', file);

      expect(apiClient.post).toHaveBeenCalledTimes(1);
      const [path, formData, config] = (apiClient.post as ReturnType<typeof vi.fn>).mock.calls[0];
      expect(path).toBe('/csv/upload');
      expect(formData).toBeInstanceOf(FormData);
      expect((formData as FormData).get('file')).toBe(file);
      expect(config).toEqual({ withCredentials: true });
    });
  });

  describe('previewCsvFile', () => {
    it('path と file を渡すと FormData を含めて apiClient.post を呼ぶ', () => {
      const file = new File(['id,name\n1,test'], 'asset-balances.csv', { type: 'text/csv' });

      previewCsvFile('/csv/preview', file);

      expect(apiClient.post).toHaveBeenCalledTimes(1);
      const [path, formData, config] = (apiClient.post as ReturnType<typeof vi.fn>).mock.calls[0];
      expect(path).toBe('/csv/preview');
      expect(formData).toBeInstanceOf(FormData);
      expect((formData as FormData).get('file')).toBe(file);
      expect(config).toEqual({ withCredentials: true });
    });
  });
});
