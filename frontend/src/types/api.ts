// src/types/api.ts
import { AxiosError } from 'axios';

export enum ApiErrorType {
  REQUEST_ERROR = 'REQUEST_ERROR',
  FETCH_ERROR = 'FETCH_ERROR',
  RESPONSE_ERROR = 'RESPONSE_ERROR',
  JSON_ERROR = 'JSON_ERROR',
  DESERIALIZATION_ERROR = 'DESERIALIZATION_ERROR',
}

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
        'サーバーにリクエストが到達しませんでした'
      );
    }

    if (error.response) {
      return new ApiError(
        ApiErrorType.RESPONSE_ERROR,
        `サーバーがステータス ${error.response.status} で応答しました`
      );
    }

    return new ApiError(ApiErrorType.FETCH_ERROR, error.message);
  }
}
