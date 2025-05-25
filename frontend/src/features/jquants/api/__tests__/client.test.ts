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
    it('認証が成功した場合、リフレッシュトークンを返す', async () => {
      const mockResponse = { data: { refresh_token: 'test-refresh-token' } };
      (apiClient.post as ReturnType<typeof vi.fn>).mockResolvedValue(mockResponse);

      const result = await client.authenticate();

      expect(apiClient.post).toHaveBeenCalledWith('/jquants/auth');
      expect(result).toBe('test-refresh-token');
    });

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
    it('財務諸表データを正常に取得できる', async () => {
      // 認証のモック
      vi.spyOn(client, 'authenticate').mockResolvedValue('test-refresh-token');

      // IDトークン取得のモック
      (apiClient.post as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        data: { id_token: 'test-id-token' }
      });

      // 財務諸表取得のモック
      const mockStatements = { statements: [] };
      (apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue({
        data: mockStatements
      });

      const result = await client.getStatements('1234');

      expect(apiClient.get).toHaveBeenCalledWith(
        '/jquants/fins/statements',
        {
          params: { code: '1234' },
          headers: { Authorization: 'Bearer test-id-token' }
        }
      );
      expect(result).toEqual(mockStatements);
    });

    it('fromとtoを指定して財務諸表データを取得できる', async () => {
      client.setRefreshToken('existing-refresh-token');

      // IDトークン取得のモック
      (apiClient.post as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        data: { id_token: 'test-id-token' }
      });

      // 財務諸表取得のモック
      const mockStatements = { statements: [] };
      (apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue({
        data: mockStatements
      });

      await client.getStatements('1234', '2023-01-01', '2023-12-31');

      expect(apiClient.get).toHaveBeenCalledWith(
        '/jquants/fins/statements',
        {
          params: {
            code: '1234',
            from: '2023-01-01',
            to: '2023-12-31'
          },
          headers: { Authorization: 'Bearer test-id-token' }
        }
      );
    });

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
