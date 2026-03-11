import React, { useMemo, useState } from 'react';
import { SecurityCodeLink } from '@/components/atoms/SecurityCodeLink';
import { formatCurrency, formatNumber, formatPercentageValue } from '@/lib/utils/formatters';
import { DividendStatus } from '@/features/jquants/api/dividendPerShareApi';

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
  dividendPerShareMap?: Map<string, number>; // 銘柄別1株配当（J-Quants予想）
  dividendStatusMap?: Map<string, DividendStatus>; // 銘柄別取得ステータス
}

// --- 純粋関数 ---

interface DividendInfoResult {
  perShare: number | null;
  annual: number | null;
  yieldValue: number | null;
  status: DividendStatus | undefined;
}

function getDividendInfo(
  securityCode: string,
  shares: number,
  averagePrice: number,
  dividendPerShareMap: Map<string, number> | undefined,
  dividendStatusMap: Map<string, DividendStatus> | undefined,
): DividendInfoResult | null {
  if (!dividendPerShareMap) return null;
  const status = dividendStatusMap?.get(securityCode);
  const perShare = dividendPerShareMap.get(securityCode);

  if (status === 'pending') return { perShare: null, annual: null, yieldValue: null, status };
  if (status === 'error') return { perShare: null, annual: null, yieldValue: null, status };
  if (status === 'zero') return { perShare: 0, annual: 0, yieldValue: null, status };
  if (perShare === undefined) return { perShare: null, annual: null, yieldValue: null, status };

  const annual = perShare * shares;
  const yieldValue = averagePrice > 0 ? (perShare / averagePrice) * 100 : null;
  return { perShare, annual, yieldValue, status };
}

function formatPerShare(divInfo: DividendInfoResult | null): string {
  if (!divInfo) return '---';
  if (divInfo.status === 'pending') return '取得中...';
  if (divInfo.status === 'error') return '取得失敗';
  if (divInfo.perShare === null) return '---';
  return formatCurrency(divInfo.perShare);
}

function formatAnnual(divInfo: DividendInfoResult | null): string {
  if (!divInfo) return '---';
  if (divInfo.status === 'pending') return '取得中...';
  if (divInfo.status === 'error') return '取得失敗';
  if (divInfo.annual === null) return '---';
  return formatCurrency(divInfo.annual);
}

function formatYield(divInfo: DividendInfoResult | null): string {
  if (!divInfo) return '---';
  if (divInfo.status === 'pending') return '取得中...';
  if (divInfo.status === 'error') return '取得失敗';
  if (divInfo.yieldValue === null) return '---';
  return formatPercentageValue(divInfo.yieldValue);
}

// --- サブコンポーネント ---

interface PortfolioItemCardProps {
  item: ChartDataItem;
  index: number;
  dividendPerShareMap?: Map<string, number>;
  dividendStatusMap?: Map<string, DividendStatus>;
}

