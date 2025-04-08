import axios from 'axios';
import { StockData } from '../data/stock';
import { ApiError, ApiErrorType } from '../types/api';

// APIクライアントの作成
const apiClient = axios.create({
  baseURL: import.meta.env.VITE_SHOKEN_WEBAPI_API_URL,
  timeout: 10000, // 10秒でタイムアウト
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  }
});

/**
 * 銘柄データを取得する
 * @param query 銘柄コードまたは銘柄名
 * @returns 銘柄データ
 */
export async function fetchStockData(query: string): Promise<StockData> {
  try {
    const response = await apiClient.get<StockData>(`/stock/${query}`);
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw ApiError.fromAxiosError(error);
    }

    throw new ApiError(
      ApiErrorType.DESERIALIZATION_ERROR,
      '株式データの処理に失敗しました'
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

    if (axios.isAxiosError(error)) {
      return { error: ApiError.fromAxiosError(error) };
    }

    const apiError = new ApiError(
      ApiErrorType.DESERIALIZATION_ERROR,
      error instanceof Error ? error.message : '不明なエラーが発生しました'
    );

    return { error: apiError };
  }
}
