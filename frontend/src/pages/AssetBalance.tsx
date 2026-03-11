import React, { useState, useMemo, useCallback, Suspense, lazy } from 'react';
import { Layout } from '../components/templates/Layout';
import { PageHeader } from '../components/atoms/PageHeader';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { Alert } from '@/components/atoms/Alert';
import { Button } from '@/components/atoms/Button';
import { Spinner } from '@/components/atoms/Spinner';
import { SearchCard } from '@/components/organisms/SearchCard/SearchCard';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { useDividendBatch } from '@/features/jquants/hooks/useDividendBatch';
import { DividendStatus } from '@/features/jquants/api/dividendPerShareApi';
import { useAssetBalanceDataSource } from '@/features/assetBalance/hooks/useAssetBalanceDataSource';
import { createSearchOptions } from '@/lib/utils/dataTransformer';
import { filterByConfig, FilterConfig } from '@/lib/utils/searchUtils';
import { ConfirmDeleteModal } from '@/components/molecules/ConfirmDeleteModal/ConfirmDeleteModal';

// rechartsを含むコンポーネントを遅延読み込み（バンドルサイズ最適化）
const AssetPortfolioSummary = lazy(() =>
  import('@/components/organisms/AssetPortfolioSummary').then(module => ({
    default: module.AssetPortfolioSummary
  }))
);


interface AssetBalanceInfoProps {
  assetBalanceData: AssetBalanceData[];
  filteredData: AssetBalanceData[];
  searchQuery: string;
  onClearFilter: () => void;
  dividendPerShareMap: Map<string, number>;
  dividendStatusMap?: Map<string, DividendStatus>;
}

/**
 * 保有銘柄データを表示するコンポーネント（概要重視）
 */
export const AssetBalanceInfo: React.FC<AssetBalanceInfoProps> = ({
  assetBalanceData,
  filteredData,
  searchQuery,
  onClearFilter,
  dividendPerShareMap,
  dividendStatusMap,
}) => {
  const isFiltered = searchQuery !== '';

  return (
    <Suspense fallback={<div className="h-64 flex items-center justify-center"><Spinner size="md" /></div>}>
      <AssetPortfolioSummary
        assetBalanceData={filteredData}
        totalCount={assetBalanceData.length}
        isFiltered={isFiltered}
        onClearFilter={isFiltered ? onClearFilter : undefined}
        dividendPerShareMap={dividendPerShareMap}
        dividendStatusMap={dividendStatusMap}
      />
    </Suspense>
  );
};

/**
 * 保有銘柄管理ページコンポーネント
 */
