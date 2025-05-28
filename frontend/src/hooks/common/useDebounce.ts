import { useState, useEffect, useRef, useCallback } from 'react';

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
  const lastCallTimeRef = useRef<number>(0);
  const leadingCallRef = useRef<boolean>(true);

  const cancel = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = undefined;
    }
    if (maxTimeoutRef.current) {
      clearTimeout(maxTimeoutRef.current);
      maxTimeoutRef.current = undefined;
    }
  }, []);

  useEffect(() => {
    const now = Date.now();
    const timeSinceLastCall = now - lastCallTimeRef.current;

    // 既存のタイマーをクリア
    cancel();

    // leadingオプションの処理
    if (leading && leadingCallRef.current) {
      setDebouncedValue(value);
      leadingCallRef.current = false;
    }

    // trailing オプションの処理
    if (trailing) {
      timeoutRef.current = setTimeout(() => {
        setDebouncedValue(value);
        leadingCallRef.current = true;
        lastCallTimeRef.current = Date.now();
      }, delay);
    } else if (!leading) {
      // trailing=false で leading=false の場合、値を更新しない
      leadingCallRef.current = true;
    }

    // maxWait オプションの処理
    if (maxWait !== undefined && trailing) {
      const remainingMaxWait = Math.max(0, maxWait - timeSinceLastCall);
      
      if (remainingMaxWait === 0) {
        // maxWaitを超えている場合は即座に実行
        setDebouncedValue(value);
        lastCallTimeRef.current = now;
        leadingCallRef.current = true;
      } else if (remainingMaxWait < delay) {
        // maxWaitまでの残り時間がdelayより短い場合
        maxTimeoutRef.current = setTimeout(() => {
          setDebouncedValue(value);
          lastCallTimeRef.current = Date.now();
          leadingCallRef.current = true;
          // 通常のデバウンスタイマーもクリア
          if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
            timeoutRef.current = undefined;
          }
        }, remainingMaxWait);
      }
    }

    // 値が変更されたら、leading callをリセット
    if (timeSinceLastCall > delay) {
      leadingCallRef.current = true;
    }

    lastCallTimeRef.current = now;

    return cancel;
  }, [value, delay, leading, trailing, maxWait, cancel]);

  // コンポーネントアンマウント時のクリーンアップ
  useEffect(() => {
    return cancel;
  }, [cancel]);

  return debouncedValue;
}

/**
 * シンプルなデバウンスフック（後方互換性のため）
 */
export function useSimpleDebounce<T>(value: T, delay: number): T {
  return useDebounce(value, delay, { leading: false, trailing: true });
}
