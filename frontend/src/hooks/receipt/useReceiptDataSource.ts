import { useState, useEffect, useCallback } from 'react';
import { useAuth } from '@/features/auth/hooks/useAuth';

interface UseReceiptDataSourceOptions<T, R> {
  csvData: Record<string, unknown>[];
  parseCSVItem: (item: Record<string, unknown>) => T;
  sortFn: (data: T[]) => T[];
  fetchFromDB: () => Promise<R[]>;
  transformDBItem: (item: R) => T;
  bulkCreate: (items: T[]) => Promise<{ inserted: number; skipped: number }>;
}

interface UseReceiptDataSourceResult<T> {
  data: T[];
  isLoading: boolean;
  error: string | null;
  isFromDB: boolean;
  saveToDB: () => Promise<void>;
  refetch: () => Promise<void>;
}

/**
 * ログイン状態に応じてDB/CSVからデータを取得するフック
 * - ログイン時: DBからデータを取得
 * - 未ログイン時: CSVからデータを取得
 */
export function useReceiptDataSource<T, R>(
  options: UseReceiptDataSourceOptions<T, R>
): UseReceiptDataSourceResult<T> {
  const { csvData, parseCSVItem, sortFn, fetchFromDB, transformDBItem, bulkCreate } = options;
  const { isAuthenticated, isLoading: authLoading } = useAuth();

  const [data, setData] = useState<T[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isFromDB, setIsFromDB] = useState(false);

  // DBからデータを取得
  const fetchData = useCallback(async () => {
    if (authLoading) return;

    setIsLoading(true);
    setError(null);

    try {
      if (isAuthenticated) {
        const dbData = await fetchFromDB();
        if (dbData.length > 0) {
          const transformed = dbData.map(transformDBItem);
          setData(sortFn(transformed));
          setIsFromDB(true);
        } else {
          // DBにデータがない場合はCSVから
          const parsedData = csvData.map(parseCSVItem);
          setData(sortFn(parsedData));
          setIsFromDB(false);
        }
      } else {
        // 未ログイン時はCSVから
        const parsedData = csvData.map(parseCSVItem);
        setData(sortFn(parsedData));
        setIsFromDB(false);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : '取得に失敗しました');
      // エラー時はCSVから
      const parsedData = csvData.map(parseCSVItem);
      setData(sortFn(parsedData));
      setIsFromDB(false);
    } finally {
      setIsLoading(false);
    }
  }, [authLoading, isAuthenticated, csvData, parseCSVItem, sortFn, fetchFromDB, transformDBItem]);

  // CSVデータをDBに保存
  const saveToDB = useCallback(async () => {
    if (!isAuthenticated) {
      throw new Error('ログインが必要です');
    }

    const parsedData = csvData.map(parseCSVItem);
    await bulkCreate(parsedData);
    await fetchData();
  }, [isAuthenticated, csvData, parseCSVItem, bulkCreate, fetchData]);

  // 初回ロードとcsvData変更時にデータを取得
  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return {
    data,
    isLoading: isLoading || authLoading,
    error,
    isFromDB,
    saveToDB,
    refetch: fetchData,
  };
}
