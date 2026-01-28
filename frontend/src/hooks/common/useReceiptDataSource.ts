import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { useCSVReader, CSVReaderHook } from '../useCSVReader';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';
import { CSVParseOptions } from '@/lib/types/csv';

/**
 * API インターフェース
 */
export interface ReceiptDataSourceApi<T> {
  list: () => Promise<T[]>;
  bulkCreate: (items: T[]) => Promise<unknown>;
  deleteAll: () => Promise<unknown>;
}

/**
 * useReceiptDataSource オプション
 */
export interface UseReceiptDataSourceOptions<T, D = T> {
  /** API インターフェース */
  api: ReceiptDataSourceApi<T>;
  /** DBデータを表示用データ型に変換する関数 */
  transformDB?: (item: T) => D;
  /** CSVデータをAPI送信用データ型に変換する関数 */
  parseCsvItem?: (item: Record<string, unknown>) => T;
  /** CSVリーダーオプション */
  csvReaderOptions?: CSVParseOptions;
  /** 削除確認メッセージ（undefinedの場合は確認なし） */
  deleteConfirmMessage?: string;
}

/**
 * useReceiptDataSource 戻り値
 */
export interface UseReceiptDataSourceResult<T, D = T> {
  // データ
  dbData: D[];
  csvData: Record<string, unknown>[];
  /** 表示用データ（認証時はDB優先、未認証時はCSV） */
  displayData: D[] | Record<string, unknown>[];
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
  refetch: (force?: boolean) => Promise<void>;
  setCsvData: (data: Record<string, unknown>[]) => void;
  clearAll: () => void;
}

/**
 * 受取金データソースを管理するフック
 * 認証→DB→CSV→保存/削除→logout時クリアを共通化
 */
export function useReceiptDataSource<T, D = T>(
  options: UseReceiptDataSourceOptions<T, D>
): UseReceiptDataSourceResult<T, D> {
  const { api, transformDB, parseCsvItem, csvReaderOptions, deleteConfirmMessage } = options;
  const { isAuthenticated, isLoading: authLoading, onLogout } = useAuth();

  // CSVデータ
  const [csvData, setCsvData] = useState<Record<string, unknown>[]>([]);
  // DBデータ
  const [dbData, setDbData] = useState<D[]>([]);
  // 状態
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);

  // CSVリーダー
  const csvReader = useCSVReader(csvReaderOptions);

  // フェッチ済みフラグ（多重実行防止）
  const hasFetched = useRef(false);
  const isFetchingRef = useRef(false);

  /**
   * DBからデータを取得
   */
  const refetch = useCallback(async (force = false) => {
    // 認証状態が確定していない場合は待機
    if (authLoading) return;
    // 未認証の場合はスキップ
    if (!isAuthenticated) return;
    // 既にフェッチ済みで強制更新でない場合はスキップ
    if (hasFetched.current && !force) return;
    // 実行中の場合はスキップ
    if (isFetchingRef.current) return;

    isFetchingRef.current = true;
    setLoading(true);
    setError(null);

    try {
      const data = await api.list();
      const transformed = transformDB
        ? data.map(item => transformDB(item))
        : (data as unknown as D[]);
      setDbData(transformed);
      hasFetched.current = true;
    } catch (err) {
      setError(getDisplayErrorMessage(err, 'データ取得に失敗しました'));
    } finally {
      setLoading(false);
      isFetchingRef.current = false;
    }
  }, [api, authLoading, isAuthenticated, transformDB]);

  /**
   * 認証状態が確定したらDBからデータを取得
   */
  useEffect(() => {
    if (!authLoading) {
      refetch();
    }
  }, [authLoading, refetch]);

  /**
   * ログアウト時に全データをクリア
   */
  useEffect(() => {
    return onLogout(() => {
      setDbData([]);
      setCsvData([]);
      setError(null);
      hasFetched.current = false;
      csvReader.reset();
    });
  }, [onLogout, csvReader]);

  /**
   * CSVファイル選択時の処理
   */
  const handleFileSelect = useCallback(async (file: File) => {
    try {
      const data = await csvReader.parseCSV(file);
      setCsvData(data);
      if (csvReader.error) csvReader.resetError();
    } catch (e) {
      setError(getDisplayErrorMessage(e, 'CSVファイルの読み込みに失敗しました'));
    }
  }, [csvReader]);

  /**
   * CSVデータをDBに保存
   */
  const handleSaveToDB = useCallback(async () => {
    if (!isAuthenticated) return;
    if (csvData.length === 0) return;
    if (!parseCsvItem) return;

    setSaving(true);
    setError(null);

    try {
      const items = csvData.map(parseCsvItem);
      await api.bulkCreate(items);
      setCsvData([]);
      csvReader.reset();
      await refetch(true);
    } catch (err) {
      setError(getDisplayErrorMessage(err, '保存に失敗しました'));
    } finally {
      setSaving(false);
    }
  }, [api, csvData, csvReader, isAuthenticated, parseCsvItem, refetch]);

  /**
   * DBデータを全削除
   */
  const handleDeleteAll = useCallback(async () => {
    if (!isAuthenticated) return;
    if (deleteConfirmMessage && !window.confirm(deleteConfirmMessage)) return;

    setDeleting(true);
    setError(null);

    try {
      await api.deleteAll();
      setDbData([]);
    } catch (err) {
      setError(getDisplayErrorMessage(err, '削除に失敗しました'));
    } finally {
      setDeleting(false);
    }
  }, [api, isAuthenticated, deleteConfirmMessage]);

  /**
   * 全データをクリア
   */
  const clearAll = useCallback(() => {
    setDbData([]);
    setCsvData([]);
    setError(null);
    hasFetched.current = false;
    csvReader.reset();
  }, [csvReader]);

  // フラグ
  const hasCsvData = csvData.length > 0;
  const hasDbData = dbData.length > 0;

  // 表示用データ（認証時はDB優先、未認証時はCSV）
  const displayData = useMemo(
    () => (isAuthenticated && hasDbData ? dbData : csvData),
    [isAuthenticated, hasDbData, dbData, csvData]
  );

  return {
    dbData,
    csvData,
    displayData,
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
    refetch,
    setCsvData,
    clearAll,
  };
}
