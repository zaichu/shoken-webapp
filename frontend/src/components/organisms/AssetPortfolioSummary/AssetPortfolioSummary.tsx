import React, { useMemo } from 'react';
import { Card, CardBody } from '@/components/atoms/Card';
import { EmptyState } from '@/components/atoms/EmptyState';
import { PortfolioPieChart, PortfolioItem } from '@/components/molecules/PortfolioPieChart';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { formatCurrency, safeAdd } from '@/lib/utils/formatters';

interface AssetPortfolioSummaryProps {
  assetBalanceData: AssetBalanceData[];
  totalCount?: number; // 全件数（絞り込み前）
  isFiltered?: boolean; // 絞り込み中かどうか
  onClearFilter?: () => void; // 絞り込み解除
}

/**
 * 保有銘柄のポートフォリオサマリーを表示するコンポーネント
 * 合計取得総額と構成比の横棒グラフを表示
 */
export const AssetPortfolioSummary: React.FC<AssetPortfolioSummaryProps> = ({
  assetBalanceData,
  totalCount,
  isFiltered = false,
  onClearFilter,
}) => {
  // 合計取得総額を計算（null/undefinedは0として扱う）
  const totalPurchaseAmount = useMemo(() => {
    return assetBalanceData.reduce(
      (sum, item) => safeAdd(sum, item.total_purchase_amount || 0),
      0
    );
  }, [assetBalanceData]);

  // 横棒グラフ用データを生成（詳細情報付き）
  const chartData: PortfolioItem[] = useMemo(() => {
    return assetBalanceData
      .filter((item) => (item.total_purchase_amount || 0) > 0)
      .map((item) => ({
        name: item.security_name || item.security_code,
        value: item.total_purchase_amount || 0,
        securityCode: item.security_code,
        shares: item.shares,
        averagePrice: item.average_purchase_price,
      }))
      .sort((a, b) => b.value - a.value);
  }, [assetBalanceData]);

  const displayCount = assetBalanceData.length;
  const actualTotalCount = totalCount ?? displayCount;

  // データがない場合
  if (assetBalanceData.length === 0) {
    return (
      <Card className="mb-3">
        <CardBody>
          <EmptyState
            title={isFiltered ? "該当する銘柄がありません" : "保有銘柄がありません"}
            description={isFiltered
              ? "検索条件を変更するか、絞り込みを解除してください。"
              : "CSVファイルをインポートするか、データを登録してください。"
            }
          />
          {isFiltered && onClearFilter && (
            <div className="mt-3 text-center">
              <button
                type="button"
                onClick={onClearFilter}
                className="text-sm text-primary hover:underline"
              >
                絞り込みを解除
              </button>
            </div>
          )}
        </CardBody>
      </Card>
    );
  }

  // 合計取得総額が0の場合
  if (totalPurchaseAmount === 0) {
    return null;
  }

  return (
    <div className="mb-3" data-testid="asset-portfolio-summary">
      <Card>
        <CardBody className="p-4">
          {/* セクションヘッダー: KPI + 銘柄数 + 絞り込み状態 */}
          <div className="mb-4 border-b border-slate-200 pb-4">
            {/* 絞り込み中バナー */}
            {isFiltered && (
              <div className="mb-4 pb-4 border-b border-slate-100">
                <div className="flex items-center justify-between rounded-md bg-blue-50 px-3 py-2">
                  <span className="text-sm font-medium text-blue-700">
                    🔍 絞り込み中: {displayCount}/{actualTotalCount}件を表示
                  </span>
                  {onClearFilter && (
                    <button
                      type="button"
                      onClick={onClearFilter}
                      className="text-sm text-blue-600 hover:text-blue-800 hover:underline"
                    >
                      解除
                    </button>
                  )}
                </div>
              </div>
            )}
            {/* KPI行 */}
            <div className="flex items-end gap-4">
              <div>
                <p className="text-sm text-slate-500">合計取得総額</p>
                <p className="text-3xl font-bold text-primary" data-negative={totalPurchaseAmount < 0 ? 'true' : undefined}>
                  {formatCurrency(totalPurchaseAmount)}
                </p>
              </div>
              <p className="pb-1 text-sm text-slate-500">
                {isFiltered
                  ? `${displayCount}銘柄（全${actualTotalCount}銘柄中）`
                  : `${displayCount}銘柄を保有`
                }
              </p>
            </div>
          </div>
          {/* 銘柄別構成比 */}
          <h3 className="mb-3 text-sm font-semibold text-slate-700">銘柄別構成比</h3>
          <PortfolioPieChart data={chartData} />
        </CardBody>
      </Card>
    </div>
  );
};
