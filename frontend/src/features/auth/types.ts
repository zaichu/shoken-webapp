/// バックエンドから返されるユーザー情報
export interface UserInfo {
  id: string;
  email: string;
  name?: string;
  picture_url?: string;
}

export interface LoginCredentials {
  username?: string;
  password?: string;
}

export interface AuthContextType {
  user: UserInfo | null;
  setUser: (user: UserInfo | null) => void;
  login: (credentials?: LoginCredentials) => void;
  logout: () => Promise<void>;
  deleteAccount: () => Promise<void>;
  isAuthenticated: boolean;
  isLoading: boolean;
  // ログアウト時のコールバック登録
  onLogout: (callback: () => void) => () => void;
}

/// Google OAuth認証URL取得のレスポンス
export interface AuthUrlResponse {
  auth_url: string;
}
