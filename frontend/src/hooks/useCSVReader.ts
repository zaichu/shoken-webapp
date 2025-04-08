import { useState, useCallback } from 'react';
import { parseCSVFile } from '../services/csvUtils';

export function useCSVReader() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /*
   * CSVファイルを読み込んでパースする関数
   * @param file CSVファイル
   * @returns パースされたデータ
   */
  const parseCSV = useCallback(async (file: File): Promise<any[]> => {
    setIsLoading(true);
    setError(null);

    try {
      const data = await parseCSVFile(file, {
        onStart: () => setIsLoading(true),
        onError: (errorMsg) => setError(errorMsg),
        onComplete: () => setIsLoading(false)
      });
      return data;
    } catch (e) {
      setIsLoading(false);
      throw e;
    }
  }, []);

  /**
   * エラーをリセットする関数
   */
  const resetError = useCallback(() => {
    setError(null);
  }, []);

  return {
    parseCSV,
    isLoading,
    error,
    resetError
  };
}
