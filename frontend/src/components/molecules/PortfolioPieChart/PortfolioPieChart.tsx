import React, { useMemo, useState } from 'react';
import { SecurityCodeLink } from '@/components/atoms/SecurityCodeLink';
import { formatCurrency, formatNumber } from '@/lib/utils/formatters';

// 横棒グラフ用の配色（視認性を考慮した10色）
const COLORS = [
  '#3b82f6', // blue-500
  '#10b981', // emerald-500
  '#f59e0b', // amber-500
  '#ef4444', // red-500
  '#8b5cf6', // violet-500
  '#ec4899', // pink-500
  '#06b6d4', // cyan-500
  '#f97316', // orange-500
  '#84cc16', // lime-500
  '#6366f1', // indigo-500
];

const TOP_N = 20; // デフォルト表示件数

export interface PortfolioItem {
  name: string;
  value: number;
  averagePrice: number;
  securityCode: string;
  shares: number;
}

interface ChartDataItem extends PortfolioItem {
  percentage: number;
}

interface PortfolioPieChartProps {
  data: PortfolioItem[];
  className?: string;
}

/**
 * ポートフォリオ構成比を表示する横棒グラフコンポーネント
 */
export const PortfolioPieChart: React.FC<PortfolioPieChartProps> = ({
  data,
  className = '',
}) => {
  const [showAll, setShowAll] = useState(false);

  // パーセンテージを計算してデータに追加（降順ソート済み）
  const chartData: ChartDataItem[] = useMemo(() => {
    const total = data.reduce((sum, item) => sum + item.value, 0);
    if (total === 0) return [];

    return data
      .map((item) => ({
        ...item,
        percentage: (item.value / total) * 100,
      }))
      .sort((a, b) => b.percentage - a.percentage);
  }, [data]);

  // 表示データ（Top N または全件）
  const displayData = useMemo(() => {
    if (showAll || chartData.length <= TOP_N) {
      return chartData;
    }
    return chartData.slice(0, TOP_N);
  }, [chartData, showAll]);

  // その他の合計（Top N以外）
  const othersPercentage = useMemo(() => {
    if (showAll || chartData.length <= TOP_N) return 0;
    return chartData.slice(TOP_N).reduce((sum, item) => sum + item.percentage, 0);
  }, [chartData, showAll]);

  if (chartData.length === 0) {
    return null;
  }

  const remainingCount = chartData.length - TOP_N;

  return (
    <div className={className} data-testid="portfolio-pie-chart">
      {/* 横棒グラフリスト（2-3列グリッド） */}
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {displayData.map((item, index) => (
          <div
            key={item.securityCode}
            className="rounded-lg border border-slate-200 bg-white p-3"
          >
            {/* 上段: 銘柄名 + 構成比 */}
            <div className="flex items-center justify-between gap-2 mb-2">
              <div className="flex items-center gap-2 min-w-0 flex-1">
                <span
                  className="h-3 w-3 shrink-0 rounded-sm"
                  style={{ backgroundColor: COLORS[index % COLORS.length] }}
                />
                <span className="truncate text-sm font-medium text-slate-700" title={item.name}>
                  {item.name}
                </span>
              </div>
              <span className="shrink-0 text-lg font-bold text-slate-800">
                {item.percentage.toFixed(1)}%
              </span>
            </div>
            {/* 横棒グラフ */}
            <div className="h-2.5 w-full rounded-full bg-slate-100 mb-2">
              <div
                className="h-full rounded-full transition-all duration-300"
                style={{
                  width: `${Math.min(item.percentage, 100)}%`,
                  backgroundColor: COLORS[index % COLORS.length],
                }}
              />
            </div>
            {/* 下段: 詳細情報 */}
            <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-slate-600">
              <div className="flex items-center gap-1">
                <span className="text-slate-500">コード:</span>
                <SecurityCodeLink value={item.securityCode} className="text-xs" />
              </div>
              <div className="flex items-center gap-1">
                <span className="text-slate-500">取得単価:</span>
                <span className="font-medium">{formatCurrency(item.averagePrice)}</span>
              </div>
              <div className="flex items-center gap-1">
                <span className="text-slate-500">数量:</span>
                <span className="font-medium">{`${formatNumber(item.shares)}株`}</span>
              </div>
              <div className="flex items-center gap-1">
                <span className="text-slate-500">取得総額:</span>
                <span className="font-medium">{formatCurrency(item.value)}</span>
              </div>
            </div>
          </div>
        ))}

        {/* その他（折りたたみ時） */}
        {!showAll && othersPercentage > 0 && (
          <div className="rounded-lg border border-dashed border-slate-300 bg-slate-50 p-3">
            <div className="flex items-center justify-between gap-2 mb-2">
              <span className="text-sm text-slate-500">
                その他 {chartData.length - TOP_N}銘柄
              </span>
              <span className="text-lg font-bold text-slate-500">
                {othersPercentage.toFixed(1)}%
              </span>
            </div>
            <div className="h-2.5 w-full rounded-full bg-slate-200">
              <div
                className="h-full rounded-full bg-slate-400 transition-all duration-300"
                style={{ width: `${Math.min(othersPercentage, 100)}%` }}
              />
            </div>
          </div>
        )}
      </div>

      {/* 全件表示トグル */}
      {chartData.length > TOP_N && (
        <button
          type="button"
          onClick={() => setShowAll(!showAll)}
          className="mt-3 w-full rounded-md border border-slate-300 bg-white py-2 text-sm font-medium text-slate-600 hover:bg-slate-50 transition-colors"
        >
          {showAll ? `上位${TOP_N}件のみ表示` : `残り${remainingCount}銘柄を表示（全${chartData.length}）`}
        </button>
      )}
    </div>
  );
};
