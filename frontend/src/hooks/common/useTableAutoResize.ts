import { useEffect, useRef, useState } from 'react';

interface UseTableAutoResizeOptions {
  /** 自動リサイズを有効にするか */
  enabled?: boolean;
  /** 最小高さ（px） */
  minHeight?: number;
  /** 最大高さ（px または文字列） */
  maxHeight?: number | string;
  /** 下部マージン（px） */
  bottomMargin?: number;
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
}: UseTableAutoResizeOptions = {}): UseTableAutoResizeResult {
  const [height, setHeight] = useState<string>('auto');
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!enabled || typeof window === 'undefined') return;

    const calculateHeight = () => {
      if (!containerRef.current) {
        setHeight('auto');
        return;
      }

      const rect = containerRef.current.getBoundingClientRect();
      const availableHeight = window.innerHeight - rect.top - bottomMargin;
      let finalHeight = Math.max(availableHeight, minHeight);

      if (typeof maxHeight === 'number') {
        finalHeight = Math.min(finalHeight, maxHeight);
      }

      setHeight(`${finalHeight}px`);
    };

    const currentContainer = containerRef.current;

    calculateHeight();

    // コンテナ自体に加え、周辺レイアウト変化で top が変わるケースも監視する
    let resizeObserver: ResizeObserver | undefined;
    if (currentContainer && 'ResizeObserver' in window) {
      resizeObserver = new ResizeObserver(() => calculateHeight());
      const observedElements = [
        currentContainer,
        currentContainer.parentElement,
        document.body,
      ].filter((element): element is HTMLElement => element !== null);

      for (const element of observedElements) {
        resizeObserver.observe(element);
      }
    }

    window.addEventListener('resize', calculateHeight);

    return () => {
      resizeObserver?.disconnect();
      window.removeEventListener('resize', calculateHeight);
    };
  }, [enabled, minHeight, maxHeight, bottomMargin]);

  return {
    containerRef,
    height,
  };
}