function PortfolioItemCard({ item, index, dividendPerShareMap, dividendStatusMap }: PortfolioItemCardProps) {
  const divInfo = getDividendInfo(item.securityCode, item.shares, item.averagePrice, dividendPerShareMap, dividendStatusMap);
  const color = COLORS[index % COLORS.length];

  return (
    <div className="rounded-[1.35rem] border border-slate-200/90 bg-white px-3.5 py-3 shadow-[0_18px_36px_-34px_rgba(15,23,42,0.38)]">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2.5 min-w-0">
            <span className="h-3 w-3 shrink-0 rounded-sm" style={{ backgroundColor: color }} />
            <div className="flex min-w-0 items-center gap-2" data-testid="portfolio-card-identity">
              <span
                className="inline-flex shrink-0 items-center rounded-full bg-slate-100 px-2 py-0.5"
                data-testid="portfolio-card-code"
              >
                <SecurityCodeLink value={item.securityCode} className="text-[11px] font-semibold tracking-[0.16em] text-slate-500 no-underline hover:underline" />
              </span>
              <p className="truncate text-[15px] font-semibold text-slate-800" title={item.name}>
                {item.name}
              </p>
            </div>
          </div>
        </div>
        <div className="shrink-0 text-right">
          <p className="text-xl font-bold text-slate-800">{item.percentage.toFixed(1)}%</p>
        </div>
      </div>

      <div className="mt-2.5 h-2 w-full rounded-full bg-slate-100">
        <div
          className="h-full rounded-full transition-all duration-300"
          style={{ width: `${Math.min(item.percentage, 100)}%`, backgroundColor: color }}
        />
      </div>

      <div
        className="mt-2.5 grid grid-cols-3 overflow-hidden rounded-[1rem] bg-slate-50"
        data-testid="portfolio-card-acquisition-stats"
      >
        <div className="min-w-0 px-3 py-2">
          <p className="truncate text-[10px] font-medium text-slate-500">取得総額</p>
          <p className="mt-0.5 truncate text-sm font-semibold text-slate-800">{formatCurrency(item.value)}</p>
        </div>
        <div className="min-w-0 border-l border-slate-200/80 px-3 py-2">
          <p className="truncate text-[10px] font-medium text-slate-500">取得単価</p>
          <p className="mt-0.5 truncate text-sm font-semibold text-slate-800">{formatCurrency(item.averagePrice)}</p>
        </div>
        <div className="min-w-0 border-l border-slate-200/80 px-3 py-2">
          <p className="truncate text-[10px] font-medium text-slate-500">数量</p>
          <p className="mt-0.5 truncate text-sm font-semibold text-slate-800">{`${formatNumber(item.shares)}株`}</p>
        </div>
      </div>

      <div className="mt-2.5 grid grid-cols-3 overflow-hidden rounded-[1rem] bg-emerald-50/55">
        <div className="min-w-0 px-3 py-2 text-xs text-slate-600">
          <p className="truncate text-[10px] font-medium text-slate-500">1株配当</p>
          <p className={`mt-0.5 truncate text-sm font-semibold ${divInfo?.perShare !== null && divInfo?.perShare !== undefined ? 'text-emerald-600' : 'text-slate-500'}`}>
            {formatPerShare(divInfo)}
          </p>
        </div>
        <div className="min-w-0 border-l border-emerald-100/80 px-3 py-2 text-xs text-slate-600">
          <p className="truncate text-[10px] font-medium text-slate-500">年間配当</p>
          <p className={`mt-0.5 truncate text-sm font-semibold ${divInfo?.annual !== null && divInfo?.annual !== undefined ? 'text-emerald-600' : 'text-slate-500'}`}>
            {formatAnnual(divInfo)}
          </p>
        </div>
        <div className="min-w-0 border-l border-emerald-100/80 px-3 py-2 text-xs text-slate-600">
          <p className="truncate text-[10px] font-medium text-slate-500">配当利回り</p>
          <p className={`mt-0.5 truncate text-sm font-semibold ${divInfo?.yieldValue !== null && divInfo?.yieldValue !== undefined ? 'text-emerald-600' : 'text-slate-500'}`}>
            {formatYield(divInfo)}
          </p>
        </div>
      </div>
    </div>
  );
}

// --- メインコンポーネント ---

/**
 * ポートフォリオ構成比を表示する横棒グラフコンポーネント
 */
export const PortfolioPieChart: React.FC<PortfolioPieChartProps> = ({
  data,
  className = '',
  dividendPerShareMap,
  dividendStatusMap,
}) => {
  const [showAll, setShowAll] = useState(false);

  // パーセンテージを計算してデータに追加（降順ソート済み）
  const chartData: ChartDataItem[] = useMemo(() => {
    const total = data.reduce((sum, item) => sum + item.value, 0);
    if (total === 0) return [];

    return data
      .map((item) => ({ ...item, percentage: (item.value / total) * 100 }))
      .sort((a, b) => b.percentage - a.percentage);
  }, [data]);

  // 表示データ（Top N または全件）
  const displayData = useMemo(() => {
    if (showAll || chartData.length <= TOP_N) return chartData;
    return chartData.slice(0, TOP_N);
  }, [chartData, showAll]);

  // その他の合計（Top N以外）
  const othersPercentage = useMemo(() => {
    if (showAll || chartData.length <= TOP_N) return 0;
    return chartData.slice(TOP_N).reduce((sum, item) => sum + item.percentage, 0);
  }, [chartData, showAll]);

  if (chartData.length === 0) return null;

  const remainingCount = chartData.length - TOP_N;
  const gridClassName = displayData.length <= 1
    ? 'grid grid-cols-1 gap-3'
    : displayData.length === 2 && !showAll
      ? 'grid grid-cols-1 gap-3 xl:grid-cols-2'
      : 'grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3';

  return (
    <div className={className} data-testid="portfolio-pie-chart">
      {/* 横棒グラフリスト（2-3列グリッド） */}
      <div className={gridClassName}>
        {displayData.map((item, index) => (
          <PortfolioItemCard
            key={item.securityCode}
            item={item}
            index={index}
            dividendPerShareMap={dividendPerShareMap}
            dividendStatusMap={dividendStatusMap}
          />
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
