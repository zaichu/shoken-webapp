import { describe, it, expect, vi, beforeEach } from 'vitest';
import axios, { AxiosInstance } from 'axios';

// モックの設定は先に定義する
vi.mock('axios');

describe('apiClient', () => {
  const mockAxiosInstance = {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
    interceptors: {
      request: {
        use: vi.fn(),
      },
      response: {
        use: vi.fn(),
      }
    }
  };

  beforeEach(() => {
    vi.clearAllMocks();
    // axios.createのモック実装をセットアップ
    vi.mocked(axios.create).mockReturnValue(mockAxiosInstance as unknown as AxiosInstance);
  });

  describe('createApiClient', () => {
    it('正しい設定でAxiosインスタンスが作成される', async () => {
      // モジュールをリロードして、createApiClientを再実行
      const { createApiClient } = await import('../client');

      createApiClient();

      expect(axios.create).toHaveBeenCalledWith({
        baseURL: import.meta.env.VITE_SHOKEN_WEBAPI_API_URL,
        timeout: 10000,
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json',
        }
      });
    });

    it('インターセプターが設定される', async () => {
      const { createApiClient } = await import('../client');

      createApiClient();

      expect(mockAxiosInstance.interceptors.request.use).toHaveBeenCalled();
      expect(mockAxiosInstance.interceptors.response.use).toHaveBeenCalled();
    });

    it('作成されたインスタンスが返される', async () => {
      const { createApiClient } = await import('../client');

      const result = createApiClient();

      expect(result).toBe(mockAxiosInstance);
    });
  });

  describe('APIクライアントの統合', () => {
    it('GETリクエストを実行できる', async () => {
      const { apiClient } = await import('../client');
      const mockData = { test: 'data' };
      (mockAxiosInstance.get as ReturnType<typeof vi.fn>).mockResolvedValue({ data: mockData });

      const result = await apiClient.get('/test');

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/test');
      expect(result.data).toEqual(mockData);
    });

    it('POSTリクエストを実行できる', async () => {
      const { apiClient } = await import('../client');
      const mockData = { test: 'data' };
      const postData = { id: 1 };
      (mockAxiosInstance.post as ReturnType<typeof vi.fn>).mockResolvedValue({ data: mockData });

      const result = await apiClient.post('/test', postData);

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/test', postData);
      expect(result.data).toEqual(mockData);
    });
  });
});
