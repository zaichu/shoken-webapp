import { useState, useEffect, useRef } from 'react';

/**
 * 値をデバウンスするカスタムフック
 * @param value デバウンスする値
 * @param delay デバウンス時間（ミリ秒）
 * @param options オプション設定
 * @returns デバウンスされた値
 */
export function useDebounce<T>(
  value: T, 
  delay: number,
  options: {
    leading?: boolean;  // 最初の呼び出しを即座に実行するか
    trailing?: boolean; // 最後の呼び出しを実行するか
    maxWait?: number;   // 最大待機時間
  } = {}
): T {
  const { leading = false, trailing = true, maxWait } = options;
  
  const [debouncedValue, setDebouncedValue] = useState<T>(value);
  const timeoutRef = useRef<NodeJS.Timeout>();
  const maxTimeoutRef = useRef<NodeJS.Timeout>();
  const lastInvokeTimeRef = useRef<number>(0);
  const isFirstCallRef = useRef<boolean>(true);

  useEffect(() => {
    // 初回の場合、leadingオプションに関係なく初期値を設定
    if (isFirstCallRef.current) {
      isFirstCallRef.current = false;
      if (leading) {
        setDebouncedValue(value);
        lastInvokeTimeRef.current = Date.now();
        return;
      }
    }

    // 既存のタイマーをクリア
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    if (maxTimeoutRef.current) {
      clearTimeout(maxTimeoutRef.current);
    }

    // leadingが有効で、初回でない場合は即座に更新
    if (leading && !isFirstCallRef.current) {
      setDebouncedValue(value);
      lastInvokeTimeRef.current = Date.now();
    }

    // trailing: 指定時間後に値を更新
    if (trailing) {
      timeoutRef.current = setTimeout(() => {
        setDebouncedValue(value);
        lastInvokeTimeRef.current = Date.now();
      }, delay);
    }

    // maxWait: 最大待機時間を設定
    if (maxWait !== undefined) {
      const timeSinceLastInvoke = Date.now() - lastInvokeTimeRef.current;
      const remainingMaxWait = maxWait - timeSinceLastInvoke;
      
      if (remainingMaxWait > 0) {
        maxTimeoutRef.current = setTimeout(() => {
          setDebouncedValue(value);
          lastInvokeTimeRef.current = Date.now();
          // 通常のデバウンスタイマーもクリア
          if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
          }
        }, remainingMaxWait);
      } else {
        // maxWaitを超えている場合は即座に実行
        setDebouncedValue(value);
        lastInvokeTimeRef.current = Date.now();
      }
    }

    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
      if (maxTimeoutRef.current) {
        clearTimeout(maxTimeoutRef.current);
      }
    };
  }, [value, delay, leading, trailing, maxWait]);

  // コンポーネントアンマウント時のクリーンアップ
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
      if (maxTimeoutRef.current) {
        clearTimeout(maxTimeoutRef.current);
      }
    };
  }, []);

  return debouncedValue;
}

/**
 * シンプルなデバウンスフック（後方互換性のため）
 */
export function useSimpleDebounce<T>(value: T, delay: number): T {
  return useDebounce(value, delay, { leading: false, trailing: true });
}
