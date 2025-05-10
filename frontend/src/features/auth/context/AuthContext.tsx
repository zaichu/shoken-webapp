import { useState, useEffect, ReactNode } from 'react';
import { UserInfo } from '../types';
import { AuthContext } from './context';

function initializeUserInfo(): UserInfo | null {
  return getUserInfoFromStorage() || null;
}

function getUserInfoFromStorage(): UserInfo | null {
  const storedInfo = localStorage.getItem('user_info');
  if (storedInfo) {
    try {
      return JSON.parse(storedInfo);
    } catch (e) {
      return null;
    }
  }
  return null;
}

function saveUserInfoToStorage(userInfo: UserInfo | null) {
  if (userInfo) {
    localStorage.setItem('user_info', JSON.stringify(userInfo));
  } else {
    localStorage.removeItem('user_info');
  }
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserInfo | null>(initializeUserInfo());

  useEffect(() => {
    saveUserInfoToStorage(user);
  }, [user]);

  useEffect(() => {
    const queryParams = new URLSearchParams(window.location.search);
    const authCode = queryParams.get('code');

    if (authCode && !user?.authCode) {
      const newInfo: UserInfo = {
        authCode,
      };
      setUser(newInfo);

      window.history.replaceState({}, '', window.location.pathname);
    }
  }, [user]);

  const login = async () => {
    try {
      const mockUser: UserInfo = {
        authCode: 'mock-auth-code',
        id: '1',
        name: 'テストユーザー',
        email: 'test@example.com'
      };

      setUser(mockUser);
    } catch (error) {
      console.error('Login failed:', error);
      throw error;
    }
  };

  const logout = () => {
    setUser(null);
    localStorage.removeItem('user_info');
  };

  return (
    <AuthContext.Provider value={{
      user,
      setUser,
      login,
      logout,
      isAuthenticated: !!user
    }}>
      {children}
    </AuthContext.Provider>
  );
}
