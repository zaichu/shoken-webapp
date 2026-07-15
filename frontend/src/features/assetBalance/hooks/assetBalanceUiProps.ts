import type { DataActionRailProps } from '@/components/organisms/DataActionRail/DataActionRail';
import type { AssetBalanceData } from '@/types/api';
import type { AssetBalanceUtilityRailProps } from '../components/AssetBalanceUtilityRail';
import type { CsvUploadResult } from '@/lib/csvImport';
import type { SearchCategories } from '@/types/common';

interface BuildActionRailPropsArgs {
  onFileSelect: (file: File) => void;
  csvFileName: string | null;
  loading: boolean;
  saving: boolean;
  deleting: boolean;
  previewing: boolean;
  hasCsvFile: boolean;
  previewRowCount: number;
  onSave: () => void;
  hasDbData: boolean;
  dbTotal: number;
  onDeleteRequest: () => void;
  lastSavedResult: CsvUploadResult | null;
}

/** DataActionRail に渡す props を組み立てる */
export function buildActionRailProps({
  onFileSelect,
  csvFileName,
  loading,
  saving,
  deleting,
  previewing,
  hasCsvFile,
  previewRowCount,
  onSave,
  hasDbData,
  dbTotal,
  onDeleteRequest,
  lastSavedResult,
}: BuildActionRailPropsArgs): DataActionRailProps {
  const saveLabel = previewRowCount > 0
    ? `${previewRowCount}件 全件置換で保存`
    : '全件置換で保存';

  return {
    onFileSelect,
    selectedFileName: csvFileName ?? undefined,
    fileInputDisabled: loading || saving || deleting || previewing,
    hasCsvFile,
    saveLabel: saving ? '保存中...' : previewing ? '解析中...' : saveLabel,
    onSave,
    saveDisabled: saving || deleting || previewing || previewRowCount === 0,
    hasDbData,
    deleteLabel: deleting ? '削除中...' : `全件削除 (${dbTotal}件)`,
    onDeleteRequest,
    deleteDisabled: saving || deleting || loading,
    saveResult: lastSavedResult,
    saveModeLabel: '全件置換',
  };
}

interface BuildUtilityRailPropsArgs {
  actionRailProps: DataActionRailProps;
  error: string | null;
  dbWarning: string | null;
  assetBalanceData: AssetBalanceData[];
  searchCategories: SearchCategories;
  searchQuery: string;
  onSearch: (query: string) => void;
}

/** AssetBalanceUtilityRail に渡す props を組み立てる */
export function buildUtilityRailProps({
  actionRailProps,
  error,
  dbWarning,
  assetBalanceData,
  searchCategories,
  searchQuery,
  onSearch,
}: BuildUtilityRailPropsArgs): AssetBalanceUtilityRailProps {
  return {
    actionRailProps,
    error,
    warning: dbWarning,
    searchCardProps: {
      visible: assetBalanceData.length > 0,
      categories: searchCategories,
      value: searchQuery,
      onSearch,
    },
    reviewPromptCardProps: {
      assetBalanceData,
    },
  };
}

interface MainStatusFlags {
  loading: boolean;
  saving: boolean;
  deleting: boolean;
  previewing: boolean;
}

/** メイン領域に表示するステータスメッセージを状態フラグから決定する */
export function getMainStatusMessage(flags: MainStatusFlags): string | null {
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
