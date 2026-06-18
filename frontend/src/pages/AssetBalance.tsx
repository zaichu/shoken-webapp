import { Suspense, lazy } from 'react';
import { Layout } from '../components/templates/Layout';
import { WorkspaceShell } from '@/components/templates/WorkspaceShell';
import { PageHeader } from '../components/atoms/PageHeader';
import { Alert } from '@/components/atoms/Alert';
import { Button } from '@/components/atoms/Button';
import { Spinner } from '@/components/atoms/Spinner';
import type { AssetBalanceData } from '@/types/api';
import { DividendStatus } from '@/features/jquants/api/dividendPerShareApi';
import { AssetBalanceUtilityRail } from '@/features/assetBalance/components/AssetBalanceUtilityRail';
import { useAssetBalanceState } from '@/features/assetBalance/hooks/useAssetBalanceState';
import { usePageTitle } from '../hooks/usePageTitle';
import { ConfirmDeleteModal } from '@/components/molecules/ConfirmDeleteModal/ConfirmDeleteModal';

// PortfolioPieChartコンポーネントを遅延読み込み（バンドルサイズ最適化）
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
export function AssetBalanceInfo({
  assetBalanceData,
  filteredData,
  searchQuery,
  onClearFilter,
  dividendPerShareMap,
  dividendStatusMap,
}: AssetBalanceInfoProps) {
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
}

/**
 * 保有銘柄管理ページコンポーネント
 */
export function AssetBalancePage() {
  usePageTitle('資産管理');

  const {
    isAuthenticated,
    authLoading,
    login,
    assetBalanceData,
    filteredData,
    clearSearch,
    utilityRailProps,
    showDeleteConfirm,
    closeDeleteConfirm,
    confirmDeleteAll,
    dbDataCount,
    deleteModalLoading,
    showPortfolioSummary,
    mainStatusMessage,
    workspaceBusy,
    dividendPerShareMap,
    dividendStatusMap,
  } = useAssetBalanceState();

  return (
    <Layout>
      <PageHeader
        title="資産管理"
        eyebrow="Portfolio"
        description="保有している銘柄の一覧と評価額を確認できます。"
      />
      <div className="mt-2" aria-busy={workspaceBusy}>
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
            <WorkspaceShell
              testIdPrefix="assetbalance"
              main={
                <>
                  <div aria-live="polite" aria-atomic="true">
                    {mainStatusMessage && (
                      <div className="status-message" role="status">
                        <Spinner size="md" className="text-primary" />
                        <p className="text-sm text-secondary">{mainStatusMessage}</p>
                      </div>
                    )}
                  </div>

                  {/* ローディング完了後に表示（空データでもEmptyStateを表示） */}
                  {showPortfolioSummary && (
                    <AssetBalanceInfo
                      assetBalanceData={assetBalanceData}
                      filteredData={filteredData}
                      searchQuery={utilityRailProps.searchCardProps.value}
                      onClearFilter={clearSearch}
                      dividendPerShareMap={dividendPerShareMap}
                      dividendStatusMap={dividendStatusMap}
                    />
                  )}
                </>
              }
              rail={<AssetBalanceUtilityRail {...utilityRailProps} />}
            />

            <ConfirmDeleteModal
              isOpen={showDeleteConfirm}
              onConfirm={confirmDeleteAll}
              onCancel={closeDeleteConfirm}
              title="資産管理データの全件削除"
              description="保存された資産管理データをすべて削除します。"
              itemCount={dbDataCount}
              loading={deleteModalLoading}
            />
          </>
        )}
      </div>
    </Layout>
  );
}