export function AssetBalancePage() {
  const { isAuthenticated, isLoading: authLoading, login } = useAuth();

  const {
    dbData,
    previewRows,
    loading,
    error,
    saving,
    deleting,
    previewing,
    lastSavedCount,
    hasCsvFile,
    hasDbData,
    csvFileName,
    handleFileSelect,
    handleSaveToDB,
    handleDeleteAll,
  } = useAssetBalanceDataSource();

  const [searchQuery, setSearchQuery] = useState('');
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const handleConfirmDelete = useCallback(async () => {
    setShowDeleteConfirm(false);
    await handleDeleteAll();
  }, [handleDeleteAll]);

  // CSVプレビュー行 > DBデータ の優先順位でテーブルデータを決定
  const assetBalanceData = useMemo(() => {
    if (previewRows.length > 0) return previewRows;
    if (dbData.length > 0) return dbData;
    return [];
  }, [previewRows, dbData]);

  // J-Quants APIから1株配当を一括取得
  const securityCodes = useMemo(
    () => (saving || loading) ? [] : assetBalanceData.map(item => item.security_code),
    [saving, loading, assetBalanceData]
  );
  const { dividendPerShareMap, dividendStatusMap } = useDividendBatch(securityCodes, isAuthenticated);

  const searchCategories = useMemo(() => ({
    securities: createSearchOptions(assetBalanceData, 'security_code', 'security_name', true)
  }), [assetBalanceData]);

  const filterConfig: FilterConfig<AssetBalanceData> = useMemo(() => ({
    partialStringFields: [
      item => item.security_code,
      item => item.security_name,
    ],
  }), []);

  const filteredData = useMemo(
    () => filterByConfig(assetBalanceData, searchQuery, filterConfig),
    [assetBalanceData, searchQuery, filterConfig]
  );

  const isProcessing = loading || saving || deleting || previewing || authLoading;

  const saveLabel = previewRows.length > 0
    ? `${previewRows.length}件 全件置換で保存`
    : '全件置換で保存';

  return (
    <Layout>
      <PageHeader
        title="資産管理"
        description="保有している銘柄の一覧と評価額を確認できます。"
      />
      <div className="mt-2" aria-busy={isProcessing}>
        {/* 認証確認中 */}
        {authLoading && (
          <div className="status-message" role="status" aria-live="polite">
            <Spinner size="md" className="text-primary" />
            <p className="text-sm text-secondary">認証状態を確認しています...</p>
          </div>
        )}

        {/* 未ログイン時のログイン誘導 */}
        {!authLoading && !isAuthenticated && (
          <Alert variant="info" className="my-3" role="status" aria-live="polite">
            <p className="mb-2 text-sm">資産管理データを管理するにはログインが必要です。</p>
            <Button
              variant="primary"
              size="sm"
              onClick={() => login()}
              aria-label="Googleアカウントでログイン"
            >
              ログイン
            </Button>
          </Alert>
        )}

        {/* ログイン済みの場合のメインコンテンツ */}
        {!authLoading && isAuthenticated && (
          <>
            {/* デスクトップ: aside（CSV操作・検索）左 + main（ポートフォリオ）右の2カラム */}
            <div className="flex flex-col lg:flex-row lg:gap-4 lg:items-start">
              {/* aside: CSV操作・検索 — モバイルでは先頭、デスクトップでは左カラム */}
              <div className="shrink-0 space-y-3 lg:w-60">
                <div className="rounded-lg border border-slate-200 bg-white p-3 shadow-sm" role="group" aria-label="データ操作">
                  <p className="mb-2.5 flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-slate-500">
                    <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                    </svg>
                    CSV操作
                  </p>
                  <div className="space-y-2">
                    <CSVFileInput
                      onFileSelect={handleFileSelect}
                      selectedFileName={csvFileName ?? ''}
                      disabled={loading || saving || deleting || previewing}
                    />
                    {hasCsvFile && (
                      <Button
                        variant="primary"
                        size="sm"
                        className="w-full"
                        onClick={handleSaveToDB}
                        disabled={saving || deleting || previewing || previewRows.length === 0}
                        aria-disabled={saving || deleting || previewing || previewRows.length === 0}
                      >
                        {saving ? '保存中...' : previewing ? '解析中...' : saveLabel}
                      </Button>
                    )}
                    {hasDbData && (
                      <Button
                        variant="outline-danger"
                        size="sm"
                        className="w-full"
                        onClick={() => setShowDeleteConfirm(true)}
                        disabled={saving || deleting || loading}
                        aria-disabled={saving || deleting || loading}
                      >
                        {deleting ? '削除中...' : `全件削除 (${dbData.length}件)`}
                      </Button>
                    )}
                  </div>
                </div>

                {error && (
                  <Alert variant="danger" role="alert" aria-live="assertive">
                    <strong>エラー:</strong> {error}
                  </Alert>
                )}

                {/* 検索カード（データがある場合のみ表示） */}
                {assetBalanceData.length > 0 && (
                  <SearchCard
                    onSearch={query => setSearchQuery(query)}
                    categories={searchCategories}
                    value={searchQuery}
                    compact
                  />
                )}
              </div>

              {/* main: ポートフォリオサマリー — モバイルでは2番目、デスクトップでは右カラム */}
              <div className="flex-1 min-w-0">
                {lastSavedCount !== null && (
                  <div className="mb-2" role="status" aria-live="polite">
                    <Alert variant="success">
                      <strong>{lastSavedCount}件保存しました</strong>
                      <span className="ml-2 text-sm text-secondary">（全件置換）</span>
                    </Alert>
                  </div>
                )}

                <div aria-live="polite" aria-atomic="true">
                  {(loading || saving || deleting || previewing) && (
                    <div className="status-message" role="status">
                      <Spinner size="md" className="text-primary" />
                      <p className="text-sm text-secondary">
                        {loading && 'データを読み込んでいます...'}
                        {saving && 'データを保存しています...'}
                        {deleting && 'データを削除しています...'}
                        {previewing && 'CSVファイルを解析しています...'}
                      </p>
                    </div>
                  )}
                </div>

                {/* ローディング完了後に表示（空データでもEmptyStateを表示） */}
                {!loading && !previewing && (
                  <AssetBalanceInfo
                    assetBalanceData={assetBalanceData}
                    filteredData={filteredData}
                    searchQuery={searchQuery}
                    onClearFilter={() => setSearchQuery('')}
                    dividendPerShareMap={dividendPerShareMap}
                    dividendStatusMap={dividendStatusMap}
                  />
                )}
              </div>
            </div>

            <ConfirmDeleteModal
              isOpen={showDeleteConfirm}
              onConfirm={handleConfirmDelete}
              onCancel={() => setShowDeleteConfirm(false)}
              title="資産管理データの全件削除"
              description="保存された資産管理データをすべて削除します。"
              itemCount={dbData.length}
              loading={deleting}
            />
          </>
        )}
      </div>
    </Layout>
  );
}
