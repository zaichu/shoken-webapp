import { useState, useEffect, ReactNode } from 'react';
import { UserInfo, AuthUrlResponse } from '../types';
import { AuthContext } from './context';
import { apiClient } from '@/lib/api/client';

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // 初期化時にバックエンドからセッションを確認
  useEffect(() => {
    const checkSession = async () => {
      try {
        // バックエンドからユーザー情報を取得（Cookieベースの認証）
        const userInfo = await apiClient.get<UserInfo>('/auth/me', {
          withCredentials: true,
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

  const login = async () => {
    try {
      // Google OAuth認証URLを取得
      const response = await apiClient.get<AuthUrlResponse>('/auth/google', {
        withCredentials: true,
      });

      // Googleの認証ページにリダイレクト
      window.location.href = response.auth_url;
    } catch (error) {
      console.error('ログイン開始に失敗しました:', error);
      throw error;
    }
  };

  const logout = async () => {
    try {
      await apiClient.post('/auth/logout', {}, {
        withCredentials: true,
      });
    } catch (error) {
      console.error('ログアウトAPIエラー:', error);
    } finally {
      setUser(null);
    }
  };

  return (
    <AuthContext.Provider value={{
      user,
      setUser,
      login,
      logout,
      isAuthenticated: !!user,
      isLoading,
    }}>
      {children}
    </AuthContext.Provider>
  );
}
