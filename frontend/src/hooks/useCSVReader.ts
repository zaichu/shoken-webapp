import { useState, useCallback } from 'react';
import { parseCSVFile } from '../lib/csv/parser';

/**
 * CSVファイルを読み込むためのカスタムフック
 */
export function useCSVReader() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fileName, setFileName] = useState('');

  const parseCSV = useCallback(async (file: File): Promise<any[]> => {
    setIsLoading(true);
    setError(null);
    setFileName(file.name);

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

  const resetError = useCallback(() => {
    setError(null);
  }, []);

  return {
    parseCSV,
    isLoading,
    error,
    resetError,
    fileName
  };
}
