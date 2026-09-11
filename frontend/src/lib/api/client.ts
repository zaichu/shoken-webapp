import { ApiError, ApiErrorType } from '../types/api';

export interface RetryConfig {
  maxRetries: number;
  retryDelay: number;
  retryDelayMultiplier: number;
  shouldRetry?: (error: ApiError) => boolean;
}

export interface RequestConfig {
  params?: Record<string, string | number | boolean | undefined>;
  headers?: Record<string, string>;
  withCredentials?: boolean;
  signal?: AbortSignal;
  retry?: Partial<RetryConfig>;
}

interface ApiClientConfig {
  baseURL?: string;
  timeout?: number;
  retry?: Partial<RetryConfig>;
  headers?: Record<string, string>;
}

// 低速な外部 API と書き込み処理で維持する従来の設定
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
    if (!qs) return url;
    return `${url}${url.includes('?') ? '&' : '?'}${qs}`;
  }

  private async executeRequest<T>(
    method: string,
    path: string,
    data?: unknown,
    config: RequestConfig = {},
    retryCount = 0
  ): Promise<T> {
    const { params, withCredentials, signal: externalSignal, retry: perRequestRetry } = config;
    const effectiveRetryConfig = perRequestRetry
      ? { ...this.retryConfig, ...perRequestRetry }
      : this.retryConfig;

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

    const headers: Record<string, string> = { ...this.defaultHeaders, ...config.headers };
    let body: BodyInit | undefined;

    if (data instanceof FormData) {
      // FormDataの場合はContent-Typeを設定せず、ブラウザが自動設定するようにする
      body = data;
    } else if (data !== undefined) {
      // JSONボディがある場合のみContent-Typeを付与（GET/DELETEではpreflight防止のため付与しない）
      headers['Content-Type'] = 'application/json';
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

      const text = await response.text();
      return (text ? (JSON.parse(text) as T) : undefined as T);
    } catch (error) {
      let apiError: ApiError;
      if (error instanceof ApiError) {
        apiError = error;
      } else if (error instanceof DOMException && error.name === 'AbortError') {
        const isTimeout = controller.signal.reason === 'timeout';
        apiError = new ApiError(
          isTimeout ? ApiErrorType.TIMEOUT_ERROR : ApiErrorType.REQUEST_ERROR,
          isTimeout ? 'リクエストがタイムアウトしました' : 'リクエストがキャンセルされました',
          undefined,
          undefined,
          path,
          method
        );
      } else {
        apiError = new ApiError(
          ApiErrorType.NETWORK_ERROR,
          'ネットワークエラーが発生しました',
          undefined,
          undefined,
          path,
          method
        );
      }

      // 統一リトライ判定
      if (
        retryCount < effectiveRetryConfig.maxRetries &&
        effectiveRetryConfig.shouldRetry!(apiError)
      ) {
        const delay = effectiveRetryConfig.retryDelay *
          Math.pow(effectiveRetryConfig.retryDelayMultiplier, retryCount);
        console.warn(
          'Retrying request',
          retryCount + 1,
          'of',
          effectiveRetryConfig.maxRetries,
          'after',
          delay,
          'ms:',
          apiError.message
        );
        await new Promise(resolve => setTimeout(resolve, delay));
        return this.executeRequest<T>(method, path, data, config, retryCount + 1);
      }
      throw apiError;
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
}

// シングルトンインスタンスの遅延初期化
let _apiClient: ApiClient | null = null;

function getApiClient(): ApiClient {
  if (!_apiClient) {
    _apiClient = new ApiClient();
  }
  return _apiClient;
}

// 通常の一覧・検索は最大約21秒（10秒 × 2試行 + 1秒待機）。
// 認証の専用クライアントと書き込み処理にはこの設定を適用しない。
let _readApiClient: ApiClient | null = null;

function getReadApiClient(): ApiClient {
  if (!_readApiClient) {
    _readApiClient = new ApiClient({
      timeout: 10_000,
      retry: { maxRetries: 1 },
    });
  }
  return _readApiClient;
}

// 後方互換性のため
export const apiClient = {
  get: <T>(url: string, config?: RequestConfig) => getReadApiClient().get<T>(url, config),
  post: <T>(url: string, data?: unknown, config?: RequestConfig) => getApiClient().post<T>(url, data, config),
  put: <T>(url: string, data?: unknown, config?: RequestConfig) => getApiClient().put<T>(url, data, config),
  patch: <T>(url: string, data?: unknown, config?: RequestConfig) => getApiClient().patch<T>(url, data, config),
  delete: <T>(url: string, config?: RequestConfig) => getApiClient().delete<T>(url, config),
};

// 型付きAPIクライアントのファクトリー関数
export function createApiClient(config?: ApiClientConfig): ApiClient {
  return new ApiClient(config);
}
