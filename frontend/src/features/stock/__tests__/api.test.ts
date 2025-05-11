import { describe, it, expect, vi, beforeEach } from 'vitest';
import axios from 'axios';
import { fetchStockData, apiRequest } from '../api';
import { apiClient } from '../../../lib/api/client';
import { ApiError, ApiErrorType } from '../../../lib/types/api';
import { StockData } from '../types';

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
    it('株式データを正常に取得できる', async () => {
      const mockStockData: StockData = {
        securityCode: '1234',
        securityNameJa: 'テスト株式',
        sectorName: 'テクノロジー',
        marketCode: 'TSE',
        marketName: '東証プライム',
        priceYesterday: 1000,
        currentPrice: 1050,
        changeFromYesterday: 50,
        percentChangeFromYesterday: 5.0,
        currentPriceTime: '2024-01-01T12:00:00Z',
        volume: 1000000,
        bidPrice: 1048,
        bidTime: '2024-01-01T12:00:00Z',
        askPrice: 1052,
        askTime: '2024-01-01T12:00:00Z',
        tradingValue: 1050000000,
        priceEarningsRatio: 15.5,
        priceBookValueRatio: 1.2,
        returnOnEquity: 8.5,
        capitalAdequacyRatio: 45.0,
        stockLabelsJa: [],
        description: 'テスト企業の説明',
        website: 'https://example.com',
        numberOfShares: 1000000,
      };

      (apiClient.get as ReturnType<typeof vi.fn>).mockResolvedValue({
        data: mockStockData
      });

      const result = await fetchStockData('1234');

      expect(apiClient.get).toHaveBeenCalledWith('/stock/1234');
      expect(result).toEqual(mockStockData);
    });

    it('Axiosエラーの場合、ApiErrorを投げる', async () => {
      const axiosError = new Error('Network Error');
      (axios.isAxiosError as ReturnType<typeof vi.fn>).mockReturnValue(true);
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(axiosError);
      
      // Mock ApiError.fromAxiosError
      const mockApiError = new ApiError(ApiErrorType.CONNECTION_ERROR, 'Network Error');
      vi.spyOn(ApiError, 'fromAxiosError').mockReturnValue(mockApiError);

      await expect(fetchStockData('1234')).rejects.toThrow(ApiError);
      expect(ApiError.fromAxiosError).toHaveBeenCalledWith(axiosError);
    });

    it('その他のエラーの場合、デフォルトのApiErrorを投げる', async () => {
      const error = new Error('Unknown Error');
      (axios.isAxiosError as ReturnType<typeof vi.fn>).mockReturnValue(false);
      (apiClient.get as ReturnType<typeof vi.fn>).mockRejectedValue(error);

      await expect(fetchStockData('1234')).rejects.toThrow('株式データの処理に失敗しました');
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
      (axios.isAxiosError as ReturnType<typeof vi.fn>).mockReturnValue(true);
      
      // Mock ApiError.fromAxiosError
      const mockApiError = new ApiError(ApiErrorType.CONNECTION_ERROR, 'Network Error');
      vi.spyOn(ApiError, 'fromAxiosError').mockReturnValue(mockApiError);
      
      const requestFn = vi.fn().mockRejectedValue(axiosError);

      const result = await apiRequest(requestFn);

      expect(result).toEqual({ error: mockApiError });
      expect(ApiError.fromAxiosError).toHaveBeenCalledWith(axiosError);
    });

    it('その他のErrorの場合、メッセージを含むApiErrorを返す', async () => {
      const error = new Error('Custom error message');
      (axios.isAxiosError as ReturnType<typeof vi.fn>).mockReturnValue(false);
      const requestFn = vi.fn().mockRejectedValue(error);

      const result = await apiRequest(requestFn);

      expect(result.error).toBeInstanceOf(ApiError);
      expect(result.error?.type).toBe(ApiErrorType.DESERIALIZATION_ERROR);
      expect(result.error?.message).toBe('Custom error message');
    });

    it('非Errorオブジェクトの場合、デフォルトメッセージのApiErrorを返す', async () => {
      const error = 'string error';
      (axios.isAxiosError as ReturnType<typeof vi.fn>).mockReturnValue(false);
      const requestFn = vi.fn().mockRejectedValue(error);

      const result = await apiRequest(requestFn);

      expect(result.error).toBeInstanceOf(ApiError);
      expect(result.error?.type).toBe(ApiErrorType.DESERIALIZATION_ERROR);
      expect(result.error?.message).toBe('不明なエラーが発生しました');
    });
  });
});
