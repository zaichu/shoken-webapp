import { describe, it, expect, vi, beforeEach } from 'vitest';
import { MarketDataApiClient } from '../client';
import { apiClient } from '@/lib/api/client';
import { ApiError, ApiErrorType } from '@/lib/types/api';

vi.mock('@/lib/api/client', () => ({
  apiClient: {
    get: vi.fn(),
  },
}));

describe('MarketDataApiClient', () => {
  let client: MarketDataApiClient;

  beforeEach(() => {
    vi.clearAllMocks();
    client = new MarketDataApiClient();
  });

  describe('getSummary', () => {
    it('決算サマリーを正常に取得できる', async () => {
      const mockResponse = {
        data: [
          { NxFDivAnn: '100' },
        ],
      };
      (apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue(mockResponse);

      const result = await client.getSummary('1234');

      expect(apiClient.get).toHaveBeenCalledWith('/api/v1/financial-statements', {
        params: { code: '1234' },
        withCredentials: true,
      });
      expect(result).toEqual(mockResponse);
    });

    it('オプションパラメータを含めて取得できる', async () => {
      const mockResponse = { data: [] };
      (apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue(mockResponse);

      await client.getSummary('1234', '2024-01-01', '2024-12-31');

      expect(apiClient.get).toHaveBeenCalledWith('/api/v1/financial-statements', {
        params: { code: '1234', from: '2024-01-01', to: '2024-12-31' },
        withCredentials: true,
      });
    });

    it('ApiErrorの場合、そのままスローする', async () => {
      const apiError = new ApiError(ApiErrorType.NETWORK_ERROR, 'Network Error');
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(apiError);

      await expect(client.getSummary('1234')).rejects.toBe(apiError);
    });

    it('その他のエラーの場合、デフォルトのApiErrorを投げる', async () => {
      const error = new Error('Unknown Error');
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(error);

      await expect(client.getSummary('1234')).rejects.toThrow('決算サマリーの取得に失敗しました');
    });
  });
});
