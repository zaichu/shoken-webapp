import React, { useMemo } from 'react';
import { Card, CardBody } from '@/components/atoms/Card';
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
    <div className="mb-3" data-testid="asset-portfolio-summary">
      {/* 構成比セクション（KPIをヘッダーに統合） */}
      {chartData.length > 0 && (
        <Card>
          <CardBody className="p-4">
            {/* セクションヘッダー: KPI + 補助情報 */}
            <div className="mb-4 border-b border-slate-200 pb-4">
              <div className="flex flex-wrap items-baseline justify-between gap-2">
                <div>
                  <p className="text-sm text-slate-500">合計取得総額</p>
                  <p className="text-3xl font-bold text-primary" data-negative={totalPurchaseAmount < 0 ? 'true' : undefined}>
                    {formatCurrency(totalPurchaseAmount)}
                  </p>
                </div>
                <p className="text-sm text-slate-500">
                  {assetBalanceData.length}銘柄を保有
                </p>
              </div>
            </div>
            {/* ドーナツ + 銘柄カード */}
            <PortfolioPieChart data={chartData} />
          </CardBody>
        </Card>
      )}
    </div>
  );
};
