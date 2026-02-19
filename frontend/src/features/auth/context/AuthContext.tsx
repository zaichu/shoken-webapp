import { useState, useEffect, useCallback, useRef, ReactNode } from 'react';
import { UserInfo } from '../types';
import { AuthContext } from './context';
import { apiClient, createApiClient } from '@/lib/api/client';
import { useIdleTimer } from '../hooks/useIdleTimer';

// 認証確認専用クライアント設定
// デフォルト設定(timeout=30s, retry=3回, 指数バックオフ)では
// fly.ioコールドスタート時に最大127秒待ちになるため、専用設定で短縮
const AUTH_CHECK_TIMEOUT_MS = 5_000;
const AUTH_CHECK_MAX_RETRIES = 1;
const AUTH_CHECK_RETRY_DELAY_MS = 500;
const AUTH_CHECK_RETRY_DELAY_MULTIPLIER = 1;

const authApiClient = createApiClient({
  timeout: AUTH_CHECK_TIMEOUT_MS,
  retry: {
    maxRetries: AUTH_CHECK_MAX_RETRIES,
    retryDelay: AUTH_CHECK_RETRY_DELAY_MS,
    retryDelayMultiplier: AUTH_CHECK_RETRY_DELAY_MULTIPLIER,
  },
});

// AbortControllerのキャンセル理由（StrictMode再マウント時）
const ABORT_REASON_CLEANUP = 'cleanup';

// アイドルタイムアウト: 30分
const IDLE_TIMEOUT = 30 * 60 * 1000;

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  // ログアウト時に呼び出されるコールバックのリスト
  const logoutCallbacksRef = useRef<Set<() => void>>(new Set());
  // 初期化時にバックエンドからセッションを確認
  // AbortControllerでStrictMode再マウント時の重複リクエストを防止
  useEffect(() => {
    const controller = new AbortController();

    const checkSession = async () => {
      await authApiClient.get<UserInfo>('/auth/me', {
        withCredentials: true,
        signal: controller.signal,
      }).then((userInfo) => {
        if (!controller.signal.aborted) {
          setUser(userInfo);
          setIsLoading(false);
        }
      }).catch(() => {
        // StrictModeクリーンアップによるabortは無視
        if (!controller.signal.aborted) {
          // セッションが無効な場合
          setUser(null);
          setIsLoading(false);
        }
      });
    };

    // URLパラメータでログイン成功を検知
    const params = new URLSearchParams(window.location.search);
    if (params.get('login') === 'success') {
      // ログイン成功後のURLパラメータをクリア
      window.history.replaceState({}, '', window.location.pathname);
    }

    checkSession();

    return () => {
      controller.abort(ABORT_REASON_CLEANUP);
    };
  }, []);

  const login = () => {
    // バックエンドの認証エンドポイントに直接リダイレクト
    // バックエンドがGoogleの認証ページにリダイレクトする
    const apiBaseUrl = import.meta.env.VITE_SHOKEN_WEBAPI_API_URL;
    window.location.assign(`${apiBaseUrl}/auth/google`);
  };

  const logout = useCallback(async () => {
    // 登録されたコールバックを先に実行（状態クリア用）
    logoutCallbacksRef.current.forEach(callback => callback());
    let hasError = false;
    let caughtError: unknown;
    await apiClient.post('/auth/logout', {}, {
      withCredentials: true,
    }).catch((error: unknown) => {
      hasError = true;
      caughtError = error;
    });
    setUser(null);
    if (hasError) {
      throw caughtError;
    }
  }, []);

  // ログアウト時のコールバック登録
  const onLogout = useCallback((callback: () => void) => {
    logoutCallbacksRef.current.add(callback);
    // クリーンアップ関数を返す
    return () => {
      logoutCallbacksRef.current.delete(callback);
    };
  }, []);

  const deleteAccount = useCallback(async () => {
    // 登録されたコールバックを先に実行（状態クリア用）
    logoutCallbacksRef.current.forEach(callback => callback());
    let hasError = false;
    let caughtError: unknown;
    await apiClient.delete('/auth/delete-account', {
      withCredentials: true,
    }).catch((error: unknown) => {
      hasError = true;
      caughtError = error;
    });
    setUser(null);
    if (hasError) {
      throw caughtError;
    }
  }, []);

  // 自動ログアウト処理
  const handleIdle = useCallback(() => {
    if (user) {
      logout();
    }
  }, [user, logout]);

  // アイドルタイマーを設定（ログイン中のみ有効）
  useIdleTimer({
    timeout: IDLE_TIMEOUT,
    onIdle: handleIdle,
    enabled: !!user,
  });

  return (
    <AuthContext.Provider value={{
      user,
      setUser,
      login,
      logout,
      deleteAccount,
      isAuthenticated: !!user,
      isLoading,
      onLogout,
    }}>
      {children}
    </AuthContext.Provider>
  );
}
