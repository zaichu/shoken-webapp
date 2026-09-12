import React, { useState } from 'react';
import { SecurityCodeLink } from '@/components/atoms/SecurityCodeLink';
import { formatCurrency, formatNumber, formatPercentageValue } from '@/lib/utils/formatters';
import { DividendStatus } from '@/features/dividendPerShare/api/dividendPerShareApi';
import {
  calculateValuation,
  formatValuationAmount,
  formatValuationRate,
  toFiniteAmount,
} from '@/features/assetBalance/valuation';
import { cn } from '@/lib/utils/classNames';

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
  /** 展開内に表示する銘柄名全文 (未指定時は name を使う) */
  fullName?: string;
  /** 評価額。欠損時は null/undefined のまま渡す (0円として扱わない) */
  marketValue?: number | null;
  /** 現在値 (取込値)。欠損時は null/undefined のまま渡す */
  currentPrice?: number | null;
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

interface PortfolioValuationCardProps {
  item: ChartDataItem;
  dividendPerShareMap?: Map<string, number>;
  dividendStatusMap?: Map<string, DividendStatus>;
}

/**
 * スマホ幅 (640px未満) 専用の評価額カード。
 * 閉じた状態は銘柄名・評価額・評価損益のみ。詳細はタップ展開で表示する。
 * 各カードが独立した開閉状態を持ち、複数同時展開できる。
 * PC幅では表示しない (sm:hidden)。PC表示は PortfolioItemCard が担う。
 */
