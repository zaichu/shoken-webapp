import { useState, useEffect, useCallback } from 'react';

/**
 * 基本的なローカルストレージ機能を提供するカスタムフック
 * シンプルで軽量な実装
 */
export function useLocalStorage<T>(
  key: string,
  initialValue: T
): [T, (value: T | ((prevValue: T) => T)) => void, () => void] {
  // SSR対応: サーバーサイドでは初期値を返す
  const isClient = typeof window !== 'undefined';

  // 初期値を取得
  const [storedValue, setStoredValue] = useState<T>(() => {
    if (!isClient) {
      return initialValue;
    }

    try {
      const item = window.localStorage.getItem(key);
      if (item === null) {
        return initialValue;
      }
      return JSON.parse(item);
    } catch (error) {
      console.warn(`ローカルストレージからの読み込みに失敗しました (キー: ${key}):`, error);
      return initialValue;
    }
  });

  // 値を設定する関数
  const setValue = useCallback((value: T | ((prevValue: T) => T)) => {
    try {
      setStoredValue(prevValue => {
        const valueToStore = value instanceof Function ? value(prevValue) : value;

        if (isClient) {
          try {
            window.localStorage.setItem(key, JSON.stringify(valueToStore));
          } catch (error) {
            console.warn(`ローカルストレージへの保存に失敗しました (キー: ${key}):`, error);
          }
        }

        return valueToStore;
      });
    } catch (error) {
      console.warn(`値の設定に失敗しました (キー: ${key}):`, error);
    }
  }, [key, isClient]);

  // 値を削除する関数
  const removeValue = useCallback(() => {
    try {
      setStoredValue(initialValue);

      if (isClient) {
        try {
          window.localStorage.removeItem(key);
        } catch (error) {
          console.warn(`ローカルストレージからの削除に失敗しました (キー: ${key}):`, error);
        }
      }
    } catch (error) {
      console.warn(`値の削除に失敗しました (キー: ${key}):`, error);
    }
  }, [key, initialValue, isClient]);

  return [storedValue, setValue, removeValue];
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
