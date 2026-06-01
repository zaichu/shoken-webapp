import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fetchStockData, apiRequest } from '../api';
import { apiClient } from '../../../lib/api/client';
import { ApiError, ApiErrorType } from '../../../lib/types/api';

vi.mock('../../../lib/api/client', () => ({
  apiClient: {
    get: vi.fn(),
  }
}));

describe('Stock API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('fetchStockData', () => {
    it('成功した場合、v1銘柄検索APIをquery param付きで呼び出して結果を返す', async () => {
      const mockStock = {
        date: '2024-03-01',
        code: '7974',
        name: '任天堂',
        market_category: 'プライム',
        industry_code_33: '37',
        industry_category_33: '情報・通信業',
        industry_code_17: '10',
        industry_category_17: '情報通信・サービスその他',
        size_code: '7',
        size_category: 'TOPIX Large70',
      };
      (apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue(mockStock);

      const result = await fetchStockData('7974');

      expect(apiClient.get).toHaveBeenCalledWith('/api/v1/stocks', { params: { query: '7974' } });
      expect(result).toEqual(mockStock);
    });

    it('ApiErrorの場合、同じエラーをそのまま再スローする', async () => {
      const apiError = new ApiError(ApiErrorType.NOT_FOUND_ERROR, 'リソースが見つかりません');
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(apiError);

      await expect(fetchStockData('1234')).rejects.toBe(apiError);
    });

    it('非ApiErrorの場合、DESERIALIZATION_ERRORを投げる', async () => {
      const error = new Error('Network Error');
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(error);

      await expect(fetchStockData('1234')).rejects.toThrow('銘柄データの読み込みに失敗しました');
    });

    it('その他のエラーの場合、デフォルトのApiErrorを投げる', async () => {
      const error = new Error('Unknown Error');
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(error);

      await expect(fetchStockData('1234')).rejects.toThrow('銘柄データの読み込みに失敗しました');
    });
  });

  describe('apiRequest', () => {
    it('成功した場合、dataを返す', async () => {
      const mockData = { test: 'data' };
      const requestFn = vi.fn().mockResolvedValue(mockData);

      const result = await apiRequest(requestFn);

      expect(result).toEqual({ data: mockData });
      expect(result.error).toBeUndefined();
    });

    it('ApiErrorの場合、errorを返す', async () => {
      const apiError = new ApiError(ApiErrorType.VALIDATION_ERROR, 'Validation failed');
      const requestFn = vi.fn().mockRejectedValue(apiError);

      const result = await apiRequest(requestFn);

      expect(result).toEqual({ error: apiError });
      expect(result.data).toBeUndefined();
    });

    it('非ApiErrorの場合、DESERIALIZATION_ERRORに変換してerrorを返す', async () => {
      const error = new Error('Network Error');
      const requestFn = vi.fn().mockRejectedValue(error);

      const result = await apiRequest(requestFn);

      expect(result.error).toBeInstanceOf(ApiError);
      expect(result.error?.type).toBe(ApiErrorType.DESERIALIZATION_ERROR);
      expect(result.error?.message).toBe('Network Error');
    });

    it('その他のErrorの場合、メッセージを含むApiErrorを返す', async () => {
      const error = new Error('Custom error message');
      const requestFn = vi.fn().mockRejectedValue(error);

      const result = await apiRequest(requestFn);

      expect(result.error).toBeInstanceOf(ApiError);
      expect(result.error?.type).toBe(ApiErrorType.DESERIALIZATION_ERROR);
      expect(result.error?.message).toBe('Custom error message');
    });

    it('非Errorオブジェクトの場合、デフォルトメッセージのApiErrorを返す', async () => {
      const error = 'string error';
      const requestFn = vi.fn().mockRejectedValue(error);

      const result = await apiRequest(requestFn);

      expect(result.error).toBeInstanceOf(ApiError);
      expect(result.error?.type).toBe(ApiErrorType.DESERIALIZATION_ERROR);
      expect(result.error?.message).toBe('不明なエラーが発生しました');
    });
  });
});
