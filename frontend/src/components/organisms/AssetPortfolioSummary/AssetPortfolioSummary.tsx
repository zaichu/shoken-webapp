import React, { useMemo } from 'react';
import { Card, CardBody } from '@/components/atoms/Card';
import { EmptyState } from '@/components/atoms/EmptyState';
import { PortfolioPieChart, PortfolioItem } from '@/components/molecules/PortfolioPieChart';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import {
  formatCurrency,
  formatPercentageValue,
  normalizeSecurityName,
  safeAdd,
} from '@/lib/utils/formatters';
import { DividendStatus } from '@/features/jquants/api/dividendPerShareApi';

interface AssetPortfolioSummaryProps {
  assetBalanceData: AssetBalanceData[];
  totalCount?: number; // 全件数（絞り込み前）
  isFiltered?: boolean; // 絞り込み中かどうか
  onClearFilter?: () => void; // 絞り込み解除
  dividendPerShareMap?: Map<string, number>; // 銘柄別1株配当（J-Quants予想）
  dividendStatusMap?: Map<string, DividendStatus>; // 銘柄別取得ステータス
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
  dividendPerShareMap,
  dividendStatusMap,
}) => {
  // 合計取得総額を計算（null/undefinedは0として扱う）
  const totalPurchaseAmount = useMemo(() => {
    return assetBalanceData.reduce(
      (sum, item) => safeAdd(sum, item.total_purchase_amount || 0),
      0
    );
  }, [assetBalanceData]);

  // ポートフォリオ全体の年間配当金額と配当利回り（%）
  // 年間配当 = Σ(1株配当 × 保有株数)、配当利回り = 年間配当 / 取得総額 × 100
  const { totalAnnualDividends, portfolioDividendYield } = useMemo(() => {
    if (!dividendPerShareMap || dividendPerShareMap.size === 0) {
      return { totalAnnualDividends: null, portfolioDividendYield: null };
    }
    let total = 0;
    assetBalanceData.forEach(item => {
      const perShare = dividendPerShareMap.get(item.security_code);
      if (perShare !== undefined) {
        total += perShare * (item.shares || 0);
      }
    });
    if (total === 0) return { totalAnnualDividends: null, portfolioDividendYield: null };
    const yieldValue = totalPurchaseAmount > 0 ? (total / totalPurchaseAmount) * 100 : null;
    return { totalAnnualDividends: total, portfolioDividendYield: yieldValue };
  }, [dividendPerShareMap, assetBalanceData, totalPurchaseAmount]);

  // 横棒グラフ用データを生成（詳細情報付き）
  const chartData: PortfolioItem[] = useMemo(() => {
    return assetBalanceData
      .filter((item) => (item.total_purchase_amount || 0) > 0)
      .map((item) => ({
        name: normalizeSecurityName(item.security_name || item.security_code),
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
            title={isFiltered ? "該当する銘柄がありません" : "資産管理データがありません"}
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
    <div className="mb-3 space-y-4" data-testid="asset-portfolio-summary">
      <section
        className="rounded-[2rem] border border-slate-200/90 bg-white/85 px-5 py-5 shadow-[0_22px_48px_-36px_rgba(15,23,42,0.45)]"
        data-testid="portfolio-kpi-strip"
      >
        <div className="flex flex-col gap-3 border-b border-slate-200/80 pb-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <h2 className="text-sm font-semibold text-slate-800">資産サマリー</h2>
          </div>
          {isFiltered && (
            <div className="flex flex-wrap items-center gap-2">
              <span className="inline-flex items-center rounded-full bg-blue-50 px-3 py-1 text-sm font-semibold text-blue-700">
                絞り込み中: {displayCount}/{actualTotalCount}件
              </span>
              {onClearFilter && (
                <button
                  type="button"
                  onClick={onClearFilter}
                  className="inline-flex items-center rounded-full border border-slate-200 bg-white px-3 py-1 text-sm font-medium text-slate-600 transition-colors hover:bg-slate-50"
                >
                  解除
                </button>
              )}
            </div>
          )}
        </div>
        <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <div className="rounded-[1.35rem] border border-slate-200/90 bg-white px-4 py-4 shadow-[0_12px_24px_-28px_rgba(15,23,42,0.4)]">
            <p className="mb-1 text-xs font-medium text-slate-600">合計取得総額</p>
            <p className="text-3xl font-bold text-primary tabular-nums" data-negative={totalPurchaseAmount < 0 ? 'true' : undefined}>
              {formatCurrency(totalPurchaseAmount)}
            </p>
          </div>
          <div className="rounded-[1.35rem] border border-emerald-100 bg-emerald-50/90 px-4 py-4 shadow-[0_12px_24px_-28px_rgba(5,150,105,0.35)]">
            <p className="mb-1 text-xs font-medium text-slate-600">年間配当金額</p>
            <p className="text-3xl font-bold text-emerald-600 tabular-nums" data-testid="portfolio-annual-dividends">
              {totalAnnualDividends !== null ? formatCurrency(totalAnnualDividends) : '---'}
            </p>
          </div>
          <div className="rounded-[1.35rem] border border-emerald-100 bg-emerald-50/90 px-4 py-4 shadow-[0_12px_24px_-28px_rgba(5,150,105,0.35)]">
            <p className="mb-1 text-xs font-medium text-slate-600">配当利回り</p>
            <p className="text-3xl font-bold text-emerald-600 tabular-nums" data-testid="portfolio-dividend-yield">
              {portfolioDividendYield !== null ? formatPercentageValue(portfolioDividendYield) : '---'}
            </p>
          </div>
          <div className="rounded-[1.35rem] border border-slate-200/90 bg-white px-4 py-4 shadow-[0_12px_24px_-28px_rgba(15,23,42,0.4)]">
            <p className="mb-1 text-xs font-medium text-slate-600">保有銘柄数</p>
            <p className="text-3xl font-bold text-slate-700 tabular-nums">
              {isFiltered
                ? `${displayCount} / ${actualTotalCount}`
                : `${displayCount}`
              }
              <span className="ml-1 text-sm font-normal text-slate-500">銘柄</span>
            </p>
          </div>
        </div>
      </section>

      <Card className="overflow-hidden rounded-[1.75rem] border-slate-200 bg-white/95 shadow-sm">
        <CardBody className="p-0">
          <div className="flex flex-col gap-3 border-b border-slate-200 px-5 py-4 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <h3 className="text-sm font-semibold text-slate-700">銘柄別構成比</h3>
            </div>
          </div>
          <div className="p-4">
            <PortfolioPieChart data={chartData} dividendPerShareMap={dividendPerShareMap} dividendStatusMap={dividendStatusMap} />
          </div>
        </CardBody>
      </Card>
    </div>
  );
};
