import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { UserInfo } from '../data/userInfo';

type UserContextType = {
  userInfo: UserInfo;
  setUserInfo: (info: UserInfo) => void;
};

const UserContext = createContext<UserContextType | undefined>(undefined);

export function UserInfoProvider({ children }: { children: ReactNode }) {
  const [userInfo, setUserInfo] = useState<UserInfo>(initializeUserInfo());

  // URLからの認証コードの取得と処理
  useEffect(() => {
    const queryParams = new URLSearchParams(window.location.search);
    const authCode = queryParams.get('code');
    
    if (authCode) {
      const newInfo: UserInfo = {
        ...userInfo,
        authCode: authCode,
      };
      setUserInfo(newInfo);
      saveUserInfoToStorage(newInfo);
      
      // URLからパラメータを削除
      window.history.replaceState({}, '', window.location.pathname);
    }
  }, []);

  return (
    <UserContext.Provider value={{ userInfo, setUserInfo }}>
      {children}
    </UserContext.Provider>
  );
}

export function useUserInfo() {
  const context = useContext(UserContext);
  if (context === undefined) {
    throw new Error('useUserInfo must be used within a UserInfoProvider');
  }
  return context;
}

function initializeUserInfo(): UserInfo {
  return getUserInfoFromStorage() || { authCode: null };
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

function saveUserInfoToStorage(userInfo: UserInfo) {
  localStorage.setItem('user_info', JSON.stringify(userInfo));
}
