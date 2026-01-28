import { describe, it, expect, vi, beforeEach } from 'vitest';
import axios from 'axios';
import { JQuantsApiClient } from '../client';
import { apiClient } from '@/lib/api/client';
import { ApiError, ApiErrorType } from '@/lib/types/api';

vi.mock('@/lib/api/client', () => ({
  apiClient: {
    get: vi.fn(),
  },
}));

vi.mock('axios');

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

    it('Axiosエラーの場合、ApiErrorを投げる', async () => {
      const axiosError = new Error('Network Error');
      (axios.isAxiosError as ReturnType<typeof vi.fn>).mockReturnValue(true);
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(axiosError);

      const mockApiError = new ApiError(ApiErrorType.CONNECTION_ERROR, 'Network Error');
      vi.spyOn(ApiError, 'fromAxiosError').mockReturnValue(mockApiError);

      await expect(client.getStatements('1234')).rejects.toThrow(ApiError);
      expect(ApiError.fromAxiosError).toHaveBeenCalledWith(axiosError);
    });

    it('その他のエラーの場合、デフォルトのApiErrorを投げる', async () => {
      const error = new Error('Unknown Error');
      (axios.isAxiosError as ReturnType<typeof vi.fn>).mockReturnValue(false);
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(error);

      await expect(client.getStatements('1234')).rejects.toThrow('財務諸表の取得に失敗しました');
    });
  });
});
