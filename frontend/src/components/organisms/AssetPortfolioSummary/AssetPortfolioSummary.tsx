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

  // 円グラフ用データを生成
  const chartData: PortfolioItem[] = useMemo(() => {
    return assetBalanceData
      .filter((item) => (item.total_purchase_amount || 0) > 0)
      .map((item) => ({
        name: item.security_name || item.security_code,
        value: item.total_purchase_amount || 0,
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
    <Card className="mb-3" data-testid="asset-portfolio-summary">
      <CardBody className="p-4">
        <div className="flex flex-col gap-4 md:flex-row md:items-start md:gap-8">
          {/* 合計取得総額 */}
          <div className="shrink-0">
            <StatItem
              title="合計取得総額"
              value={
                <span
                  dangerouslySetInnerHTML={{
                    __html: formatCurrency(totalPurchaseAmount),
                  }}
                />
              }
              variant="default"
              titleClassName="text-secondary"
              valueClassName="text-2xl text-primary"
            />
          </div>

          {/* 構成比円グラフ */}
          {chartData.length > 0 && (
            <div className="min-w-0 flex-1">
              <h3 className="mb-2 text-sm font-semibold text-secondary">
                銘柄別構成比
              </h3>
              <PortfolioPieChart data={chartData} />
            </div>
          )}
        </div>
      </CardBody>
    </Card>
  );
};
