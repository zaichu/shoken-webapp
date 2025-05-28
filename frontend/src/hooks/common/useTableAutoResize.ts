import { useCallback, useEffect, useRef, useState } from 'react';

/**
 * テーブルの自動リサイズ機能を提供するカスタムフック
 */
export interface UseTableAutoResizeOptions {
  /** 自動リサイズを有効にするか */
  enabled?: boolean;
  /** 最小高さ（px） */
  minHeight?: number;
  /** 最大高さ（px または文字列） */
  maxHeight?: number | string;
  /** 下部マージン（px） */
  bottomMargin?: number;
  /** 外部からの強制リサイズトリガー */
  forceResize?: number;
}

export interface UseTableAutoResizeResult {
  /** コンテナのref */
  containerRef: React.RefObject<HTMLDivElement>;
  /** 計算された高さ */
  height: string;
}

export function useTableAutoResize({
  enabled = true,
  minHeight = 200,
  maxHeight,
  bottomMargin = 20,
  forceResize,
}: UseTableAutoResizeOptions = {}): UseTableAutoResizeResult {
  const [height, setHeight] = useState<string>('auto');
  const containerRef = useRef<HTMLDivElement>(null);

  // 高さを計算する関数
  const calculateHeight = useCallback(() => {
    if (!enabled || !containerRef.current || typeof window === 'undefined') {
      setHeight('auto');
      return;
    }

    const rect = containerRef.current.getBoundingClientRect();
    const availableHeight = window.innerHeight - rect.top - bottomMargin;

    // 最小高さの制約を適用
    let finalHeight = Math.max(availableHeight, minHeight);

    // 最大高さの制約を適用
    if (typeof maxHeight === 'number') {
      finalHeight = Math.min(finalHeight, maxHeight);
    }

    setHeight(`${finalHeight}px`);
  }, [enabled, minHeight, maxHeight, bottomMargin]);

  // ウィンドウリサイズイベントのハンドラー
  const handleResize = useCallback(() => {
    calculateHeight();
  }, [calculateHeight]);

  // ResizeObserverでコンテナサイズの変更を監視
  useEffect(() => {
    if (!enabled || typeof window === 'undefined') return;

    const currentContainer = containerRef.current;
    if (!currentContainer || !('ResizeObserver' in window)) return;

    const resizeObserver = new ResizeObserver(() => {
      calculateHeight();
    });

    resizeObserver.observe(currentContainer);

    return () => {
      resizeObserver.unobserve(currentContainer);
      resizeObserver.disconnect();
    };
  }, [enabled, calculateHeight]);

  // ウィンドウリサイズイベントを監視
  useEffect(() => {
    if (!enabled || typeof window === 'undefined') return;

    window.addEventListener('resize', handleResize);
    calculateHeight(); // 初回計算

    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, [enabled, handleResize, calculateHeight]);

  // 強制リサイズトリガーの監視
  useEffect(() => {
    if (forceResize !== undefined) {
      calculateHeight();
    }
  }, [forceResize, calculateHeight]);

  return {
    containerRef,
    height,
  };
}
