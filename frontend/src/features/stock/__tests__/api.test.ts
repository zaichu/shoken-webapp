import { describe, it, expect, vi, beforeEach } from 'vitest';
import axios from 'axios';
import { fetchStockData, apiRequest } from '../api';
import { apiClient } from '../../../lib/api/client';
import { ApiError, ApiErrorType } from '../../../lib/types/api';

vi.mock('../../../lib/api/client', () => ({
  apiClient: {
    get: vi.fn(),
  }
}));

vi.mock('axios');

describe('Stock API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('fetchStockData', () => {
    it('ApiErrorの場合、同じエラーをそのまま再スローする', async () => {
      const apiError = new ApiError(ApiErrorType.NOT_FOUND_ERROR, 'リソースが見つかりません');
      vi.mocked(axios.isAxiosError).mockReturnValue(false);
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(apiError);
      const fromAxiosErrorSpy = vi.spyOn(ApiError, 'fromAxiosError');

      await expect(fetchStockData('1234')).rejects.toBe(apiError);
      expect(fromAxiosErrorSpy).not.toHaveBeenCalled();
    });

    it('Axiosエラーの場合、ApiErrorを投げる', async () => {
      const axiosError = new Error('Network Error');
      vi.mocked(axios.isAxiosError).mockReturnValue(true);
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(axiosError);

      // Mock ApiError.fromAxiosError
      const mockApiError = new ApiError(ApiErrorType.NETWORK_ERROR, 'Network Error');
      vi.spyOn(ApiError, 'fromAxiosError').mockReturnValue(mockApiError);

      await expect(fetchStockData('1234')).rejects.toThrow(ApiError);
      expect(ApiError.fromAxiosError).toHaveBeenCalledWith(axiosError);
    });

    it('その他のエラーの場合、デフォルトのApiErrorを投げる', async () => {
      const error = new Error('Unknown Error');
      vi.mocked(axios.isAxiosError).mockReturnValue(false);
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

    it('Axiosエラーの場合、ApiErrorに変換してerrorを返す', async () => {
      const axiosError = new Error('Network Error');
      vi.mocked(axios.isAxiosError).mockReturnValue(true);

      // Mock ApiError.fromAxiosError
      const mockApiError = new ApiError(ApiErrorType.NETWORK_ERROR, 'Network Error');
      vi.spyOn(ApiError, 'fromAxiosError').mockReturnValue(mockApiError);

      const requestFn = vi.fn().mockRejectedValue(axiosError);

      const result = await apiRequest(requestFn);

      expect(result).toEqual({ error: mockApiError });
      expect(ApiError.fromAxiosError).toHaveBeenCalledWith(axiosError);
    });

    it('その他のErrorの場合、メッセージを含むApiErrorを返す', async () => {
      const error = new Error('Custom error message');
      vi.mocked(axios.isAxiosError).mockReturnValue(false);
      const requestFn = vi.fn().mockRejectedValue(error);

      const result = await apiRequest(requestFn);

      expect(result.error).toBeInstanceOf(ApiError);
      expect(result.error?.type).toBe(ApiErrorType.DESERIALIZATION_ERROR);
      expect(result.error?.message).toBe('Custom error message');
    });

    it('非Errorオブジェクトの場合、デフォルトメッセージのApiErrorを返す', async () => {
      const error = 'string error';
      vi.mocked(axios.isAxiosError).mockReturnValue(false);
      const requestFn = vi.fn().mockRejectedValue(error);

      const result = await apiRequest(requestFn);

      expect(result.error).toBeInstanceOf(ApiError);
      expect(result.error?.type).toBe(ApiErrorType.DESERIALIZATION_ERROR);
      expect(result.error?.message).toBe('不明なエラーが発生しました');
    });
  });
});
