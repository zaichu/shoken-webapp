import { useReducer, useEffect, useCallback } from 'react';
import { Layout } from '../components/templates/Layout';
import { PageHeader } from '../components/atoms/PageHeader';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { Alert } from '@/components/atoms/Alert';
import { Button } from '@/components/atoms/Button';
import { Spinner } from '@/components/atoms/Spinner';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { Dividend } from './Receipt/Dividend';
import { DomesticStock } from './Receipt/DomesticStock';
import { Mutualfund } from './Receipt/Mutualfund';
import { ConfirmDeleteModal } from '@/components/molecules/ConfirmDeleteModal/ConfirmDeleteModal';
import { type ReceiptsType, initialState, receiptsReducer } from './receiptsReducer';
import { useReceiptsData } from '@/features/receipt/hooks/useReceiptsData';
import {
  transformDBDividend,
  transformDBDomesticStock,
  transformDBMutualfund,
} from '@/features/receipt/parsers';

// 明細種類ごとのラベル
const TAB_LABEL: Record<ReceiptsType, string> = {
  dividend: '配当金',
  domesticstock: '国内株式',
  mutualfund: '投資信託',
};

/**
 * 明細種類ごとにCSVデータを管理するページコンポーネント
 */
export function ReceiptsPage() {
  const { isAuthenticated, isLoading: authLoading, onLogout } = useAuth();
  const [state, dispatch] = useReducer(receiptsReducer, initialState);
  const { receiptsType, rawFiles, lastImportResults, showDeleteConfirm } = state;

  // TanStack Query ベースのデータ管理
  const {
    dividendData,
    domesticstockData,
    mutualfundData,
    dbLoading,
    dbError,
    saving,
    deleting,
    previewing,
    uploadCsv,
    previewCsv,
    deleteAll,
  } = useReceiptsData();

  // ログアウト時に状態をクリア（Query キャッシュは useReceiptsData が内部処理）
  useEffect(() => {
    return onLogout(() => {
      dispatch({ type: 'LOGOUT' });
    });
  }, [onLogout]);

  // ファイル名表示用（ファイル選択後のみ表示）
  const rawFile = rawFiles[receiptsType];
  const selectedFileName = rawFile?.name ?? undefined;
  const csvPreview = state.csvPreviews[receiptsType];

  /**
   * ファイル選択時にrawFileを保存しプレビューを取得する
   */
  const handleFileSelect = useCallback((file: File) => {
    dispatch({ type: 'SET_RAW_FILE', receiptsType, payload: file });
    previewCsv({
      type: receiptsType,
      file,
      onSuccess: (result) => {
        dispatch({ type: 'SET_CSV_PREVIEW', receiptsType, payload: result });
      },
    });
  }, [receiptsType, previewCsv]);

  /**
   * CSVファイルをバックエンドに送信して保存
   */
  const handleSaveToDB = useCallback(() => {
    if (!isAuthenticated || rawFile === null) return;
    uploadCsv({
      type: receiptsType,
      file: rawFile,
      onSuccess: (result) => {
        dispatch({ type: 'SET_RAW_FILE', receiptsType, payload: null });
        dispatch({ type: 'SET_IMPORT_RESULT', receiptsType, payload: result });
      },
    });
  }, [uploadCsv, isAuthenticated, receiptsType, rawFile]);

  /**
   * DBデータを全削除
   */
  const handleDeleteAll = useCallback(() => {
    if (!isAuthenticated) return;
    dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: false });
    deleteAll(receiptsType, {
      // 削除成功時のみimport resultをクリア（失敗時は保持）
      onSuccess: () => dispatch({ type: 'CLEAR_IMPORT_RESULT', receiptsType }),
    });
  }, [deleteAll, isAuthenticated, receiptsType]);

  const hasCsvFile = rawFile !== null;
  const dbDataCount = (receiptsType === 'dividend' ? dividendData : receiptsType === 'domesticstock' ? domesticstockData : mutualfundData).length;
  const hasDbData = dbDataCount > 0;
  const tabName = TAB_LABEL[receiptsType];
  const importResult = lastImportResults[receiptsType];
  const saveLabel = csvPreview
    ? `${csvPreview.validRows}件 追加で保存`
    : '追加で保存';

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
      <div className="mt-2" aria-busy={dbLoading || authLoading || saving || deleting}>
        <div className="action-toolbar">
          <div className="form-input-container">
            <CSVFileInput
              onFileSelect={handleFileSelect}
              selectedFileName={selectedFileName}
              disabled={dbLoading || saving || deleting || previewing || authLoading}
            />
          </div>
          {isAuthenticated && (
            <>
              <div className="action-button-group" role="group" aria-label="データ操作">
                {hasCsvFile && (
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={handleSaveToDB}
                    disabled={saving || deleting || previewing}
                    aria-disabled={saving || deleting || previewing}
                  >
                    {saving ? '保存中...' : previewing ? '解析中...' : saveLabel}
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

        {dbError && (
          <Alert variant="danger" className="my-3" role="alert" aria-live="assertive">
            <strong>エラー:</strong> {dbError}
          </Alert>
        )}

        {hasCsvFile && !previewing && csvPreview && !importResult && (
          <div className="my-3" role="status" aria-live="polite">
            <Alert variant={csvPreview.errors.length > 0 ? 'warning' : 'info'}>
              <p>
                <strong>{csvPreview.validRows}件 追加で保存されます</strong>
                {csvPreview.errors.length > 0 && ` / ${csvPreview.errors.length}件エラー`}
                <span className="ml-2 text-xs text-secondary">（保存モード: 追加）</span>
              </p>
              {csvPreview.errors.length > 0 && (
                <ul className="mt-2 list-disc list-inside text-sm space-y-1">
                  {csvPreview.errors.map((e) => (
                    <li key={e.row}>{e.row}行目: {e.message}</li>
                  ))}
                </ul>
              )}
            </Alert>
          </div>
        )}

        {importResult && (() => {
          const hasErrors = importResult.errors.length > 0;
          return (
            <div className="my-3" role="status" aria-live="polite">
              <Alert variant={hasErrors ? 'warning' : 'success'}>
                <p className="flex flex-wrap items-center gap-x-2">
                  <strong>{importResult.inserted}件登録</strong>
                  {importResult.skipped > 0 && (
                    <span className="inline-flex items-center rounded-full bg-slate-100 px-2.5 py-0.5 text-xs font-medium text-slate-600">
                      {importResult.skipped}件スキップ（重複）
                    </span>
                  )}
                  {hasErrors && (
                    <span className="text-sm text-secondary">
                      {importResult.errors.length}件エラー
                    </span>
                  )}
                </p>
                {hasErrors && (
                  <ul className="mt-2 list-disc list-inside text-sm space-y-1">
                    {importResult.errors.map((e) => (
                      <li key={e.row}>{e.row}行目: {e.message}</li>
                    ))}
                  </ul>
                )}
              </Alert>
            </div>
          );
        })()}

        <div aria-live="polite" aria-atomic="true">
          {(dbLoading || authLoading) && (
            <div className="status-message" role="status">
              <Spinner size="md" className="text-primary" />
              <p className="text-sm text-secondary">
                {authLoading && '認証状態を確認しています...'}
                {dbLoading && 'データを読み込んでいます...'}
              </p>
            </div>
          )}
        </div>

        {/* ローディング完了後のみコンテンツを表示（0円集計との同時表示を防止） */}
        {!authLoading && !dbLoading && (
          <>
            {receiptsType === 'dividend' && (
              <Dividend
                data={dividendData}
                previewData={csvPreview?.rows?.map(r => transformDBDividend(r))}
                importResult={importResult}
              />
            )}
            {receiptsType === 'domesticstock' && (
              <DomesticStock
                data={domesticstockData}
                previewData={csvPreview?.rows?.map(r => transformDBDomesticStock(r))}
                importResult={importResult}
              />
            )}
            {receiptsType === 'mutualfund' && (
              <Mutualfund
                data={mutualfundData}
                previewData={csvPreview?.rows?.map(r => transformDBMutualfund(r))}
                importResult={importResult}
              />
            )}
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
