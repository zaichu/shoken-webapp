import React, { useState, useCallback, ReactNode } from 'react';
import { ResizeContext, ResizeContextValue } from './ResizeContextDefinition';

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
