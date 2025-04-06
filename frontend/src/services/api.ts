import axios, { AxiosError } from 'axios';
import { StockData } from '../data/stock';

// エラータイプの定義
export enum ApiErrorType {
  REQUEST_ERROR = 'REQUEST_ERROR',
  FETCH_ERROR = 'FETCH_ERROR',
  RESPONSE_ERROR = 'RESPONSE_ERROR',
  JSON_ERROR = 'JSON_ERROR',
  DESERIALIZATION_ERROR = 'DESERIALIZATION_ERROR',
}

// APIエラークラス（Rustのenumに相当）
export class ApiError extends Error {
  type: ApiErrorType;

  constructor(type: ApiErrorType, message: string) {
    super(message);
    this.type = type;
    this.name = 'ApiError';
  }

  static fromAxiosError(error: AxiosError): ApiError {
    if (error.request && !error.response) {
      return new ApiError(
        ApiErrorType.REQUEST_ERROR,
        'Request failed to reach the server'
      );
    }

    if (error.response) {
      return new ApiError(
        ApiErrorType.RESPONSE_ERROR,
        `Server responded with status ${error.response.status}`
      );
    }

    return new ApiError(ApiErrorType.FETCH_ERROR, error.message);
  }
}

const apiClient = axios.create({
  baseURL: import.meta.env.VITE_SHOKEN_WEBAPI_API_URL,
});

export async function fetchStockData(code: string): Promise<StockData> {
  try {
    console.log(code);
    const response = await apiClient.get<StockData>(`/stock/${code}`);
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw ApiError.fromAxiosError(error);
    }

    throw new ApiError(
      ApiErrorType.DESERIALIZATION_ERROR,
      'Failed to process stock data'
    );
  }
}
