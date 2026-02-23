import { useReducer, useEffect, useCallback, useMemo } from 'react';
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
import { getDisplayErrorMessage } from '@/lib/utils/errorHandler';
import { ConfirmDeleteModal } from '@/components/molecules/ConfirmDeleteModal/ConfirmDeleteModal';
import { type ReceiptsType, initialState, receiptsReducer } from './receiptsReducer';
import { useReceiptsData } from '@/features/receipt/hooks/useReceiptsData';

// 明細種類ごとのラベル
const TAB_LABEL: Record<ReceiptsType, string> = {
  dividend: '配当金',
  domesticstock: '国内株式',
  mutualfund: '投資信託',
};

/**
 * 明細種類ごとにCSVデータを管理するページコンポーネント
 * ログイン時はDBからデータを取得、未ログイン時はCSVから取得
 */
export function ReceiptsPage() {
  const { isAuthenticated, isLoading: authLoading, onLogout } = useAuth();
  const [state, dispatch] = useReducer(receiptsReducer, initialState);
  const { receiptsType, csvData, showDeleteConfirm } = state;

  // 各明細種類ごとにCSVリーダーフックを作成
  const dividendCSV = useCSVReader();
  const domesticStockCSV = useCSVReader();
  const mutualfundCSV = useCSVReader();

  // TanStack Query ベースのデータ管理
  const {
    dividendData,
    domesticstockData,
    mutualfundData,
    dbLoading,
    dbError,
    saving,
    deleting,
    bulkCreate,
    deleteAll,
    clearCache,
  } = useReceiptsData();

  // ログアウト時に CSV データと Query キャッシュをクリア
  useEffect(() => {
    return onLogout(() => {
      dispatch({ type: 'LOGOUT' });
      clearCache();
    });
  }, [onLogout, clearCache]);

  // ランタイムデータマッピング（switch削減用）
  const runtimeDataMap = useMemo(() => ({
    dividend: {
      csvReader: dividendCSV,
      csvData: csvData.dividend,
      setCsvData: (data: Record<string, unknown>[]) =>
        dispatch({ type: 'SET_CSV_DATA', receiptsType: 'dividend', payload: data }),
    },
    domesticstock: {
      csvReader: domesticStockCSV,
      csvData: csvData.domesticstock,
      setCsvData: (data: Record<string, unknown>[]) =>
        dispatch({ type: 'SET_CSV_DATA', receiptsType: 'domesticstock', payload: data }),
    },
    mutualfund: {
      csvReader: mutualfundCSV,
      csvData: csvData.mutualfund,
      setCsvData: (data: Record<string, unknown>[]) =>
        dispatch({ type: 'SET_CSV_DATA', receiptsType: 'mutualfund', payload: data }),
    },
  }), [dividendCSV, domesticStockCSV, mutualfundCSV, csvData]);

  /**
   * 現在選択中のタブに応じてCSV処理を切り替える
   */
  const handleFileSelect = useCallback(async (file: File) => {
    const { csvReader, setCsvData } = runtimeDataMap[receiptsType];
    try {
      setCsvData(await csvReader.parseCSV(file));
      if (csvReader.error) csvReader.resetError();
    } catch (e) {
      // CSV parse error is surfaced via csvReader.error
      void getDisplayErrorMessage(e, 'CSVファイルの読み込みに失敗しました');
    }
  }, [receiptsType, runtimeDataMap]);

  /**
   * CSVデータをDBに保存
   */
  const handleSaveToDB = useCallback(() => {
    const { csvData: tabCsvData, setCsvData } = runtimeDataMap[receiptsType];
    if (!isAuthenticated || tabCsvData.length === 0) return;
    bulkCreate({
      type: receiptsType,
      csvData: tabCsvData,
      onSuccess: () => setCsvData([]),
    });
  }, [bulkCreate, isAuthenticated, receiptsType, runtimeDataMap]);

  /**
   * DBデータを全削除
   */
  const handleDeleteAll = useCallback(() => {
    if (!isAuthenticated) return;
    dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: false });
    deleteAll(receiptsType);
  }, [deleteAll, isAuthenticated, receiptsType]);

  /**
   * 現在選択中のタブに対応するCSV状態を取得
   */
  const currentCSVState = useMemo(() => {
    const { csvReader, csvData: tabCsvData } = runtimeDataMap[receiptsType];
    return {
      isLoading: csvReader.isLoading,
      error: csvReader.error,
      fileName: csvReader.fileName,
      tabCsvData,
    };
  }, [runtimeDataMap, receiptsType]);

  const { isLoading, error, fileName, tabCsvData } = currentCSVState;
  const hasCsvData = tabCsvData.length > 0;

  // 現在のタブのDBデータ
  const currentDbData = useMemo(() => ({
    dividend: dividendData,
    domesticstock: domesticstockData,
    mutualfund: mutualfundData,
  }), [dividendData, domesticstockData, mutualfundData]);

  const dbDataCount = currentDbData[receiptsType].length;
  const hasDbData = dbDataCount > 0;
  const tabName = TAB_LABEL[receiptsType];

  // 表示用データを決定（ログイン時はDB優先、未ログイン時はCSV）
  const dividendDisplayData = isAuthenticated && dividendData.length > 0 ? dividendData : csvData.dividend;
  const domesticStockDisplayData = isAuthenticated && domesticstockData.length > 0 ? domesticstockData : csvData.domesticstock;
  const mutualfundDisplayData = isAuthenticated && mutualfundData.length > 0 ? mutualfundData : csvData.mutualfund;

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
            return (
              <button
                key={tab}
                className={`px-4 py-3 text-sm font-medium transition-colors border-b-2 -mb-px ${
                  isActive
                    ? 'border-primary text-primary bg-white'
                    : 'border-transparent text-slate-600 hover:text-slate-900 hover:border-slate-300'
                }`}
                onClick={() => dispatch({ type: 'SET_RECEIPTS_TYPE', payload: tab })}
                type="button"
                role="tab"
                aria-selected={isActive}
              >
                {TAB_LABEL[tab]}
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
                    onClick={() => dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: true })}
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
            {receiptsType === 'dividend' && <Dividend csvData={dividendDisplayData as Record<string, unknown>[]} />}
            {receiptsType === 'domesticstock' && <DomesticStock csvData={domesticStockDisplayData as Record<string, unknown>[]} />}
            {receiptsType === 'mutualfund' && <Mutualfund csvData={mutualfundDisplayData as Record<string, unknown>[]} />}
          </>
        )}

        <ConfirmDeleteModal
          isOpen={showDeleteConfirm}
          onConfirm={handleDeleteAll}
          onCancel={() => dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: false })}
          title={`${tabName}データの全件削除`}
          description={`【${tabName}】のデータをすべて削除します。`}
          itemCount={dbDataCount}
          loading={deleting}
        />
      </div>
    </Layout>
  );
}
