import { beforeEach, describe, expect, it, vi } from 'vitest';

import { apiClient } from '@/lib/api/client';
import { previewCsvFile, uploadCsvFile } from '@/lib/api/csvHelpers';
import { assetBalanceApi } from '../assetBalanceApi';

vi.mock('@/lib/api/client', () => ({
  apiClient: {
    get: vi.fn(),
    post: vi.fn(),
    delete: vi.fn(),
  },
}));

vi.mock('@/lib/api/csvHelpers', () => ({
  uploadCsvFile: vi.fn(),
  previewCsvFile: vi.fn(),
}));

describe('assetBalanceApi', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('list() は asset-balances 一覧取得 API を呼ぶ', () => {
    assetBalanceApi.list();

    expect(apiClient.get).toHaveBeenCalledWith('/api/v1/asset-balances', {
      withCredentials: true,
    });
  });

  it('previewCsv(file) は previewCsvFile を呼ぶ', () => {
    const file = new File(['id,name\n1,test'], 'asset-balances.csv', { type: 'text/csv' });

    assetBalanceApi.previewCsv(file);

    expect(previewCsvFile).toHaveBeenCalledWith('/api/v1/asset-balance-import-validations', file);
  });

  it('uploadCsv(file) は uploadCsvFile を呼ぶ', () => {
    const file = new File(['id,name\n1,test'], 'asset-balances.csv', { type: 'text/csv' });

    assetBalanceApi.uploadCsv(file);

    expect(uploadCsvFile).toHaveBeenCalledWith('/api/v1/asset-balance-imports', file);
  });

  it('deleteAll() は全件削除 API を呼ぶ', async () => {
    await assetBalanceApi.deleteAll();

    expect(apiClient.delete).toHaveBeenCalledWith('/api/v1/asset-balances', {
      withCredentials: true,
    });
  });
});
