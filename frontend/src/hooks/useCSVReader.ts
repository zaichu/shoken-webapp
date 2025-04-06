import { useState } from 'react';
import { parseCSVFile } from '../services/csvUtils';

export function useCSVReader() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fileName, setFileName] = useState('');
  
  // CSVファイルを読み込んでパースする関数
  const parseCSV = async (file: File): Promise<any[]> => {
    setIsLoading(true);
    setError(null);
    setFileName(file.name);
    
    try {
      const data = await parseCSVFile(file, {
        onError: (errorMsg) => setError(errorMsg),
        onComplete: () => setIsLoading(false)
      });
      return data;
    } catch (e) {
      // エラーは parseCSVFile 内で処理済み
      throw e;
    }
  };
  
  return {
    parseCSV,
    isLoading,
    error,
    fileName
  };
}
