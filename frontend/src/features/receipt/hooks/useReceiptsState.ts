import { useEffect, useReducer, useRef, type KeyboardEvent } from 'react';
import type { DataActionRailProps } from '@/components/organisms/DataActionRail/DataActionRail';
import { useAuth } from '@/features/auth/hooks/useAuth';
import type { ReceiptsUtilityRailProps } from '../components/ReceiptsUtilityRail';
import { TAB_LABEL, TABS } from '../reducer';
import {
  transformDBDividend,
  transformDBDomesticStock,
  transformDBMutualfund,
} from '../parsers';
import { initialState, receiptsReducer, type ReceiptsType } from '../reducer';
import { useReceiptsData } from './useReceiptsData';

export function useReceiptsState() {
  const { isAuthenticated, isLoading: authLoading, onLogout } = useAuth();
  const [state, dispatch] = useReducer(receiptsReducer, initialState);
  const {
    data,
    summaries,
    dbLoading,
    loadingByTab,
    dbError,
    saving,
    deleting,
    previewing,
    receiptListLimit,
    uploadCsv,
    previewCsv,
    deleteAll,
  } = useReceiptsData(state.receiptsType);

  useEffect(() => {
    return onLogout(() => {
      dispatch({ type: 'LOGOUT' });
    });
  }, [onLogout]);

  const { receiptsType, rawFiles, csvPreviews, lastImportResults, showDeleteConfirm } = state;
  // 選択中タブの取得完了だけを待つ。他タブのバックグラウンド取得はブロックしない
  const activeTabLoading = loadingByTab[receiptsType];
  const tablistRef = useRef<HTMLDivElement>(null);
  const currentData = data[receiptsType];
  const rawFile = rawFiles[receiptsType];
  const csvPreview = csvPreviews[receiptsType];
  const importResult = lastImportResults[receiptsType];
  const dbDataCount = currentData.length;
  const dbWarning = dbDataCount >= receiptListLimit
    ? '一覧は最大' + receiptListLimit + '件まで表示しています。検索条件を絞り込んでください。'
    : null;
  const hasCsvFile = rawFile !== null;
  const tabName = TAB_LABEL[receiptsType];
  const saveLabel = csvPreview ? `${csvPreview.validRows}件 追加で保存` : '追加で保存';

  const counts = {
    dividend: data.dividend.length,
    domesticstock: data.domesticstock.length,
    mutualfund: data.mutualfund.length,
  };

  const previewData = {
    dividend: csvPreviews.dividend?.rows.map((row) => transformDBDividend(row)),
    domesticstock: csvPreviews.domesticstock?.rows.map((row) => transformDBDomesticStock(row)),
    mutualfund: csvPreviews.mutualfund?.rows.map((row) => transformDBMutualfund(row)),
  };

  const setReceiptsType = (tab: ReceiptsType) => {
    dispatch({ type: 'SET_RECEIPTS_TYPE', payload: tab });
  };

  const handleTabKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    const currentIndex = TABS.indexOf(receiptsType);
    let nextIndex: number | null = null;

    if (event.key === 'ArrowRight') {
      nextIndex = (currentIndex + 1) % TABS.length;
    } else if (event.key === 'ArrowLeft') {
      nextIndex = (currentIndex - 1 + TABS.length) % TABS.length;
    } else if (event.key === 'Home') {
      nextIndex = 0;
    } else if (event.key === 'End') {
      nextIndex = TABS.length - 1;
    }

    if (nextIndex === null) {
      return;
    }

    event.preventDefault();
    const nextTab = TABS[nextIndex];
    dispatch({ type: 'SET_RECEIPTS_TYPE', payload: nextTab });
    const buttons = tablistRef.current?.querySelectorAll<HTMLButtonElement>('[role="tab"]');
    buttons?.[nextIndex]?.focus();
  };

  const handleFileSelect = (file: File) => {
    dispatch({ type: 'SET_RAW_FILE', receiptsType, payload: file });
    previewCsv({
      type: receiptsType,
      file,
      onSuccess: (result) => {
        dispatch({ type: 'SET_CSV_PREVIEW', receiptsType, payload: result });
      },
    });
  };

  const handleSaveToDB = () => {
    if (!isAuthenticated || rawFile === null) {
      return;
    }

    uploadCsv({
      type: receiptsType,
      file: rawFile,
      onSuccess: (result) => {
        dispatch({ type: 'SET_RAW_FILE', receiptsType, payload: null });
        dispatch({ type: 'SET_IMPORT_RESULT', receiptsType, payload: result });
      },
    });
  };

  const openDeleteConfirm = () => {
    dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: true });
  };

  const closeDeleteConfirm = () => {
    dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: false });
  };

  const confirmDeleteAll = () => {
    if (!isAuthenticated) {
      return;
    }

    closeDeleteConfirm();
    deleteAll(receiptsType, {
      onSuccess: () => {
        dispatch({ type: 'CLEAR_IMPORT_RESULT', receiptsType });
      },
    });
  };

  const actionRailProps: DataActionRailProps = {
    onFileSelect: handleFileSelect,
    selectedFileName: rawFile?.name ?? undefined,
    fileInputDisabled: dbLoading || saving || deleting || previewing || authLoading,
    hasCsvFile: isAuthenticated && hasCsvFile,
    saveLabel: saving ? '保存中...' : previewing ? '解析中...' : saveLabel,
    onSave: handleSaveToDB,
    saveDisabled: saving || deleting || previewing,
    hasDbData: isAuthenticated && dbDataCount > 0,
    deleteLabel: deleting ? '削除中...' : `全件削除 (${dbDataCount}件)`,
    onDeleteRequest: openDeleteConfirm,
    deleteDisabled: saving || deleting || dbLoading,
    saveResult: importResult,
    saveModeLabel: '追加保存',
  };

  const utilityRailProps: ReceiptsUtilityRailProps = {
    actionRailProps,
    alertsProps: {
      dbError,
      dbWarning,
      hasCsvFile,
      previewing,
      csvPreview,
    },
    authLoading,
    dbLoading,
  };

  return {
    receiptsType,
    counts,
    tablistRef,
    setReceiptsType,
    handleTabKeyDown,
    showDeleteConfirm,
    closeDeleteConfirm,
    confirmDeleteAll,
    tabName,
    dbDataCount,
    data,
    summaries,
    previewData,
    utilityRailProps,
    deleteModalLoading: deleting,
    initialLoading: authLoading || activeTabLoading,
    workspaceBusy: authLoading || dbLoading || saving || deleting,
  };
}
