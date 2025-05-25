import { useState, useEffect, useCallback } from 'react';

export interface UseLocalStorageOptions<T = unknown> {
  serializer?: {
    serialize: (value: T) => string;
    deserialize: (text: string) => T;
  };
  syncAcrossTabs?: boolean;
}

/**
 * ローカルストレージとの同期を行うカスタムフック
 * @param key ローカルストレージのキー
 * @param initialValue 初期値
 * @param options オプション設定
 * @returns [値, 設定関数, 削除関数]
 */
export function useLocalStorage<T>(
  key: string,
  initialValue: T,
  options: UseLocalStorageOptions<T> = {}
) {
  const {
    serializer = {
      serialize: JSON.stringify,
      deserialize: JSON.parse,
    },
    syncAcrossTabs = true
  } = options;

  // SSR対応: サーバーサイドでは初期値を返す
  const isClient = typeof window !== 'undefined';

  // 初期値を取得する関数（依存配列を最小化）
  // const getInitialValue = useCallback((): T => {
  //   if (!isClient) {
  //     return initialValue;
  //   }

  //   try {
  //     const item = window.localStorage.getItem(key);
  //     if (item === null) {
  //       return initialValue;
  //     }
  //     return serializer.deserialize(item);
  //   } catch (error) {
  //     console.warn(`Error reading localStorage key "${key}":`, error);
  //     return initialValue;
  //   }
  // }, [key, initialValue, isClient]); // serializerを除外

  const [storedValue, setStoredValue] = useState<T>(() => {
    // 初期化時に一度だけ実行
    if (!isClient) {
      return initialValue;
    }

    try {
      const item = window.localStorage.getItem(key);
      if (item === null) {
        return initialValue;
      }
      return serializer.deserialize(item);
    } catch (error) {
      console.warn(`Error reading localStorage key "${key}":`, error);
      return initialValue;
    }
  });

  // 値を設定する関数
  const setValue = useCallback((value: T | ((prevValue: T) => T)) => {
    try {
      setStoredValue(prevValue => {
        const valueToStore = value instanceof Function ? value(prevValue) : value;

        // localStorage操作を try-catch で囲む
        if (isClient) {
          try {
            window.localStorage.setItem(key, serializer.serialize(valueToStore));

            // カスタムイベントを発火してタブ間同期をトリガー
            if (syncAcrossTabs) {
              window.dispatchEvent(new CustomEvent('local-storage', {
                detail: { key, newValue: valueToStore }
              }));
            }
          } catch (error) {
            console.warn(`Error setting localStorage key "${key}":`, error);
          }
        }

        return valueToStore;
      });
    } catch (error) {
      console.warn(`Error setting localStorage key "${key}":`, error);
    }
  }, [key, serializer, isClient, syncAcrossTabs]);

  // 値を削除する関数
  const removeValue = useCallback(() => {
    try {
      setStoredValue(initialValue);

      // localStorage操作を try-catch で囲む
      if (isClient) {
        try {
          window.localStorage.removeItem(key);

          // カスタムイベントを発火してタブ間同期をトリガー
          if (syncAcrossTabs) {
            window.dispatchEvent(new CustomEvent('local-storage', {
              detail: { key, newValue: null }
            }));
          }
        } catch (error) {
          console.warn(`Error removing localStorage key "${key}":`, error);
        }
      }
    } catch (error) {
      console.warn(`Error removing localStorage key "${key}":`, error);
    }
  }, [key, initialValue, isClient, syncAcrossTabs]);

  // タブ間での同期を監視
  useEffect(() => {
    if (!isClient || !syncAcrossTabs) {
      return;
    }

    const handleStorageChange = (e: StorageEvent | CustomEvent) => {
      if ('key' in e && e.key === key) {
        // 標準のstorageイベント
        try {
          if (e.newValue === null) {
            setStoredValue(initialValue);
          } else {
            setStoredValue(serializer.deserialize(e.newValue));
          }
        } catch (error) {
          console.warn(`Error parsing storage event for key "${key}":`, error);
        }
      } else if ('detail' in e && e.detail.key === key) {
        // カスタムイベント
        if (e.detail.newValue === null) {
          setStoredValue(initialValue);
        } else {
          setStoredValue(e.detail.newValue);
        }
      }
    };

    // 標準のstorageイベントとカスタムイベントの両方を監視
    window.addEventListener('storage', handleStorageChange);
    window.addEventListener('local-storage', handleStorageChange as EventListener);

    return () => {
      window.removeEventListener('storage', handleStorageChange);
      window.removeEventListener('local-storage', handleStorageChange as EventListener);
    };
  }, [key, initialValue, serializer, isClient, syncAcrossTabs]);

  return [storedValue, setValue, removeValue] as const;
}

/**
 * ローカルストレージのキーの存在チェック
 */
export function useLocalStorageKey(key: string): boolean {
  const [exists, setExists] = useState(false);

  useEffect(() => {
    if (typeof window !== 'undefined') {
      setExists(window.localStorage.getItem(key) !== null);
    }
  }, [key]);

  return exists;
}

/**
 * 複数のローカルストレージキーを管理
 */
export function useMultipleLocalStorage<T extends Record<string, unknown>>(
  keys: Record<keyof T, unknown>
): [T, (key: keyof T, value: T[keyof T]) => void, (key: keyof T) => void] {
  const [values, setValues] = useState<T>(() => {
    if (typeof window === 'undefined') {
      return keys as T;
    }

    const result = {} as T;
    Object.entries(keys).forEach(([key, defaultValue]) => {
      try {
        const item = window.localStorage.getItem(key);
        result[key as keyof T] = item ? JSON.parse(item) : defaultValue;
      } catch {
        result[key as keyof T] = defaultValue;
      }
    });
    return result;
  });

  const setValue = useCallback((key: keyof T, value: T[keyof T]) => {
    setValues(prev => ({ ...prev, [key]: value }));

    if (typeof window !== 'undefined') {
      try {
        window.localStorage.setItem(key as string, JSON.stringify(value));
      } catch (error) {
        console.warn(`Error setting localStorage key "${String(key)}":`, error);
      }
    }
  }, []);

  const removeValue = useCallback((key: keyof T) => {
    setValues(prev => ({ ...prev, [key]: keys[key] }));

    if (typeof window !== 'undefined') {
      try {
        window.localStorage.removeItem(key as string);
      } catch (error) {
        console.warn(`Error removing localStorage key "${String(key)}":`, error);
      }
    }
  }, [keys]);

  return [values, setValue, removeValue];
}
