import { createContext } from 'react';

// リサイズコンテキストの型定義
export interface ResizeContextValue {
  forceResize: number;
  triggerResize: () => void;
}

// リサイズコンテキストの作成
export const ResizeContext = createContext<ResizeContextValue | null>(null);
