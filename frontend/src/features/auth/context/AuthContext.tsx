import { useState, useEffect, useCallback, useRef, ReactNode } from 'react';
import { UserInfo } from '../types';
import { AuthContext } from './context';
import { apiClient } from '@/lib/api/client';
import { useIdleTimer } from '../hooks/useIdleTimer';

// アイドルタイムアウト: 30分
const IDLE_TIMEOUT = 30 * 60 * 1000;
// 初期表示をブロックする認証確認は長く待たずにフォールバックする
const AUTH_SESSION_CHECK_TIMEOUT = 5000;

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  // ログアウト時に呼び出されるコールバックのリスト
  const logoutCallbacksRef = useRef<Set<() => void>>(new Set());

  // 初期化時にバックエンドからセッションを確認
  useEffect(() => {
    const checkSession = async () => {
      try {
        // バックエンドからユーザー情報を取得（Cookieベースの認証）
        const userInfo = await apiClient.get<UserInfo>('/auth/me', {
          withCredentials: true,
          timeout: AUTH_SESSION_CHECK_TIMEOUT,
        });
        setUser(userInfo);
      } catch {
        // セッションが無効な場合
        setUser(null);
      } finally {
        setIsLoading(false);
      }
    };

    // URLパラメータでログイン成功を検知
    const params = new URLSearchParams(window.location.search);
    if (params.get('login') === 'success') {
      // ログイン成功後のURLパラメータをクリア
      window.history.replaceState({}, '', window.location.pathname);
    }

    checkSession();
  }, []);

  const login = () => {
    // バックエンドの認証エンドポイントに直接リダイレクト
    // バックエンドがGoogleの認証ページにリダイレクトする
    const apiBaseUrl = import.meta.env.VITE_SHOKEN_WEBAPI_API_URL;
    window.location.href = `${apiBaseUrl}/auth/google`;
  };

  const logout = useCallback(async () => {
    // 登録されたコールバックを先に実行（状態クリア用）
    logoutCallbacksRef.current.forEach(callback => callback());
    try {
      await apiClient.post('/auth/logout', {}, {
        withCredentials: true,
      });
    } finally {
      setUser(null);
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
    try {
      await apiClient.delete('/auth/delete-account', {
        withCredentials: true,
      });
    } finally {
      setUser(null);
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
