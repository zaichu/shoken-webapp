import { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { assetBalanceApi } from '@/features/assetBalance/api/assetBalanceApi';
import { useCSVReader, CSVReaderHook } from '@/hooks/useCSVReader';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';

export interface UseAssetBalanceDataSourceResult {
  // データ
  dbData: AssetBalanceData[];
  csvData: Record<string, unknown>[];
  // 状態
  loading: boolean;
  error: string | null;
  saving: boolean;
  deleting: boolean;
  // CSV関連
  csvReader: CSVReaderHook;
  // フラグ
  hasCsvData: boolean;
  hasDbData: boolean;
  // アクション
  handleFileSelect: (file: File) => Promise<void>;
  handleSaveToDB: () => Promise<void>;
  handleDeleteAll: () => Promise<void>;
}

/**
 * 保有銘柄データソースを管理するフック
 * 認証→DB取得→CSV→保存/削除→logout時クリアを管理
 */
export function useAssetBalanceDataSource(
  parseCsvItem: (item: Record<string, unknown>) => AssetBalanceData,
  filterCsvItem?: (item: AssetBalanceData) => boolean
): UseAssetBalanceDataSourceResult {
  const { isAuthenticated, isLoading: authLoading, onLogout } = useAuth();

  const [csvData, setCsvData] = useState<Record<string, unknown>[]>([]);
  const [dbData, setDbData] = useState<AssetBalanceData[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const csvReader = useCSVReader({ skipHeaderRows: 6 });

  const hasFetched = useRef(false);
  const isFetchingRef = useRef(false);

  const refetch = useCallback(async (force = false) => {
    if (authLoading) return;
    if (!isAuthenticated) return;
    if (hasFetched.current && !force) return;
    if (isFetchingRef.current) return;

    isFetchingRef.current = true;
    setLoading(true);
    setError(null);

    try {
      const data = await assetBalanceApi.list();
      setDbData(data);
      hasFetched.current = true;
    } catch (err) {
      setError(getDisplayErrorMessage(err, 'データ取得に失敗しました'));
    } finally {
      setLoading(false);
      isFetchingRef.current = false;
    }
  }, [authLoading, isAuthenticated]);

  useEffect(() => {
    if (!authLoading) {
      refetch();
    }
  }, [authLoading, refetch]);

  useEffect(() => {
    return onLogout(() => {
      setDbData([]);
      setCsvData([]);
      setError(null);
      hasFetched.current = false;
      csvReader.reset();
    });
  }, [onLogout, csvReader]);

  const handleFileSelect = useCallback(async (file: File) => {
    try {
      const data = await csvReader.parseCSV(file);
      setCsvData(data);
      if (csvReader.error) csvReader.resetError();
    } catch (e) {
      setError(getDisplayErrorMessage(e, 'CSVファイルの読み込みに失敗しました'));
    }
  }, [csvReader]);

  const handleSaveToDB = useCallback(async () => {
    if (!isAuthenticated) return;
    if (csvData.length === 0) return;

    setSaving(true);
    setError(null);

    try {
      let items = csvData.map(parseCsvItem);
      if (filterCsvItem) {
        items = items.filter(filterCsvItem);
      }
      await assetBalanceApi.bulkCreate(items);
      setCsvData([]);
      csvReader.reset();
      await refetch(true);
    } catch (err) {
      setError(getDisplayErrorMessage(err, '保存に失敗しました'));
    } finally {
      setSaving(false);
    }
  }, [csvData, csvReader, filterCsvItem, isAuthenticated, parseCsvItem, refetch]);

  const handleDeleteAll = useCallback(async () => {
    if (!isAuthenticated) return;

    setDeleting(true);
    setError(null);

    try {
      await assetBalanceApi.deleteAll();
      setDbData([]);
    } catch (err) {
      setError(getDisplayErrorMessage(err, '削除に失敗しました'));
    } finally {
      setDeleting(false);
    }
  }, [isAuthenticated]);

  const hasCsvData = csvData.length > 0;
  const hasDbData = useMemo(() => dbData.length > 0, [dbData]);

  return {
    dbData,
    csvData,
    loading,
    error,
    saving,
    deleting,
    csvReader,
    hasCsvData,
    hasDbData,
    handleFileSelect,
    handleSaveToDB,
    handleDeleteAll,
  };
}
