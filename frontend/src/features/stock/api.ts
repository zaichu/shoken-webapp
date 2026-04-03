import { StockData } from './types';
import { apiClient } from '../../lib/api/client';
import { ApiError, ApiErrorType } from '../../lib/types/api';

/**
 * 銘柄データを取得する
 * @param query 銘柄コードまたは銘柄名
 * @returns 銘柄データ
 */
export async function fetchStockData(query: string): Promise<StockData> {
  try {
    const response = await apiClient.get<StockData>(`/stocks/${query}`);
    return response;
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }

    throw new ApiError(
      ApiErrorType.DESERIALIZATION_ERROR,
      '銘柄データの読み込みに失敗しました。データ形式が変更された可能性があります。'
    );
  }
}

/**
 * APIリクエストのラッパー関数
 * @param request 非同期リクエスト関数
 * @returns リクエスト結果またはエラー
 */
export async function apiRequest<T>(
  request: () => Promise<T>
): Promise<{ data?: T; error?: ApiError }> {
  try {
    const data = await request();
    return { data };
  } catch (error) {
    if (error instanceof ApiError) {
      return { error };
    }

    const apiError = new ApiError(
      ApiErrorType.DESERIALIZATION_ERROR,
      error instanceof Error ? error.message : '不明なエラーが発生しました'
    );

    return { error: apiError };
  }
}
