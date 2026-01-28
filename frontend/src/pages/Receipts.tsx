import { useState, useEffect, useCallback, useRef } from 'react';
import { Layout } from '../components/templates/Layout';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { useCSVReader } from '../hooks/useCSVReader';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { Dividend } from './Receipt/Dividend';
import { DomesticStock } from './Receipt/DomesticStock';
import { Mutualfund } from './Receipt/Mutualfund';
import { dividendApi, domesticStockApi, mutualfundApi } from '@/features/receipt/api/receiptApi';
import {
  parseDividendCsvItem,
  parseDomesticStockCsvItem,
  parseMutualfundCsvItem,
  transformDBDividend,
  transformDBDomesticStock,
  transformDBMutualfund,
} from '@/features/receipt/parsers';
import { DividendData } from '@/lib/interfaces/dividend';
import { DomesticStockData } from '@/lib/interfaces/domesticStock';
import { MutualfundData } from '@/lib/interfaces/mutualfund';

type ReceiptsType = 'dividend' | 'domesticstock' | 'mutualfund';

/**
 * 明細種類ごとにCSVデータを管理するページコンポーネント
 * ログイン時はDBからデータを取得、未ログイン時はCSVから取得
 */
export function ReceiptsPage() {
  const { isAuthenticated, isLoading: authLoading, onLogout } = useAuth();
  const [receiptsType, setReceiptsType] = useState<ReceiptsType>('dividend');

  // CSVから読み込んだデータ
  const [dividendCsvData, setDividendCsvData] = useState<Record<string, unknown>[]>([]);
  const [domesticStockCsvData, setDomesticStockCsvData] = useState<Record<string, unknown>[]>([]);
  const [mutualfundCsvData, setMutualfundCsvData] = useState<Record<string, unknown>[]>([]);

  // DBから読み込んだデータ
  const [dividendDBData, setDividendDBData] = useState<DividendData[]>([]);
  const [domesticStockDBData, setDomesticStockDBData] = useState<DomesticStockData[]>([]);
  const [mutualfundDBData, setMutualfundDBData] = useState<MutualfundData[]>([]);

  // DB読み込み状態
  const [dbLoading, setDbLoading] = useState(false);
  const [dbError, setDbError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);

  // 各明細種類ごとにCSVリーダーフックを作成
  const dividendCSV = useCSVReader();
  const domesticStockCSV = useCSVReader();
  const mutualfundCSV = useCSVReader();

  // フェッチ済みフラグ（多重実行防止）
  const hasFetched = useRef(false);
  const isFetchingRef = useRef(false);

  // DBからデータを取得
  const fetchFromDB = useCallback(async (force = false) => {
    // 認証状態が確定していない場合は待機
    if (authLoading) return;
    // 未認証の場合はスキップ
    if (!isAuthenticated) return;
    // 既にフェッチ済みで強制更新でない場合はスキップ
    if (hasFetched.current && !force) return;

    if (isFetchingRef.current) return;
    isFetchingRef.current = true;
    setDbLoading(true);
    setDbError(null);

    try {
      const [dividends, stocks, funds] = await Promise.all([
        dividendApi.list(),
        domesticStockApi.list(),
        mutualfundApi.list(),
      ]);

      setDividendDBData(dividends.map(d => transformDBDividend(d as unknown as Record<string, unknown>)));
      setDomesticStockDBData(stocks.map(d => transformDBDomesticStock(d as unknown as Record<string, unknown>)));
      setMutualfundDBData(funds.map(d => transformDBMutualfund(d as unknown as Record<string, unknown>)));
      hasFetched.current = true;
    } catch (err) {
      setDbError(err instanceof Error ? err.message : 'データ取得に失敗しました');
    } finally {
      setDbLoading(false);
      isFetchingRef.current = false;
    }
  }, [isAuthenticated, authLoading]);

  // 認証状態が確定したらDBからデータを取得
  useEffect(() => {
    if (!authLoading) {
      fetchFromDB();
    }
  }, [authLoading, fetchFromDB]);

  // ログアウト時に全データをクリア
  useEffect(() => {
    return onLogout(() => {
      // DBデータをクリア
      setDividendDBData([]);
      setDomesticStockDBData([]);
      setMutualfundDBData([]);
      // CSVデータをクリア
      setDividendCsvData([]);
      setDomesticStockCsvData([]);
      setMutualfundCsvData([]);
      // エラー状態をクリア
      setDbError(null);
      // フェッチフラグをリセット
      hasFetched.current = false;
    });
  }, [onLogout]);

  /**
   * 現在選択中のタブに応じてCSV処理を切り替える
   */
  const handleFileSelect = async (file: File) => {
    try {
      switch (receiptsType) {
        case 'dividend':
          setDividendCsvData(await dividendCSV.parseCSV(file));
          if (dividendCSV.error) dividendCSV.resetError();
          break;
        case 'domesticstock':
          setDomesticStockCsvData(await domesticStockCSV.parseCSV(file));
          if (domesticStockCSV.error) domesticStockCSV.resetError();
          break;
        case 'mutualfund':
          setMutualfundCsvData(await mutualfundCSV.parseCSV(file));
          if (mutualfundCSV.error) mutualfundCSV.resetError();
          break;
      }
    } catch (e) {
      console.error('CSV処理エラー:', e);
    }
  };

  /**
   * CSVデータをDBに保存
   */
  const handleSaveToDB = async () => {
    if (!isAuthenticated) return;

    setSaving(true);
    setDbError(null);

    try {
      switch (receiptsType) {
        case 'dividend': {
          const items = dividendCsvData.map(parseDividendCsvItem);
          await dividendApi.bulkCreate(items);
          setDividendCsvData([]);
          break;
        }
        case 'domesticstock': {
          const items = domesticStockCsvData.map(parseDomesticStockCsvItem);
          await domesticStockApi.bulkCreate(items);
          setDomesticStockCsvData([]);
          break;
        }
        case 'mutualfund': {
          const items = mutualfundCsvData.map(parseMutualfundCsvItem);
          await mutualfundApi.bulkCreate(items);
          setMutualfundCsvData([]);
          break;
        }
      }
      await fetchFromDB(true);
    } catch (err) {
      setDbError(err instanceof Error ? err.message : '保存に失敗しました');
    } finally {
      setSaving(false);
    }
  };

  /**
   * DBデータを全削除
   */
  const handleDeleteAll = async () => {
    if (!isAuthenticated) return;
    if (!window.confirm('現在のタブのデータをすべて削除しますか？')) return;

    setDeleting(true);
    setDbError(null);
    console.log(`[handleDeleteAll] 開始: ${receiptsType}`);

    try {
      switch (receiptsType) {
        case 'dividend':
          await dividendApi.deleteAll();
          setDividendDBData([]);
          break;
        case 'domesticstock':
          await domesticStockApi.deleteAll();
          setDomesticStockDBData([]);
          break;
        case 'mutualfund':
          await mutualfundApi.deleteAll();
          setMutualfundDBData([]);
          break;
      }
      console.log(`[handleDeleteAll] 成功: ${receiptsType}`);
    } catch (err) {
      console.error(`[handleDeleteAll] 失敗: ${receiptsType}`, err);
      setDbError(err instanceof Error ? err.message : '削除に失敗しました');
    } finally {
      setDeleting(false);
    }
  };

  /**
   * 現在選択中のタブに対応するエラーとローディング状態を取得
   */
  const getCurrentCSVState = () => {
    switch (receiptsType) {
      case 'dividend':
        return { isLoading: dividendCSV.isLoading, error: dividendCSV.error, fileName: dividendCSV.fileName, csvData: dividendCsvData };
      case 'domesticstock':
        return { isLoading: domesticStockCSV.isLoading, error: domesticStockCSV.error, fileName: domesticStockCSV.fileName, csvData: domesticStockCsvData };
      case 'mutualfund':
        return { isLoading: mutualfundCSV.isLoading, error: mutualfundCSV.error, fileName: mutualfundCSV.fileName, csvData: mutualfundCsvData };
    }
  };

  const { isLoading, error, fileName, csvData } = getCurrentCSVState();
  const hasCsvData = csvData.length > 0;

  // 現在のタブのDBデータがあるか判定
  const hasDbData = (() => {
    switch (receiptsType) {
      case 'dividend': return dividendDBData.length > 0;
      case 'domesticstock': return domesticStockDBData.length > 0;
      case 'mutualfund': return mutualfundDBData.length > 0;
    }
  })();

  // 表示用データを決定（ログイン時はDB優先、未ログイン時はCSV）
  const getDividendData = () => {
    if (isAuthenticated && dividendDBData.length > 0) return dividendDBData;
    return dividendCsvData;
  };
  const getDomesticStockData = () => {
    if (isAuthenticated && domesticStockDBData.length > 0) return domesticStockDBData;
    return domesticStockCsvData;
  };
  const getMutualfundData = () => {
    if (isAuthenticated && mutualfundDBData.length > 0) return mutualfundDBData;
    return mutualfundCsvData;
  };

  return (
    <Layout>
      <nav className="nav nav-tabs">
        <ul className="nav nav-tabs">
          <li className="nav-item">
            <button
              className={receiptsType === 'dividend' ? 'nav-link active' : 'nav-link'}
              onClick={() => setReceiptsType('dividend')}
            >
              配当金
            </button>
          </li>
          <li className="nav-item">
            <button
              className={receiptsType === 'domesticstock' ? 'nav-link active' : 'nav-link'}
              onClick={() => setReceiptsType('domesticstock')}
            >
              国内株式
            </button>
          </li>
          <li className="nav-item">
            <button
              className={receiptsType === 'mutualfund' ? 'nav-link active' : 'nav-link'}
              onClick={() => setReceiptsType('mutualfund')}
            >
              投資信託
            </button>
          </li>
        </ul>
      </nav>
      <div className="receipt-page mt-2" aria-busy={isLoading || dbLoading || authLoading || saving || deleting}>
        <div className="d-flex align-items-center gap-2 flex-wrap">
          <div className="page-control-panel">
            <CSVFileInput
              onFileSelect={handleFileSelect}
              selectedFileName={fileName}
              disabled={dbLoading || saving || deleting || authLoading}
            />
          </div>
          {isAuthenticated && (
            <div className="btn-group" role="group" aria-label="データ操作">
              {hasCsvData && (
                <button
                  className="btn btn-primary btn-sm"
                  onClick={handleSaveToDB}
                  disabled={saving || deleting}
                  aria-disabled={saving || deleting}
                >
                  {saving ? '保存中...' : '保存'}
                </button>
              )}
              {hasDbData && (
                <button
                  className="btn btn-outline-danger btn-sm"
                  onClick={handleDeleteAll}
                  disabled={saving || deleting}
                  aria-disabled={saving || deleting}
                >
                  {deleting ? '削除中...' : '削除'}
                </button>
              )}
            </div>
          )}
        </div>

        {(error || dbError) && (
          <div className="alert alert-danger my-3" role="alert" aria-live="assertive">
            <strong>エラー:</strong> {error || dbError}
          </div>
        )}

        <div aria-live="polite" aria-atomic="true">
          {(isLoading || dbLoading || authLoading) && (
            <div className="text-center my-4" role="status">
              <div className="spinner-border text-primary" aria-hidden="true" />
              <p className="mt-2 text-muted">
                {authLoading && '認証状態を確認しています...'}
                {dbLoading && 'データを読み込んでいます...'}
                {isLoading && 'CSVファイルを処理しています...'}
              </p>
            </div>
          )}
        </div>

        {receiptsType === 'dividend' && <Dividend csvData={getDividendData() as Record<string, unknown>[]} />}
        {receiptsType === 'domesticstock' && <DomesticStock csvData={getDomesticStockData() as Record<string, unknown>[]} />}
        {receiptsType === 'mutualfund' && <Mutualfund csvData={getMutualfundData() as Record<string, unknown>[]} />}
      </div>
    </Layout>
  );
}
