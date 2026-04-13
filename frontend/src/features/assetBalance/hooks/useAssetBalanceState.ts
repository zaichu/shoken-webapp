import { useCallback, useEffect, useMemo, useReducer } from 'react';
import type { DataActionRailProps } from '@/components/organisms/DataActionRail/DataActionRail';
import type { DividendStatus } from '@/features/jquants/api/dividendPerShareApi';
import { useDividendBatch } from '@/features/jquants/hooks/useDividendBatch';
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

  const handleSearch = useCallback((query: string) => {
    dispatch({ type: 'SET_SEARCH_QUERY', payload: query });
  }, []);

  const clearSearch = useCallback(() => {
    dispatch({ type: 'SET_SEARCH_QUERY', payload: '' });
  }, []);

  const openDeleteConfirm = useCallback(() => {
    dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: true });
  }, []);

  const closeDeleteConfirm = useCallback(() => {
    dispatch({ type: 'SET_SHOW_DELETE_CONFIRM', payload: false });
  }, []);

  const confirmDeleteAll = useCallback(async () => {
    closeDeleteConfirm();
    await deleteAll();
  }, [closeDeleteConfirm, deleteAll]);

  const assetBalanceData = useMemo(() => {
    if (previewRows.length > 0) {
      return previewRows;
    }
    if (dbData.length > 0) {
      return dbData;
    }
    return [];
  }, [previewRows, dbData]);

  const securityCodes = useMemo(
    () => (saving || loading ? [] : assetBalanceData.map((item) => item.security_code)),
    [saving, loading, assetBalanceData]
  );

  const { dividendPerShareMap, dividendStatusMap } = useDividendBatch(securityCodes, isAuthenticated);

  const filteredData = useMemo(
    () => filterByConfig(assetBalanceData, state.searchQuery, filterConfig),
    [assetBalanceData, state.searchQuery]
  );

  const searchCategories = useMemo<SearchCategories>(
    () => ({
      securities: createSearchOptions(assetBalanceData, 'security_code', 'security_name', true),
    }),
    [assetBalanceData]
  );

  const saveLabel = previewRows.length > 0
    ? `${previewRows.length}件 全件置換で保存`
    : '全件置換で保存';

  const actionRailProps = useMemo<DataActionRailProps>(
    () => ({
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
    }),
    [
      selectFile,
      csvFileName,
      loading,
      saving,
      deleting,
      previewing,
      hasCsvFile,
      saveLabel,
      saveToDB,
      previewRows.length,
      hasDbData,
      dbData.length,
      openDeleteConfirm,
      lastSavedResult,
    ]
  );

  const utilityRailProps = useMemo<AssetBalanceUtilityRailProps>(
    () => ({
      actionRailProps,
      error,
      searchCardProps: {
        visible: assetBalanceData.length > 0,
        categories: searchCategories,
        value: state.searchQuery,
        onSearch: handleSearch,
      },
    }),
    [actionRailProps, error, assetBalanceData.length, searchCategories, state.searchQuery, handleSearch]
  );

  const mainStatusMessage = getMainStatusMessage({ loading, saving, deleting, previewing });

  return {
    isAuthenticated,
    authLoading,
    login,
    assetBalanceData,
    filteredData,
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
