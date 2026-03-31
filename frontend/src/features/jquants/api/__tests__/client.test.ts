import { describe, it, expect, vi, beforeEach } from 'vitest';
import { JQuantsApiClient } from '../client';
import { apiClient } from '@/lib/api/client';
import { ApiError, ApiErrorType } from '@/lib/types/api';

vi.mock('@/lib/api/client', () => ({
  apiClient: {
    get: vi.fn(),
  },
}));

describe('JQuantsApiClient', () => {
  let client: JQuantsApiClient;

  beforeEach(() => {
    vi.clearAllMocks();
    client = new JQuantsApiClient();
  });

  describe('getStatements', () => {
    it('財務諸表を正常に取得できる', async () => {
      const mockResponse = {
        data: [
          { NxFDivAnn: '100' },
        ],
      };
      (apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue(mockResponse);

      const result = await client.getStatements('1234');

      expect(apiClient.get).toHaveBeenCalledWith('/jquants/fins/statements', {
        params: { code: '1234' },
      });
      expect(result).toEqual(mockResponse);
    });

    it('オプションパラメータを含めて取得できる', async () => {
      const mockResponse = { data: [] };
      (apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue(mockResponse);

      await client.getStatements('1234', '2024-01-01', '2024-12-31');

      expect(apiClient.get).toHaveBeenCalledWith('/jquants/fins/statements', {
        params: { code: '1234', from: '2024-01-01', to: '2024-12-31' },
      });
    });

    it('ApiErrorの場合、そのままスローする', async () => {
      const apiError = new ApiError(ApiErrorType.NETWORK_ERROR, 'Network Error');
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(apiError);

      await expect(client.getStatements('1234')).rejects.toBe(apiError);
    });

    it('その他のエラーの場合、デフォルトのApiErrorを投げる', async () => {
      const error = new Error('Unknown Error');
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(error);

      await expect(client.getStatements('1234')).rejects.toThrow('財務諸表の取得に失敗しました');
    });
  });
});