function PortfolioValuationCard({ item, dividendPerShareMap, dividendStatusMap }: PortfolioValuationCardProps) {
  const [open, setOpen] = useState(false);
  const detailId = `portfolio-item-detail-${item.securityCode}`;
  const divInfo = getDividendInfo(item.securityCode, item.shares, item.averagePrice, dividendPerShareMap, dividendStatusMap);
  // 損益率はDB値を使わず valuation.ts で再計算する (Issue #851 改訂1)
  const valuation = calculateValuation(item.marketValue, item.value);
  const fullName = item.fullName || item.name;

  // 評価額そのものの表示。欠損は0円とせず「—」にする (Issue #851 改訂2)
  const marketDisplay = (() => {
    const market = toFiniteAmount(item.marketValue);
    return market === null ? '—' : formatCurrency(market);
  })();
  const currentPriceDisplay = (() => {
    const current = toFiniteAmount(item.currentPrice);
    return current === null ? '—' : formatCurrency(current);
  })();

  return (
    <div className="rounded-lg border border-slate-950/10 bg-white shadow-sm sm:hidden" data-testid="portfolio-valuation-card">
      <button
        type="button"
        aria-expanded={open}
        aria-controls={detailId}
        onClick={() => setOpen((prev) => !prev)}
        className="block min-h-[44px] w-full px-3.5 py-4 text-left"
      >
        <span className="flex min-w-0 items-center gap-2">
          <span className="min-w-0 flex-1 truncate text-[15px] font-semibold text-slate-800">
            {item.name}
          </span>
          <span
            className="inline-flex shrink-0 items-center rounded-full bg-blue-50 px-2 py-0.5 text-[11px] font-semibold tracking-[0.16em] text-blue-700"
            data-testid="portfolio-valuation-card-code"
          >
            {item.securityCode}
          </span>
        </span>
        <span className="mt-2 flex items-baseline justify-between gap-2">
          <span className="shrink-0 text-xs font-medium text-slate-500">評価額</span>
          <span className="truncate text-base font-bold tabular-nums text-slate-800">
            {marketDisplay}
          </span>
        </span>
        <span className="mt-2 flex items-center justify-between gap-2">
          <span className="shrink-0 text-xs font-medium text-slate-500">評価損益</span>
          <span className="flex min-w-0 items-center gap-1">
            <span className="truncate text-sm font-bold tabular-nums text-slate-800">
              {valuation.amount === null || valuation.rate === null
                ? valuation.amount === null
                  ? '—'
                  : `${formatValuationAmount(valuation.amount, formatCurrency)}（算出不可）`
                : `${formatValuationAmount(valuation.amount, formatCurrency)}（${formatValuationRate(valuation.rate)}）`}
            </span>
            <span aria-hidden="true" className="shrink-0 text-xs text-slate-400">
              {open ? '▴' : '▾'}
            </span>
          </span>
        </span>
      </button>
      {open && (
        <div id={detailId} className="border-t border-slate-950/10 px-3.5 py-3">
          <dl className="space-y-1.5 text-xs text-slate-600">
            <div className="flex items-start justify-between gap-2">
              <dt className="shrink-0 font-medium text-slate-500">銘柄名</dt>
              <dd className="min-w-0 break-words text-right font-semibold text-slate-800">{fullName}</dd>
            </div>
            <div className="flex items-center justify-between gap-2">
              <dt className="shrink-0 font-medium text-slate-500">取得総額</dt>
              <dd className="truncate font-semibold tabular-nums text-slate-800">{formatCurrency(item.value)}</dd>
            </div>
            <div className="flex items-center justify-between gap-2">
              <dt className="shrink-0 font-medium text-slate-500">取得単価</dt>
              <dd className="truncate font-semibold tabular-nums text-slate-800">{formatCurrency(item.averagePrice)}</dd>
            </div>
            <div className="flex items-center justify-between gap-2">
              <dt className="shrink-0 font-medium text-slate-500">数量</dt>
              <dd className="truncate font-semibold tabular-nums text-slate-800">{`${formatNumber(item.shares)}株`}</dd>
            </div>
            <div className="flex items-center justify-between gap-2">
              <dt className="shrink-0 font-medium text-slate-500">現在値</dt>
              <dd className="truncate font-semibold tabular-nums text-slate-800">{currentPriceDisplay}</dd>
            </div>
            <div className="flex items-center justify-between gap-2">
              <dt className="shrink-0 font-medium text-slate-500">取得額構成比</dt>
              <dd className="truncate font-semibold tabular-nums text-slate-800">{Number.isNaN(item.percentage) ? '—' : formatPercentageValue(item.percentage, 1)}</dd>
            </div>
            <div className="flex items-center justify-between gap-2">
              <dt className="shrink-0 font-medium text-slate-500">予想年間配当</dt>
              <dd className="truncate font-semibold text-emerald-600">{formatAnnual(divInfo)}</dd>
            </div>
            <div className="flex items-center justify-between gap-2">
              <dt className="shrink-0 font-medium text-slate-500">1株配当</dt>
              <dd className="truncate font-semibold text-emerald-600">{formatPerShare(divInfo)}</dd>
            </div>
            <div className="flex items-center justify-between gap-2">
              <dt className="shrink-0 font-medium text-slate-500">取得額基準利回り</dt>
              <dd className="truncate font-semibold text-emerald-600">{formatYield(divInfo)}</dd>
            </div>
          </dl>
          <p className="mt-2.5 border-t border-slate-100 pt-2.5 text-xs">
            <SecurityCodeLink value={item.securityCode} className="text-xs" />
            <span className="ml-1 text-slate-500">の銘柄情報を見る</span>
          </p>
        </div>
      )}
    </div>
  );
}

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
    // PC幅の表示は従来どおり。スマホ幅では PortfolioValuationCard を使う
    <div className="rounded-lg border border-slate-950/10 bg-white px-3.5 py-3 shadow-sm max-sm:hidden">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2.5 min-w-0">
            <span className="h-3 w-3 shrink-0 rounded-sm" style={{ backgroundColor: color }} />
            <div className="flex min-w-0 items-center gap-2" data-testid="portfolio-card-identity">
              <span
                className="inline-flex shrink-0 items-center rounded-full bg-blue-50 px-2 py-0.5"
                data-testid="portfolio-card-code"
              >
                <SecurityCodeLink value={item.securityCode} className="text-[11px] font-semibold tracking-[0.16em] no-underline hover:underline" />
              </span>
              <p className="truncate text-[15px] font-semibold text-slate-800" title={item.name}>
                {item.name}
              </p>
            </div>
          </div>
        </div>
        <div className="shrink-0 text-right">
          <p className="text-xl font-bold text-slate-800">{formatPercentageValue(item.percentage, 1)}</p>
        </div>
      </div>

      {/* パーセントを数値でも出しているためモバイルでは冗長。縦の圧縮を優先して隠す */}
      <div className="mt-2.5 h-2 w-full rounded-full bg-slate-100 max-sm:hidden">
        <div
          className="h-full rounded-full transition-all duration-300"
          style={{ width: `${Math.min(item.percentage, 100)}%`, backgroundColor: color }}
        />
      </div>

      <div
        className="mt-2 grid grid-cols-3 overflow-hidden rounded-md bg-slate-50"
        data-testid="portfolio-card-acquisition-stats"
      >
        <div className="min-w-0 px-2 py-2 max-sm:px-1.5">
          <p className="truncate text-[10px] font-medium text-slate-500">取得総額</p>
          <p className="mt-0.5 truncate text-[12px] font-semibold text-slate-800 max-sm:text-[11px] max-sm:tracking-tight" title={formatCurrency(item.value)}>{formatCurrency(item.value)}</p>
        </div>
        <div className="min-w-0 border-l border-slate-200/80 px-2 py-2 max-sm:px-1.5">
          <p className="truncate text-[10px] font-medium text-slate-500">取得単価</p>
          <p className="mt-0.5 truncate text-[12px] font-semibold text-slate-800" title={formatCurrency(item.averagePrice)}>{formatCurrency(item.averagePrice)}</p>
        </div>
        <div className="min-w-0 border-l border-slate-200/80 px-2 py-2 max-sm:px-1.5">
          <p className="truncate text-[10px] font-medium text-slate-500">数量</p>
          <p className="mt-0.5 truncate text-[12px] font-semibold text-slate-800" title={`${formatNumber(item.shares)}株`}>{`${formatNumber(item.shares)}株`}</p>
        </div>
      </div>

      <div className="mt-2 grid grid-cols-3 overflow-hidden rounded-md bg-emerald-50/55">
        <div className="min-w-0 px-2 py-2 text-xs text-slate-600">
          <p className="truncate text-[10px] font-medium text-slate-500">1株配当</p>
          <p
            className={cn('mt-0.5 truncate text-[12px] font-semibold', divInfo?.perShare !== null && divInfo?.perShare !== undefined ? 'text-emerald-600' : 'text-slate-500')}
            title={formatPerShare(divInfo)}
          >
            {formatPerShare(divInfo)}
          </p>
        </div>
        <div className="min-w-0 border-l border-emerald-100/80 px-2 py-2 text-xs text-slate-600">
          <p className="truncate text-[10px] font-medium text-slate-500">年間配当</p>
          <p
            className={cn('mt-0.5 truncate text-[12px] font-semibold', divInfo?.annual != null ? 'text-emerald-600' : 'text-slate-500')}
            title={formatAnnual(divInfo)}
          >
            {formatAnnual(divInfo)}
          </p>
        </div>
        <div className="min-w-0 border-l border-emerald-100/80 px-2 py-2 text-xs text-slate-600">
          <p className="truncate text-[10px] font-medium text-slate-500">配当利回り</p>
          <p
            className={cn('mt-0.5 truncate text-[12px] font-semibold', divInfo?.yieldValue != null ? 'text-emerald-600' : 'text-slate-500')}
            title={formatYield(divInfo)}
          >
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
  // 取得総額の合計が0でも、評価額を持つ銘柄があればカードを表示する (Issue #851 改訂2)。
  // その場合の取得額構成比は分母が0のため算出不可 (NaN) とする。
  const chartData: ChartDataItem[] = (() => {
    const total = data.reduce((sum, item) => sum + item.value, 0);
    if (total === 0) {
      const hasValuation = data.some((item) => {
        const market = toFiniteAmount(item.marketValue);
        return market !== null && market !== 0;
      });
      if (!hasValuation) return [];
    }

    return data
      .map((item) => ({ ...item, percentage: total === 0 ? Number.NaN : (item.value / total) * 100 }))
      .sort((a, b) => b.percentage - a.percentage);
  })();

  // 表示データ（Top N または全件）
  const displayData = (() => {
    if (showAll || chartData.length <= TOP_N) return chartData;
    return chartData.slice(0, TOP_N);
  })();

  // その他の合計（Top N以外）
  const othersPercentage = (() => {
    if (showAll || chartData.length <= TOP_N) return 0;
    return chartData.slice(TOP_N).reduce((sum, item) => sum + item.percentage, 0);
  })();

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
      <div className={gridClassName} data-testid="portfolio-items-grid">
        {displayData.map((item, index) => (
          <div key={item.securityCode}>
            <PortfolioItemCard
              item={item}
              index={index}
              dividendPerShareMap={dividendPerShareMap}
              dividendStatusMap={dividendStatusMap}
            />
            <PortfolioValuationCard
              item={item}
              dividendPerShareMap={dividendPerShareMap}
              dividendStatusMap={dividendStatusMap}
            />
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
                {formatPercentageValue(othersPercentage, 1)}
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
