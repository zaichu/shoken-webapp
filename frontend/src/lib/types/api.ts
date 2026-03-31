export enum ApiErrorType {
  REQUEST_ERROR = 'REQUEST_ERROR',
  NETWORK_ERROR = 'NETWORK_ERROR',
  TIMEOUT_ERROR = 'TIMEOUT_ERROR',
  RESPONSE_ERROR = 'RESPONSE_ERROR',
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  AUTHENTICATION_ERROR = 'AUTHENTICATION_ERROR',
  AUTHORIZATION_ERROR = 'AUTHORIZATION_ERROR',
  NOT_FOUND_ERROR = 'NOT_FOUND_ERROR',
  SERVER_ERROR = 'SERVER_ERROR',
  JSON_ERROR = 'JSON_ERROR',
  DESERIALIZATION_ERROR = 'DESERIALIZATION_ERROR',
  UNKNOWN_ERROR = 'UNKNOWN_ERROR',
}

interface ApiErrorDetails {
  code?: string;
  field?: string;
  constraints?: Record<string, string>;
  context?: Record<string, unknown>;
}

export class ApiError extends Error {
  public readonly type: ApiErrorType;
  public readonly statusCode?: number;
  public readonly details?: ApiErrorDetails;
  public readonly timestamp: Date;
  public requestUrl?: string;
  public requestMethod?: string;

  constructor(
    type: ApiErrorType,
    message: string,
    statusCode?: number,
    details?: ApiErrorDetails,
    requestUrl?: string,
    requestMethod?: string
  ) {
    super(message);
    this.type = type;
    this.statusCode = statusCode;
    this.details = details;
    this.timestamp = new Date();
    this.name = 'ApiError';
    this.requestUrl = requestUrl;
    this.requestMethod = requestMethod;

    // スタックトレースを保持
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, ApiError);
    }
  }

  /**
   * エラーの詳細情報を含む文字列を返す
   */
  public toDetailedString(): string {
    const parts = [`[${this.type}] ${this.message}`];

    if (this.statusCode) {
      parts.push(`Status: ${this.statusCode}`);
    }

    if (this.requestUrl) {
      parts.push(`URL: ${this.requestMethod || 'GET'} ${this.requestUrl}`);
    }

    if (this.details) {
      parts.push(`Details: ${JSON.stringify(this.details)}`);
    }

    parts.push(`Timestamp: ${this.timestamp.toISOString()}`);

    return parts.join(' | ');
  }

  /**
   * ユーザーフレンドリーなエラーメッセージを返す
   */
  public getUserMessage(): string {
    switch (this.type) {
      case ApiErrorType.NETWORK_ERROR:
        return 'ネットワーク接続を確認してください';
      case ApiErrorType.TIMEOUT_ERROR:
        return 'リクエストがタイムアウトしました。もう一度お試しください';
      case ApiErrorType.AUTHENTICATION_ERROR:
        return 'ログインが必要です';
      case ApiErrorType.AUTHORIZATION_ERROR:
        return 'このリソースへのアクセス権限がありません';
      case ApiErrorType.NOT_FOUND_ERROR:
        return '指定されたリソースが見つかりません';
      case ApiErrorType.VALIDATION_ERROR:
        return '入力内容を確認してください';
      case ApiErrorType.SERVER_ERROR:
        return 'サーバーエラーが発生しました。しばらくしてから再度お試しください';
      case ApiErrorType.DESERIALIZATION_ERROR:
        return '銘柄情報の取得に失敗しました。時間をおいて再度お試しください。';
      default:
        return this.message;
    }
  }

  /**
   * HTTPレスポンスからApiErrorを生成
   */
  static fromHttpResponse(
    status: number,
    data: unknown,
    url?: string,
    method?: string
  ): ApiError {
    let errorType: ApiErrorType;
    let message: string;

    switch (status) {
      case 400: {
        errorType = ApiErrorType.VALIDATION_ERROR;
        const data400 = data as { error?: { message?: string } } | null;
        message = data400?.error?.message ?? 'リクエストが不正です';
        break;
      }
      case 401:
        errorType = ApiErrorType.AUTHENTICATION_ERROR;
        message = '認証が必要です';
        break;
      case 403:
        errorType = ApiErrorType.AUTHORIZATION_ERROR;
        message = 'アクセスが拒否されました';
        break;
      case 404:
        errorType = ApiErrorType.NOT_FOUND_ERROR;
        message = 'リソースが見つかりません';
        break;
      case 500:
      case 502:
      case 503:
      case 504:
        errorType = ApiErrorType.SERVER_ERROR;
        message = 'サーバーエラーが発生しました';
        break;
      default:
        errorType = ApiErrorType.RESPONSE_ERROR;
        message = `エラーが発生しました (ステータス: ${status})`;
    }

    const details: ApiErrorDetails = {};
    if (data) {
      if (typeof data === 'object') {
        Object.assign(details, data);
      } else {
        details.context = { responseData: data };
      }
    }

    return new ApiError(errorType, message, status, details, url, method);
  }

  /**
   * 再試行可能なエラーかどうかを判定
   */
  public isRetryable(): boolean {
    return [
      ApiErrorType.NETWORK_ERROR,
      ApiErrorType.TIMEOUT_ERROR,
      ApiErrorType.SERVER_ERROR
    ].includes(this.type);
  }

  /**
   * エラーがユーザーの操作で解決可能かどうかを判定
   */
  public isUserActionable(): boolean {
    return [
      ApiErrorType.VALIDATION_ERROR,
      ApiErrorType.AUTHENTICATION_ERROR,
      ApiErrorType.AUTHORIZATION_ERROR
    ].includes(this.type);
  }
}
