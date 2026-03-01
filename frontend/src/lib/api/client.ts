import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import { ApiError, ApiErrorType } from '../types/api';

// Axiosの型拡張
declare module 'axios' {
  interface InternalAxiosRequestConfig {
    metadata?: { startTime: number };
  }
}

interface RetryConfig {
  maxRetries: number;
  retryDelay: number;
  retryDelayMultiplier: number;
  shouldRetry?: (error: ApiError) => boolean;
}

interface ApiClientConfig {
  baseURL?: string;
  timeout?: number;
  retry?: Partial<RetryConfig>;
  headers?: Record<string, string>;
}

// デフォルトタイムアウト: fly.ioの起動待ち時間を考慮
const DEFAULT_TIMEOUT_MS = 30_000;
// デフォルトリトライ設定
const DEFAULT_MAX_RETRIES = 3;
const DEFAULT_RETRY_DELAY_MS = 1000;
const DEFAULT_RETRY_DELAY_MULTIPLIER = 2;

const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: DEFAULT_MAX_RETRIES,
  retryDelay: DEFAULT_RETRY_DELAY_MS,
  retryDelayMultiplier: DEFAULT_RETRY_DELAY_MULTIPLIER,
  shouldRetry: (error: ApiError) => error.isRetryable(),
};

class ApiClient {
  private client: AxiosInstance;
  private retryConfig: RetryConfig;

  constructor(config: ApiClientConfig = {}) {
    const {
      baseURL = import.meta.env.VITE_SHOKEN_WEBAPI_API_URL,
      timeout = DEFAULT_TIMEOUT_MS,
      retry = {},
      headers = {},
    } = config;

    this.retryConfig = { ...DEFAULT_RETRY_CONFIG, ...retry };

    this.client = axios.create({
      baseURL,
      timeout,
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        ...headers,
      },
    });

    this.setupInterceptors();
  }

  private setupInterceptors(): void {
    // リクエストインターセプター
    this.client.interceptors.request.use(
      (config) => {
        // リクエスト開始時刻を記録
        config.metadata = { startTime: Date.now() };
        // FormDataの場合はContent-Typeを削除し、axiosがboundary付きで自動設定するようにする
        if (config.data instanceof FormData) {
          delete config.headers['Content-Type'];
        }
        return config;
      },
      (error) => {
        return Promise.reject(this.handleError(error));
      }
    );

    // レスポンスインターセプター
    this.client.interceptors.response.use(
      (response) => {
        return response;
      },
      async (error) => {
        const apiError = this.handleError(error);

        // 再試行ロジック
        if (this.shouldRetry(error.config, apiError)) {
          return this.retryRequest(error.config, apiError);
        }

        return Promise.reject(apiError);
      }
    );
  }

  private handleError(error: unknown): ApiError {
    if (axios.isAxiosError(error)) {
      const apiError = ApiError.fromAxiosError(error);
      return apiError;
    }

    if (error instanceof Error) {
      return new ApiError(
        ApiErrorType.UNKNOWN_ERROR,
        error.message
      );
    }

    return new ApiError(
      ApiErrorType.UNKNOWN_ERROR,
      '不明なエラーが発生しました'
    );
  }

  private shouldRetry(config: AxiosRequestConfig & { retryCount?: number }, error: ApiError): boolean {
    if (!config || config.retryCount === undefined) {
      config.retryCount = 0;
    }

    return (
      config.retryCount < this.retryConfig.maxRetries &&
      this.retryConfig.shouldRetry!(error)
    );
  }

  private async retryRequest(
    config: AxiosRequestConfig & { retryCount?: number },
    error: ApiError
  ): Promise<AxiosResponse> {
    config.retryCount = (config.retryCount || 0) + 1;
    
    const delay = this.calculateRetryDelay(config.retryCount);
    console.warn(`Retrying request (${config.retryCount}/${this.retryConfig.maxRetries}) after ${delay}ms:`, error.message);
    
    await this.sleep(delay);
    
    return this.client.request(config);
  }

  private calculateRetryDelay(retryCount: number): number {
    return this.retryConfig.retryDelay * Math.pow(this.retryConfig.retryDelayMultiplier, retryCount - 1);
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  // HTTPメソッド
  async get<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.get<T>(url, config);
    return response.data;
  }

  async post<T>(url: string, data?: unknown, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.post<T>(url, data, config);
    return response.data;
  }

  async put<T>(url: string, data?: unknown, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.put<T>(url, data, config);
    return response.data;
  }

  async patch<T>(url: string, data?: unknown, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.patch<T>(url, data, config);
    return response.data;
  }

  async delete<T>(url: string, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.delete<T>(url, config);
    return response.data;
  }

  // バッチリクエスト
  async batch<T extends readonly unknown[]>(
    requests: {
      [K in keyof T]: () => Promise<T[K]>
    }
  ): Promise<T> {
    const promises = requests.map(requestFn => requestFn());
    return Promise.all(promises) as unknown as Promise<T>;
  }

  // キャンセル可能なリクエスト
  createCancelableRequest<T>(
    requestFn: (signal: AbortSignal) => Promise<T>
  ): {
    promise: Promise<T>;
    cancel: () => void;
  } {
    const controller = new AbortController();
    
    // Axiosリクエストをキャンセル可能にするラッパー
    const wrappedPromise = requestFn(controller.signal).catch(error => {
      if (controller.signal.aborted) {
        throw new ApiError(
          ApiErrorType.REQUEST_ERROR,
          'リクエストがキャンセルされました'
        );
      }
      throw error;
    });
    
    return {
      promise: wrappedPromise,
      cancel: () => controller.abort(),
    };
  }
}

// シングルトンインスタンスの遅延初期化
let _apiClient: ApiClient | null = null;

export function getApiClient(): ApiClient {
  if (!_apiClient) {
    _apiClient = new ApiClient();
  }
  return _apiClient;
}

// 後方互換性のため
export const apiClient = {
  get: <T>(url: string, config?: AxiosRequestConfig) => getApiClient().get<T>(url, config),
  post: <T>(url: string, data?: unknown, config?: AxiosRequestConfig) => getApiClient().post<T>(url, data, config),
  put: <T>(url: string, data?: unknown, config?: AxiosRequestConfig) => getApiClient().put<T>(url, data, config),
  patch: <T>(url: string, data?: unknown, config?: AxiosRequestConfig) => getApiClient().patch<T>(url, data, config),
  delete: <T>(url: string, config?: AxiosRequestConfig) => getApiClient().delete<T>(url, config),
  batch: <T extends readonly unknown[]>(requests: { [K in keyof T]: () => Promise<T[K]> }) => getApiClient().batch<T>(requests),
  createCancelableRequest: <T>(requestFn: (signal: AbortSignal) => Promise<T>) => getApiClient().createCancelableRequest<T>(requestFn),
};

// 型付きAPIクライアントのファクトリー関数
export function createApiClient(config?: ApiClientConfig): ApiClient {
  return new ApiClient(config);
}

// テスト環境でのシングルトンリセット用
export function resetApiClient(): void {
  _apiClient = null;
}

// エクスポート
export { ApiClient, type ApiClientConfig, type RetryConfig };
