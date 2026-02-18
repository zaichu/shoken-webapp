import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { Layout } from '../components/templates/Layout';
import { PageHeader } from '../components/atoms/PageHeader';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { Alert } from '@/components/atoms/Alert';
import { Button } from '@/components/atoms/Button';
import { Spinner } from '@/components/atoms/Spinner';
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
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';
import { ConfirmDeleteModal } from '@/components/molecules/ConfirmDeleteModal/ConfirmDeleteModal';

type ReceiptsType = 'dividend' | 'domesticstock' | 'mutualfund';

// 明細種類ごとの静的設定
interface ReceiptTypeStaticConfig {
  api: {
    /* eslint-disable @typescript-eslint/no-explicit-any */
    list: () => Promise<any[]>;
    bulkCreate: (items: any[]) => Promise<any>;
    deleteAll: () => Promise<any>;
    /* eslint-enable @typescript-eslint/no-explicit-any */
  };
  parser: (item: Record<string, unknown>) => unknown;
  transformer: (item: Record<string, unknown>) => unknown;
}

const RECEIPT_TYPE_CONFIG: Record<ReceiptsType, ReceiptTypeStaticConfig> = {
  dividend: {
    api: dividendApi,
    parser: parseDividendCsvItem,
    transformer: transformDBDividend,
  },
  domesticstock: {
    api: domesticStockApi,
    parser: parseDomesticStockCsvItem,
    transformer: transformDBDomesticStock,
  },
  mutualfund: {
    api: mutualfundApi,
    parser: parseMutualfundCsvItem,
    transformer: transformDBMutualfund,
  },
};

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
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  // 各明細種類ごとにCSVリーダーフックを作成
  const dividendCSV = useCSVReader();
  const domesticStockCSV = useCSVReader();
  const mutualfundCSV = useCSVReader();

  // ランタイムデータマッピング（switch削減用）
  const runtimeDataMap = useMemo(() => ({
    dividend: {
      csvReader: dividendCSV,
      csvData: dividendCsvData,
      setCsvData: setDividendCsvData,
      dbData: dividendDBData,
      setDbData: setDividendDBData,
    },
    domesticstock: {
      csvReader: domesticStockCSV,
      csvData: domesticStockCsvData,
      setCsvData: setDomesticStockCsvData,
      dbData: domesticStockDBData,
      setDbData: setDomesticStockDBData,
    },
    mutualfund: {
      csvReader: mutualfundCSV,
      csvData: mutualfundCsvData,
      setCsvData: setMutualfundCsvData,
      dbData: mutualfundDBData,
      setDbData: setMutualfundDBData,
    },
  }), [
    dividendCSV, dividendCsvData, dividendDBData,
    domesticStockCSV, domesticStockCsvData, domesticStockDBData,
    mutualfundCSV, mutualfundCsvData, mutualfundDBData,
  ]);

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

    await Promise.all([
      dividendApi.list(),
      domesticStockApi.list(),
      mutualfundApi.list(),
    ]).then(([dividends, stocks, funds]) => {
      setDividendDBData(dividends.map(d => transformDBDividend(d as unknown as Record<string, unknown>)));
      setDomesticStockDBData(stocks.map(d => transformDBDomesticStock(d as unknown as Record<string, unknown>)));
      setMutualfundDBData(funds.map(d => transformDBMutualfund(d as unknown as Record<string, unknown>)));
      hasFetched.current = true;
    }).catch((err) => {
      setDbError(getDisplayErrorMessage(err, 'データ取得に失敗しました'));
    });

    setDbLoading(false);
    isFetchingRef.current = false;
  }, [isAuthenticated, authLoading]);

  // 認証状態が確定したらDBからデータを取得
  useEffect(() => {
    if (authLoading) return;
    const timeoutId = window.setTimeout(() => {
      void fetchFromDB();
    }, 0);
    return () => {
      window.clearTimeout(timeoutId);
    };
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
    const { csvReader, setCsvData } = runtimeDataMap[receiptsType];
    try {
      setCsvData(await csvReader.parseCSV(file));
      if (csvReader.error) csvReader.resetError();
    } catch (e) {
      setDbError(getDisplayErrorMessage(e, 'CSVファイルの読み込みに失敗しました'));
    }
  };

  /**
   * CSVデータをDBに保存
   */
  const handleSaveToDB = async () => {
    if (!isAuthenticated) return;

    setSaving(true);
    setDbError(null);

    const config = RECEIPT_TYPE_CONFIG[receiptsType];
    const { csvData, setCsvData } = runtimeDataMap[receiptsType];

    const items = csvData.map(config.parser);
    await config.api.bulkCreate(items).then(async () => {
      setCsvData([]);
      await fetchFromDB(true);
    }).catch((err) => {
      setDbError(getDisplayErrorMessage(err, '保存に失敗しました'));
    });

    setSaving(false);
  };

  /**
   * DBデータを全削除
   */
  const handleDeleteAll = async () => {
    if (!isAuthenticated) return;

    setShowDeleteConfirm(false);
    setDeleting(true);
    setDbError(null);

    const config = RECEIPT_TYPE_CONFIG[receiptsType];
    const { setDbData } = runtimeDataMap[receiptsType];

    await config.api.deleteAll().then(() => {
      setDbData([]);
    }).catch((err) => {
      setDbError(getDisplayErrorMessage(err, '削除に失敗しました'));
    });

    setDeleting(false);
  };

  /**
   * 現在選択中のタブに対応するエラーとローディング状態を取得
   */
  const currentCSVState = useMemo(() => {
    const { csvReader, csvData } = runtimeDataMap[receiptsType];
    return {
      isLoading: csvReader.isLoading,
      error: csvReader.error,
      fileName: csvReader.fileName,
      csvData,
    };
  }, [runtimeDataMap, receiptsType]);

  const { isLoading, error, fileName, csvData } = currentCSVState;
  const hasCsvData = csvData.length > 0;

  // 現在のタブのDBデータ件数を取得
  const dbDataCount = runtimeDataMap[receiptsType].dbData.length;

  // 現在のタブのDBデータがあるか判定
  const hasDbData = dbDataCount > 0;

  // タブ名の取得
  const tabName = receiptsType === 'dividend' ? '配当金' : receiptsType === 'domesticstock' ? '国内株式' : '投資信託';

  // 表示用データを決定（ログイン時はDB優先、未ログイン時はCSV）
  const dividendData = useMemo(
    () => (isAuthenticated && dividendDBData.length > 0 ? dividendDBData : dividendCsvData),
    [isAuthenticated, dividendDBData, dividendCsvData]
  );
  const domesticStockData = useMemo(
    () => (isAuthenticated && domesticStockDBData.length > 0 ? domesticStockDBData : domesticStockCsvData),
    [isAuthenticated, domesticStockDBData, domesticStockCsvData]
  );
  const mutualfundData = useMemo(
    () => (isAuthenticated && mutualfundDBData.length > 0 ? mutualfundDBData : mutualfundCsvData),
    [isAuthenticated, mutualfundDBData, mutualfundCsvData]
  );

  return (
    <Layout>
      <PageHeader
        title="取引明細"
        description="配当金・国内株式・投資信託の取引明細を管理します。"
      />
      <nav className="border-b border-slate-200 no-print">
        <div className="flex flex-wrap gap-1">
          {(['dividend', 'domesticstock', 'mutualfund'] as const).map((tab) => {
            const isActive = receiptsType === tab;
            const label = tab === 'dividend' ? '配当金' : tab === 'domesticstock' ? '国内株式' : '投資信託';
            return (
              <button
                key={tab}
                className={`px-4 py-3 text-sm font-medium transition-colors border-b-2 -mb-px ${
                  isActive
                    ? 'border-primary text-primary bg-white'
                    : 'border-transparent text-slate-600 hover:text-slate-900 hover:border-slate-300'
                }`}
                onClick={() => setReceiptsType(tab)}
                type="button"
                role="tab"
                aria-selected={isActive}
              >
                {label}
              </button>
            );
          })}
        </div>
      </nav>
      <div className="mt-2" aria-busy={isLoading || dbLoading || authLoading || saving || deleting}>
        <div className="action-toolbar">
          <div className="form-input-container">
            <CSVFileInput
              onFileSelect={handleFileSelect}
              selectedFileName={fileName}
              disabled={dbLoading || saving || deleting || authLoading}
            />
          </div>
          {isAuthenticated && (
            <>
              <div className="action-button-group" role="group" aria-label="データ操作">
                {hasCsvData && (
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={handleSaveToDB}
                    disabled={saving || deleting}
                    aria-disabled={saving || deleting}
                  >
                    {saving ? '保存中...' : '保存'}
                  </Button>
                )}
              </div>
              {hasDbData && (
                <div className="ml-auto border-l border-slate-300 pl-3">
                  <Button
                    variant="outline-danger"
                    size="sm"
                    onClick={() => setShowDeleteConfirm(true)}
                    disabled={saving || deleting || dbLoading}
                    aria-disabled={saving || deleting || dbLoading}
                  >
                    {deleting ? '削除中...' : `全件削除 (${dbDataCount}件)`}
                  </Button>
                </div>
              )}
            </>
          )}
        </div>

        {(error || dbError) && (
          <Alert variant="danger" className="my-3" role="alert" aria-live="assertive">
            <strong>エラー:</strong> {error || dbError}
          </Alert>
        )}

        <div aria-live="polite" aria-atomic="true">
          {(isLoading || dbLoading || authLoading) && (
            <div className="status-message" role="status">
              <Spinner size="md" className="text-primary" />
              <p className="text-sm text-secondary">
                {authLoading && '認証状態を確認しています...'}
                {dbLoading && 'データを読み込んでいます...'}
                {isLoading && 'CSVファイルを処理しています...'}
              </p>
            </div>
          )}
        </div>

        {/* ローディング完了後のみコンテンツを表示（0円集計との同時表示を防止） */}
        {!authLoading && !dbLoading && !isLoading && (
          <>
            {receiptsType === 'dividend' && <Dividend csvData={dividendData as Record<string, unknown>[]} />}
            {receiptsType === 'domesticstock' && <DomesticStock csvData={domesticStockData as Record<string, unknown>[]} />}
            {receiptsType === 'mutualfund' && <Mutualfund csvData={mutualfundData as Record<string, unknown>[]} />}
          </>
        )}

        <ConfirmDeleteModal
          isOpen={showDeleteConfirm}
          onConfirm={handleDeleteAll}
          onCancel={() => setShowDeleteConfirm(false)}
          title={`${tabName}データの全件削除`}
          description={`【${tabName}】のデータをすべて削除します。`}
          itemCount={dbDataCount}
          loading={deleting}
        />
      </div>
    </Layout>
  );
}
