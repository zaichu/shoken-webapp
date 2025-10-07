import { useContext } from 'react';
import { ResizeContext } from '@/contexts/ResizeContextDefinition';

/**
 * forceResize値を取得するカスタムフック
 */
export const useForceResize = () => {
  const context = useContext(ResizeContext);
  return context?.forceResize;
};

/**
 * リサイズをトリガーするカスタムフック
 */
export const useTriggerResize = () => {
  const context = useContext(ResizeContext);
  return context?.triggerResize;
};
