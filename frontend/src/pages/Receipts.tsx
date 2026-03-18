import { useReducer, useEffect, useCallback, useRef } from 'react';
import { Layout } from '../components/templates/Layout';
import { PageHeader } from '../components/atoms/PageHeader';
import { usePageTitle } from '../hooks/usePageTitle';
import { Spinner } from '@/components/atoms/Spinner';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { Dividend } from './Receipt/Dividend';
import { DomesticStock } from './Receipt/DomesticStock';
import { Mutualfund } from './Receipt/Mutualfund';
import { ConfirmDeleteModal } from '@/components/molecules/ConfirmDeleteModal/ConfirmDeleteModal';
import { initialState, receiptsReducer } from './receiptsReducer';
import { useReceiptsData } from '@/features/receipt/hooks/useReceiptsData';
import {
  transformDBDividend,
  transformDBDomesticStock,
  transformDBMutualfund,
} from '@/features/receipt/parsers';
import { ReceiptsTabNav, TABS, TAB_LABEL } from './ReceiptsTabNav';
import { DataActionRail } from '@/components/organisms/DataActionRail/DataActionRail';
import { ReceiptsAlerts } from './ReceiptsAlerts';

/**
 * 明細種類ごとにCSVデータを管理するページコンポーネント
 */
export function ReceiptsPage() {
  usePageTitle('取引明細');

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

  const rawFile = rawFiles[receiptsType];
  const selectedFileName = rawFile?.name ?? undefined;
  const csvPreview = state.csvPreviews[receiptsType];

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

  const handleDeleteAll = useCallback(() => {
    if (!isAuthenticated) return;
    dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: false });
    deleteAll(receiptsType, {
      onSuccess: () => dispatch({ type: 'CLEAR_IMPORT_RESULT', receiptsType }),
    });
  }, [deleteAll, isAuthenticated, receiptsType]);

  const hasCsvFile = rawFile !== null;
  const dbDataCount = { dividend: dividendData, domesticstock: domesticstockData, mutualfund: mutualfundData }[receiptsType].length;
  const hasDbData = dbDataCount > 0;
  const tabName = TAB_LABEL[receiptsType];
  const importResult = lastImportResults[receiptsType];
  const saveLabel = csvPreview ? `${csvPreview.validRows}件 追加で保存` : '追加で保存';

  const tablistRef = useRef<HTMLDivElement>(null);

  // 矢印キーでのタブ切り替え（roving tabIndex パターン）
  const handleTabKeyDown = useCallback((e: React.KeyboardEvent<HTMLButtonElement>) => {
    const currentIndex = TABS.indexOf(receiptsType);
    let nextIndex: number | null = null;
    if (e.key === 'ArrowRight') nextIndex = (currentIndex + 1) % TABS.length;
    else if (e.key === 'ArrowLeft') nextIndex = (currentIndex - 1 + TABS.length) % TABS.length;
    else if (e.key === 'Home') nextIndex = 0;
    else if (e.key === 'End') nextIndex = TABS.length - 1;
    if (nextIndex !== null) {
      e.preventDefault();
      dispatch({ type: 'SET_RECEIPTS_TYPE', payload: TABS[nextIndex] });
      const buttons = tablistRef.current?.querySelectorAll<HTMLButtonElement>('[role="tab"]');
      buttons?.[nextIndex]?.focus();
    }
  }, [receiptsType]);

  const utilityRail = (
    <>
      <DataActionRail
        onFileSelect={handleFileSelect}
        selectedFileName={selectedFileName}
        fileInputDisabled={dbLoading || saving || deleting || previewing || authLoading}
        hasCsvFile={isAuthenticated && hasCsvFile}
        saveLabel={saving ? '保存中...' : previewing ? '解析中...' : saveLabel}
        onSave={handleSaveToDB}
        saveDisabled={saving || deleting || previewing}
        hasDbData={isAuthenticated && hasDbData}
        deleteLabel={deleting ? '削除中...' : `全件削除 (${dbDataCount}件)`}
        onDeleteRequest={() => dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: true })}
        deleteDisabled={saving || deleting || dbLoading}
        saveResult={importResult}
        saveModeLabel="追加保存"
      />

      <ReceiptsAlerts
        dbError={dbError}
        hasCsvFile={hasCsvFile}
        previewing={previewing}
        csvPreview={csvPreview}
      />

      <div aria-live="polite" aria-atomic="true">
        {(dbLoading || authLoading) && (
          <section className="px-5 py-4" role="status">
            <div className="status-message">
              <Spinner size="md" className="text-primary" />
              <p className="text-sm text-secondary">
                {authLoading && '認証状態を確認しています...'}
                {dbLoading && 'データを読み込んでいます...'}
              </p>
            </div>
          </section>
        )}
      </div>
    </>
  );

  return (
    <Layout>
      <PageHeader
        title="取引明細"
        description="配当金・国内株式・投資信託の取引明細を管理します。"
      />
      <ReceiptsTabNav
        receiptsType={receiptsType}
        tablistRef={tablistRef}
        onTabChange={(tab) => dispatch({ type: 'SET_RECEIPTS_TYPE', payload: tab })}
        onKeyDown={handleTabKeyDown}
        counts={{
          dividend: dividendData.length,
          domesticstock: domesticstockData.length,
          mutualfund: mutualfundData.length,
        }}
      />
      <div className="mt-2" aria-busy={dbLoading || authLoading || saving || deleting}>
        <div data-testid="receipts-workspace">
          {!authLoading && !dbLoading && (
            <>
              <div id="tabpanel-dividend" role="tabpanel" aria-labelledby="tab-dividend" hidden={receiptsType !== 'dividend'}>
                {receiptsType === 'dividend' && (
                  <Dividend
                    data={dividendData}
                    previewData={csvPreview?.rows?.map(r => transformDBDividend(r))}
                    utilityRail={utilityRail}
                  />
                )}
              </div>
              <div id="tabpanel-domesticstock" role="tabpanel" aria-labelledby="tab-domesticstock" hidden={receiptsType !== 'domesticstock'}>
                {receiptsType === 'domesticstock' && (
                  <DomesticStock
                    data={domesticstockData}
                    previewData={csvPreview?.rows?.map(r => transformDBDomesticStock(r))}
                    utilityRail={utilityRail}
                  />
                )}
              </div>
              <div id="tabpanel-mutualfund" role="tabpanel" aria-labelledby="tab-mutualfund" hidden={receiptsType !== 'mutualfund'}>
                {receiptsType === 'mutualfund' && (
                  <Mutualfund
                    data={mutualfundData}
                    previewData={csvPreview?.rows?.map(r => transformDBMutualfund(r))}
                    utilityRail={utilityRail}
                  />
                )}
              </div>
            </>
          )}
        </div>

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
