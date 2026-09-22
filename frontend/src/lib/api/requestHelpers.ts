import type { RequestConfig } from '@/lib/api/client';

// 認証Cookieを送るだけの共通config（deleteAll等で使い回す）
export const withCredentialsConfig: RequestConfig = { withCredentials: true };

export function listRequestConfig<T extends NonNullable<RequestConfig['params']>>(
  params?: T
): RequestConfig {
  return params ? { params, withCredentials: true } : withCredentialsConfig;
}
