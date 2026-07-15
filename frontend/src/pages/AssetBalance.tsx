import { Suspense, lazy } from 'react';
import { Layout } from '../components/templates/Layout';
import { WorkspaceShell } from '@/components/templates/WorkspaceShell';
import { PageHeader } from '../components/atoms/PageHeader';
import { Spinner } from '@/components/atoms/Spinner';
import type { AssetBalanceData } from '@/types/api';
import type { AssetBalanceSummary } from '@/features/assetBalance/hooks/useAssetBalanceDataSource';
import { DividendStatus } from '@/features/dividendPerShare/api/dividendPerShareApi';
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
  portfolioSummary?: AssetBalanceSummary;
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
  portfolioSummary,
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
        summary={portfolioSummary}
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
    assetBalanceData,
    filteredData,
    portfolioSummary,
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
        <WorkspaceShell
          testIdPrefix="assetbalance"
          main={
            <>
              {mainStatusMessage && (
                <div
                  className="status-message"
                  role="status"
                  aria-live="polite"
                  aria-atomic="true"
                >
                  <Spinner size="md" className="text-primary" aria-hidden="true" />
                  <p className="text-sm text-secondary">{mainStatusMessage}</p>
                </div>
              )}

              {/* ローディング完了後に表示（空データでもEmptyStateを表示） */}
              {showPortfolioSummary && (
                <AssetBalanceInfo
                  assetBalanceData={assetBalanceData}
                  filteredData={filteredData}
                  searchQuery={utilityRailProps.searchCardProps.value}
                  onClearFilter={clearSearch}
                  dividendPerShareMap={dividendPerShareMap}
                  dividendStatusMap={dividendStatusMap}
                  portfolioSummary={portfolioSummary}
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
      </div>
    </Layout>
  );
}
