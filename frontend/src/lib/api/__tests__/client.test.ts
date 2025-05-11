import { describe, it, expect, vi, beforeEach } from 'vitest';
import axios, { AxiosInstance } from 'axios';
import { apiClient } from '../client';

vi.mock('axios');

describe('apiClient', () => {
  let createMock: ReturnType<typeof vi.spyOn>;
  let instanceMock: Partial<AxiosInstance>;

  beforeEach(() => {
    instanceMock = {
      get: vi.fn(),
      post: vi.fn(),
      put: vi.fn(),
      delete: vi.fn(),
      interceptors: {
        request: {
          use: vi.fn(),
          eject: vi.fn(),
          clear: vi.fn(),
          handlers: []
        },
        response: {
          use: vi.fn(),
          eject: vi.fn(),
          clear: vi.fn(),
          handlers: []
        }
      }
    };

    createMock = vi.spyOn(axios, 'create').mockReturnValue(instanceMock as AxiosInstance);
  });

  describe('設定の初期化', () => {
    it('正しい設定でAxiosインスタンスが作成される', () => {
      // apiClient のインポート時に既に create が呼ばれているはず
      expect(createMock).toHaveBeenCalledWith({
        baseURL: import.meta.env.VITE_SHOKEN_WEBAPI_API_URL,
        timeout: 10000,
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json',
        }
      });
    });

    it('インターセプターが設定される', () => {
      expect(instanceMock.interceptors?.request.use).toHaveBeenCalled();
      expect(instanceMock.interceptors?.response.use).toHaveBeenCalled();
    });
  });

  describe('APIクライアントの統合', () => {
    it('GETリクエストを実行できる', async () => {
      const mockData = { test: 'data' };
      (instanceMock.get as ReturnType<typeof vi.fn>).mockResolvedValue({ data: mockData });

      const result = await apiClient.get('/test');
      
      expect(instanceMock.get).toHaveBeenCalledWith('/test');
      expect(result.data).toEqual(mockData);
    });

    it('POSTリクエストを実行できる', async () => {
      const mockData = { test: 'data' };
      const postData = { id: 1 };
      (instanceMock.post as ReturnType<typeof vi.fn>).mockResolvedValue({ data: mockData });

      const result = await apiClient.post('/test', postData);
      
      expect(instanceMock.post).toHaveBeenCalledWith('/test', postData);
      expect(result.data).toEqual(mockData);
    });
  });
});
