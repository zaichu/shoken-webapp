import { useEffect } from 'react';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { useAssetBalanceDataSourceCore } from './useAssetBalanceDataSource';
import { useAssetBalanceSearch } from './useAssetBalanceSearch';
import { useAssetBalanceDeleteConfirm } from './useAssetBalanceDeleteConfirm';
import { useAssetBalanceDividendBatch } from './useAssetBalanceDividendBatch';
import { buildActionRailProps, buildUtilityRailProps, getMainStatusMessage } from './assetBalanceUiProps';

export function useAssetBalanceState() {
  const { isAuthenticated, isLoading: authLoading, login, onLogout, user } = useAuth();
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

  const assetBalanceData = hasCsvFile ? previewRows : dbData;

  const {
    searchQuery,
    handleSearch,
    clearSearch,
    resetSearch,
    filteredData,
    searchCategories,
  } = useAssetBalanceSearch(assetBalanceData, facets, hasCsvFile);

  const {
    showDeleteConfirm,
    openDeleteConfirm,
    closeDeleteConfirm,
    confirmDeleteAll,
    resetDeleteConfirm,
  } = useAssetBalanceDeleteConfirm(deleteAll);

  useEffect(() => {
    return onLogout(() => {
      resetState();
      resetSearch();
      resetDeleteConfirm();
    });
  }, [onLogout, resetState, resetSearch, resetDeleteConfirm]);

  const { dividendPerShareMap, dividendStatusMap } = useAssetBalanceDividendBatch(
    assetBalanceData,
    isAuthenticated,
    saving || loading
  );

  // summary は一覧 API 由来のため、CSV プレビュー中や検索絞り込み中は使わない
  const portfolioSummary = !hasCsvFile && searchQuery === '' ? summary : undefined;

  const dbWarning = !hasCsvFile && dbTotal > assetBalanceListLimit
    ? `一覧は最大${assetBalanceListLimit}件まで表示しています。未表示の銘柄がある可能性があります。`
    : null;

  const actionRailProps = buildActionRailProps({
    onFileSelect: selectFile,
    csvFileName,
    loading,
    saving,
    deleting,
    previewing,
    hasCsvFile,
    previewRowCount: previewRows.length,
    onSave: saveToDB,
    hasDbData,
    dbTotal,
    onDeleteRequest: openDeleteConfirm,
    lastSavedResult,
  });

  const utilityRailProps = buildUtilityRailProps({
    actionRailProps,
    error,
    dbWarning,
    assetBalanceData,
    searchCategories,
    searchQuery,
    onSearch: handleSearch,
  });

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
    showDeleteConfirm,
    closeDeleteConfirm,
    confirmDeleteAll,
    dbDataCount: dbTotal,
    deleteModalLoading: deleting,
    showPortfolioSummary: !loading && !previewing,
    mainStatusMessage,
    workspaceBusy: authLoading || loading || saving || deleting || previewing,
    dividendPerShareMap,
    dividendStatusMap,
  };
}
