import React, { useMemo } from 'react';
import { Card, CardBody } from '@/components/atoms/Card';
import { StatItem } from '@/components/atoms/StatItem';
import { EmptyState } from '@/components/atoms/EmptyState';
import { PortfolioPieChart, PortfolioItem } from '@/components/molecules/PortfolioPieChart';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { formatCurrency, safeAdd } from '@/lib/utils/formatters';

interface AssetPortfolioSummaryProps {
  assetBalanceData: AssetBalanceData[];
}

/**
 * 保有銘柄のポートフォリオサマリーを表示するコンポーネント
 * 合計取得総額と構成比の円グラフを表示
 */
export const AssetPortfolioSummary: React.FC<AssetPortfolioSummaryProps> = ({
  assetBalanceData,
}) => {
  // 合計取得総額を計算（null/undefinedは0として扱う）
  const totalPurchaseAmount = useMemo(() => {
    return assetBalanceData.reduce(
      (sum, item) => safeAdd(sum, item.total_purchase_amount || 0),
      0
    );
  }, [assetBalanceData]);

  // 円グラフ用データを生成（詳細情報付き）
  const chartData: PortfolioItem[] = useMemo(() => {
    return assetBalanceData
      .filter((item) => (item.total_purchase_amount || 0) > 0)
      .map((item) => ({
        name: item.security_name || item.security_code,
        value: item.total_purchase_amount || 0,
        securityCode: item.security_code,
        shares: item.shares,
      }))
      .sort((a, b) => b.value - a.value);
  }, [assetBalanceData]);

  // データがない場合
  if (assetBalanceData.length === 0) {
    return (
      <Card className="mb-3">
        <CardBody>
          <EmptyState
            title="保有銘柄がありません"
            description="CSVファイルをインポートするか、データを登録してください。"
          />
        </CardBody>
      </Card>
    );
  }

  // 合計取得総額が0の場合
  if (totalPurchaseAmount === 0) {
    return null;
  }

  return (
    <div className="mb-3 space-y-3" data-testid="asset-portfolio-summary">
      {/* KPI: 合計取得総額（独立カード） */}
      <Card>
        <CardBody className="p-4">
          <StatItem
            title="合計取得総額"
            value={
              <span data-negative={totalPurchaseAmount < 0 ? 'true' : undefined}>
                {formatCurrency(totalPurchaseAmount)}
              </span>
            }
            variant="default"
            titleClassName="text-secondary"
            valueClassName="text-3xl font-bold text-primary"
          />
          <p className="mt-1 text-xs text-slate-500">
            {assetBalanceData.length}銘柄を保有
          </p>
        </CardBody>
      </Card>

      {/* 構成比エリア */}
      {chartData.length > 0 && (
        <Card>
          <CardBody className="p-4">
            <h3 className="mb-3 text-sm font-semibold text-slate-700">
              銘柄別構成比
            </h3>
            <PortfolioPieChart data={chartData} />
          </CardBody>
        </Card>
      )}
    </div>
  );
};
