import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';

// リサイズコンテキストの型定義
interface ResizeContextValue {
  forceResize: number;
  triggerResize: () => void;
}

// リサイズコンテキストの作成
const ResizeContext = createContext<ResizeContextValue | null>(null);

// カスタムフック
export const useForceResize = () => {
  const context = useContext(ResizeContext);
  return context?.forceResize;
};

export const useTriggerResize = () => {
  const context = useContext(ResizeContext);
  return context?.triggerResize;
};

// プロバイダーコンポーネント
interface ResizeProviderProps {
  children: ReactNode;
}

export const ResizeProvider: React.FC<ResizeProviderProps> = ({ children }) => {
  const [forceResize, setForceResize] = useState(0);

  const triggerResize = useCallback(() => {
    setForceResize(prev => prev + 1);
  }, []);

  const value: ResizeContextValue = {
    forceResize,
    triggerResize,
  };

  return (
    <ResizeContext.Provider value={value}>
      {children}
    </ResizeContext.Provider>
  );
};
