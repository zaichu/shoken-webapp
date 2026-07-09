import { useCallback, useEffect, useReducer } from 'react';
import type { DataActionRailProps } from '@/components/organisms/DataActionRail/DataActionRail';
import type { DividendStatus } from '@/features/dividendPerShare/api/dividendPerShareApi';
import { useDividendBatch } from '@/features/dividendPerShare/hooks/useDividendBatch';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { createSearchOptions } from '@/lib/utils/dataTransformer';
import { filterByConfig, type FilterConfig } from '@/lib/utils/searchUtils';
import type { AssetBalanceData } from '@/types/api';
import type { SearchCategories } from '@/types/common';
import type { AssetBalanceUtilityRailProps } from '../components/AssetBalanceUtilityRail';
import { useAssetBalanceDataSourceCore } from './useAssetBalanceDataSource';

interface AssetBalanceState {
  searchQuery: string;
  showDeleteConfirm: boolean;
}

type AssetBalanceAction =
  | { type: 'SET_SEARCH_QUERY'; payload: string }
  | { type: 'SET_SHOW_DELETE_CONFIRM'; payload: boolean }
  | { type: 'LOGOUT' };

const initialState: AssetBalanceState = {
  searchQuery: '',
  showDeleteConfirm: false,
};

const filterConfig: FilterConfig<AssetBalanceData> = {
  partialStringFields: [
    (item) => item.security_code,
    (item) => item.security_name,
  ],
};

function assetBalanceReducer(
  state: AssetBalanceState,
  action: AssetBalanceAction
): AssetBalanceState {
  switch (action.type) {
    case 'SET_SEARCH_QUERY':
      return { ...state, searchQuery: action.payload };
    case 'SET_SHOW_DELETE_CONFIRM':
      return { ...state, showDeleteConfirm: action.payload };
    case 'LOGOUT':
      return initialState;
    default:
      return state;
  }
}

function getMainStatusMessage(flags: {
  loading: boolean;
  saving: boolean;
  deleting: boolean;
  previewing: boolean;
}) {
  if (flags.loading) {
    return 'データを読み込んでいます...';
  }
  if (flags.saving) {
    return 'データを保存しています...';
  }
  if (flags.deleting) {
    return 'データを削除しています...';
  }
  if (flags.previewing) {
    return 'CSVファイルを解析しています...';
  }
  return null;
}

export function useAssetBalanceState() {
  const { isAuthenticated, isLoading: authLoading, login, onLogout, user } = useAuth();
  const [state, dispatch] = useReducer(assetBalanceReducer, initialState);
  const {
    dbData,
    dbTotal,
    summary,
    facets,
    assetBalanceListLimit,
    previewRows,
    loading,
    error,
    saving,
    deleting,
    previewing,
    lastSavedResult,
    hasCsvFile,
    hasDbData,
    csvFileName,
    handleFileSelect: selectFile,
    handleSaveToDB: saveToDB,
    handleDeleteAll: deleteAll,
    resetState,
  } = useAssetBalanceDataSourceCore({
    isAuthenticated,
    authLoading,
    onLogout,
    userId: user?.id ?? '',
    registerLogoutReset: false,
  });

  useEffect(() => {
    return onLogout(() => {
      resetState();
      dispatch({ type: 'LOGOUT' });
    });
  }, [onLogout, resetState]);

  const handleSearch = (query: string) => {
    dispatch({ type: 'SET_SEARCH_QUERY', payload: query });
  };

  const clearSearch = () => {
    dispatch({ type: 'SET_SEARCH_QUERY', payload: '' });
  };

  const openDeleteConfirm = () => {
    dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: true });
  };

  const closeDeleteConfirm = useCallback(() => {
    dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: false });
  }, []);

  const confirmDeleteAll = useCallback(async () => {
    closeDeleteConfirm();
    await deleteAll();
  }, [closeDeleteConfirm, deleteAll]);

  const assetBalanceData = hasCsvFile ? previewRows : dbData;
  const securityCodes = saving || loading ? [] : assetBalanceData.map((item) => item.security_code);

  const { dividendPerShareMap, dividendStatusMap } = useDividendBatch(securityCodes, isAuthenticated);

  const filteredData = filterByConfig(assetBalanceData, state.searchQuery, filterConfig);

  // facets は保存済み DB データ（一覧 API）由来のため、CSV プレビュー中は使わない
  const searchCategories: SearchCategories = !hasCsvFile && facets?.securities
    ? {
      securities: facets.securities.map((option) => ({ value: option.value, label: option.label })),
    }
    : {
      securities: createSearchOptions(assetBalanceData, 'security_code', 'security_name', true),
    };

  // summary も一覧 API 由来のため、CSV プレビュー中や検索絞り込み中は使わない
  const portfolioSummary = !hasCsvFile && state.searchQuery === '' ? summary : undefined;

  const dbWarning = !hasCsvFile && dbTotal > assetBalanceListLimit
    ? `一覧は最大${assetBalanceListLimit}件まで表示しています。未表示の銘柄がある可能性があります。`
    : null;

  const saveLabel = previewRows.length > 0
    ? `${previewRows.length}件 全件置換で保存`
    : '全件置換で保存';

  const actionRailProps: DataActionRailProps = {
    onFileSelect: selectFile,
    selectedFileName: csvFileName ?? undefined,
    fileInputDisabled: loading || saving || deleting || previewing,
    hasCsvFile,
    saveLabel: saving ? '保存中...' : previewing ? '解析中...' : saveLabel,
    onSave: saveToDB,
    saveDisabled: saving || deleting || previewing || previewRows.length === 0,
    hasDbData,
    deleteLabel: deleting ? '削除中...' : `全件削除 (${dbData.length}件)`,
    onDeleteRequest: openDeleteConfirm,
    deleteDisabled: saving || deleting || loading,
    saveResult: lastSavedResult,
    saveModeLabel: '全件置換',
  };

  const utilityRailProps: AssetBalanceUtilityRailProps = {
    actionRailProps,
    error,
    warning: dbWarning,
    searchCardProps: {
      visible: assetBalanceData.length > 0,
      categories: searchCategories,
      value: state.searchQuery,
      onSearch: handleSearch,
    },
    reviewPromptCardProps: {
      assetBalanceData,
    },
  };

  const mainStatusMessage = getMainStatusMessage({ loading, saving, deleting, previewing });

  return {
    isAuthenticated,
    authLoading,
    login,
    assetBalanceData,
    filteredData,
    portfolioSummary,
    clearSearch,
    utilityRailProps,
    showDeleteConfirm: state.showDeleteConfirm,
    closeDeleteConfirm,
    confirmDeleteAll,
    dbDataCount: dbData.length,
    deleteModalLoading: deleting,
    showPortfolioSummary: !loading && !previewing,
    mainStatusMessage,
    workspaceBusy: authLoading || loading || saving || deleting || previewing,
    dividendPerShareMap: dividendPerShareMap as Map<string, number>,
    dividendStatusMap: dividendStatusMap as Map<string, DividendStatus> | undefined,
  };
}
