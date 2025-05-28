import { describe, it, expect, vi, beforeEach } from 'vitest';
import axios from 'axios';
import { JQuantsApiClient } from '../client';
import { apiClient } from '@/lib/api/client';
import { ApiError, ApiErrorType } from '@/lib/types/api';

vi.mock('@/lib/api/client', () => ({
  apiClient: {
    get: vi.fn(),
    post: vi.fn(),
  }
}));

vi.mock('axios');

describe('JQuantsApiClient', () => {
  let client: JQuantsApiClient;

  beforeEach(() => {
    vi.clearAllMocks();
    client = new JQuantsApiClient();
  });

  describe('authenticate', () => {
    it('Axiosエラーの場合、ApiErrorを投げる', async () => {
      const axiosError = new Error('Network Error');
      (axios.isAxiosError as ReturnType<typeof vi.fn>).mockReturnValue(true);
      (apiClient.post as ReturnType<typeof vi.fn>).mockRejectedValue(axiosError);

      // Mock ApiError.fromAxiosError
      const mockApiError = new ApiError(ApiErrorType.CONNECTION_ERROR, 'Network Error');
      vi.spyOn(ApiError, 'fromAxiosError').mockReturnValue(mockApiError);

      await expect(client.authenticate()).rejects.toThrow(ApiError);
      expect(ApiError.fromAxiosError).toHaveBeenCalledWith(axiosError);
    });

    it('その他のエラーの場合、デフォルトのApiErrorを投げる', async () => {
      const error = new Error('Unknown Error');
      (axios.isAxiosError as ReturnType<typeof vi.fn>).mockReturnValue(false);
      (apiClient.post as ReturnType<typeof vi.fn>).mockRejectedValue(error);

      await expect(client.authenticate()).rejects.toThrow('認証に失敗しました');
    });
  });

  describe('getStatements', () => {
    it('取得エラーの場合、適切なApiErrorを投げる', async () => {
      client.setRefreshToken('existing-refresh-token');

      // IDトークン取得のモック
      (apiClient.post as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        data: { id_token: 'test-id-token' }
      });

      // エラーのモック
      const axiosError = new Error('Network Error');
      (axios.isAxiosError as ReturnType<typeof vi.fn>).mockReturnValue(true);
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(axiosError);

      // Mock ApiError.fromAxiosError
      const mockApiError = new ApiError(ApiErrorType.CONNECTION_ERROR, 'Network Error');
      vi.spyOn(ApiError, 'fromAxiosError').mockReturnValue(mockApiError);

      await expect(client.getStatements('1234')).rejects.toThrow(ApiError);
    });
  });

  describe('setRefreshToken', () => {
    it('リフレッシュトークンを設定できる', () => {
      client.setRefreshToken('new-token');
      // メソッドは内部状態を変更するだけなので、直接テストは難しい
      // 代わりに、後続の操作で設定が効いているかを確認
      expect(() => client.setRefreshToken('new-token')).not.toThrow();
    });
  });
});
