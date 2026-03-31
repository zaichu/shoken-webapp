import { ApiError, ApiErrorType } from '../types/api';

export interface RequestConfig {
  params?: Record<string, string | number | boolean | undefined>;
  headers?: Record<string, string>;
  withCredentials?: boolean;
  signal?: AbortSignal;
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
  private baseURL: string;
  private timeout: number;
  private retryConfig: RetryConfig;
  private defaultHeaders: Record<string, string>;

  constructor(config: ApiClientConfig = {}) {
    const {
      baseURL = import.meta.env.VITE_SHOKEN_WEBAPI_API_URL ?? '',
      timeout = DEFAULT_TIMEOUT_MS,
      retry = {},
      headers = {},
    } = config;

    this.baseURL = baseURL;
    this.timeout = timeout;
    this.retryConfig = { ...DEFAULT_RETRY_CONFIG, ...retry };
    this.defaultHeaders = {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
      ...headers,
    };
  }

  private buildURL(path: string, params?: RequestConfig['params']): string {
    const url = this.baseURL + path;
    if (!params) return url;
    const searchParams = new URLSearchParams();
    for (const [key, value] of Object.entries(params)) {
      if (value !== undefined) {
        searchParams.set(key, String(value));
      }
    }
    const qs = searchParams.toString();
    return qs ? `${url}?${qs}` : url;
  }

  private async executeRequest<T>(
    method: string,
    path: string,
    data?: unknown,
    config: RequestConfig = {},
    retryCount = 0
  ): Promise<T> {
    const { params, withCredentials, signal: externalSignal } = config;

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort('timeout'), this.timeout);

    // 外部シグナルが既にabort済みならすぐに中断
    if (externalSignal?.aborted) {
      clearTimeout(timeoutId);
      throw new ApiError(
        ApiErrorType.REQUEST_ERROR,
        'リクエストがキャンセルされました',
        undefined,
        undefined,
        path,
        method
      );
    }

    // 外部シグナルのabortをタイムアウトコントローラに伝播
    const onExternalAbort = () => controller.abort(externalSignal?.reason);
    externalSignal?.addEventListener('abort', onExternalAbort);

    const headers: Record<string, string> = { ...this.defaultHeaders, ...(config.headers ?? {}) };
    let body: BodyInit | undefined;

    if (data instanceof FormData) {
      // FormDataの場合はContent-Typeを削除し、ブラウザが自動設定するようにする
      delete headers['Content-Type'];
      body = data;
    } else if (data !== undefined) {
      body = JSON.stringify(data);
    }

    const url = this.buildURL(path, params);

    try {
      const response = await fetch(url, {
        method,
        headers,
        body,
        credentials: withCredentials ? 'include' : 'same-origin',
        signal: controller.signal,
      });

      if (!response.ok) {
        const responseData = await response.json().catch(() => null);
        throw ApiError.fromHttpResponse(response.status, responseData, path, method);
      }

      // 204 No Content
      if (response.status === 204) return undefined as T;

      return (await response.json()) as T;
    } catch (error) {
      if (error instanceof ApiError) {
        // リトライ判定
        if (
          retryCount < this.retryConfig.maxRetries &&
          this.retryConfig.shouldRetry!(error)
        ) {
          const delay = this.retryConfig.retryDelay *
            Math.pow(this.retryConfig.retryDelayMultiplier, retryCount);
          console.warn(
            `Retrying request (${retryCount + 1}/${this.retryConfig.maxRetries}) after ${delay}ms:`,
            error.message
          );
          await new Promise(resolve => setTimeout(resolve, delay));
          return this.executeRequest<T>(method, path, data, config, retryCount + 1);
        }
        throw error;
      }

      // AbortError: タイムアウトまたはキャンセル
      if (error instanceof DOMException && error.name === 'AbortError') {
        const isTimeout = controller.signal.reason === 'timeout';
        throw new ApiError(
          isTimeout ? ApiErrorType.TIMEOUT_ERROR : ApiErrorType.REQUEST_ERROR,
          isTimeout ? 'リクエストがタイムアウトしました' : 'リクエストがキャンセルされました',
          undefined,
          undefined,
          path,
          method
        );
      }

      // ネットワークエラー
      throw new ApiError(
        ApiErrorType.NETWORK_ERROR,
        'ネットワークエラーが発生しました',
        undefined,
        undefined,
        path,
        method
      );
    } finally {
      clearTimeout(timeoutId);
      externalSignal?.removeEventListener('abort', onExternalAbort);
    }
  }

  // HTTPメソッド
  async get<T>(url: string, config?: RequestConfig): Promise<T> {
    return this.executeRequest<T>('GET', url, undefined, config);
  }

  async post<T>(url: string, data?: unknown, config?: RequestConfig): Promise<T> {
    return this.executeRequest<T>('POST', url, data, config);
  }

  async put<T>(url: string, data?: unknown, config?: RequestConfig): Promise<T> {
    return this.executeRequest<T>('PUT', url, data, config);
  }

  async patch<T>(url: string, data?: unknown, config?: RequestConfig): Promise<T> {
    return this.executeRequest<T>('PATCH', url, data, config);
  }

  async delete<T>(url: string, config?: RequestConfig): Promise<T> {
    return this.executeRequest<T>('DELETE', url, undefined, config);
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

function getApiClient(): ApiClient {
  if (!_apiClient) {
    _apiClient = new ApiClient();
  }
  return _apiClient;
}

// 後方互換性のため
export const apiClient = {
  get: <T>(url: string, config?: RequestConfig) => getApiClient().get<T>(url, config),
  post: <T>(url: string, data?: unknown, config?: RequestConfig) => getApiClient().post<T>(url, data, config),
  put: <T>(url: string, data?: unknown, config?: RequestConfig) => getApiClient().put<T>(url, data, config),
  patch: <T>(url: string, data?: unknown, config?: RequestConfig) => getApiClient().patch<T>(url, data, config),
  delete: <T>(url: string, config?: RequestConfig) => getApiClient().delete<T>(url, config),
  batch: <T extends readonly unknown[]>(requests: { [K in keyof T]: () => Promise<T[K]> }) => getApiClient().batch<T>(requests),
  createCancelableRequest: <T>(requestFn: (signal: AbortSignal) => Promise<T>) => getApiClient().createCancelableRequest<T>(requestFn),
};

// 型付きAPIクライアントのファクトリー関数
export function createApiClient(config?: ApiClientConfig): ApiClient {
  return new ApiClient(config);
}
