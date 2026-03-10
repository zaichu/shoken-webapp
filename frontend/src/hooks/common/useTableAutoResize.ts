import { useEffect, useRef, useState } from 'react';

/**
 * テーブルの自動リサイズ機能を提供するカスタムフック
 */
interface UseTableAutoResizeOptions {
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

interface UseTableAutoResizeResult {
  /** コンテナのref */
  containerRef: React.RefObject<HTMLDivElement | null>;
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

  // すべての高さ計算とイベント監視を1つのuseEffectで管理
  useEffect(() => {
    if (!enabled || typeof window === 'undefined') return;

    // 高さを計算する関数
    const calculateHeight = () => {
      if (!containerRef.current) {
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
    };

    const currentContainer = containerRef.current;

    // 初回計算
    calculateHeight();

    // ResizeObserverの設定
    let resizeObserver: ResizeObserver | undefined;
    if (currentContainer && 'ResizeObserver' in window) {
      resizeObserver = new ResizeObserver(calculateHeight);
      resizeObserver.observe(currentContainer);
    }

    // ウィンドウリサイズイベントの設定
    window.addEventListener('resize', calculateHeight);

    return () => {
      if (resizeObserver && currentContainer) {
        resizeObserver.unobserve(currentContainer);
        resizeObserver.disconnect();
      }
      window.removeEventListener('resize', calculateHeight);
    };
  }, [enabled, minHeight, maxHeight, bottomMargin, forceResize]);

  return {
    containerRef,
    height,
  };
}
