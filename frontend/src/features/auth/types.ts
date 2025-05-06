export interface UserInfo {
  authCode: string | null;
  name?: string;
  email?: string;
  id?: string;
}

export interface AuthContextType {
  user: UserInfo | null;
  setUser: (user: UserInfo | null) => void;
  login: (credentials?: any) => Promise<void>;
  logout: () => void;
  isAuthenticated: boolean;
}
