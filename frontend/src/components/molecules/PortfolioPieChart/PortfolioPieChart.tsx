import React, { useMemo, useState, useEffect } from 'react';
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from 'recharts';
import { SecurityCodeLink } from '@/components/atoms/SecurityCodeLink';
import { formatCurrency, formatNumber } from '@/lib/utils/formatters';

// 円グラフ用の配色（視認性を考慮した10色）
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

export interface PortfolioItem {
  name: string;
  value: number;
  securityCode?: string;
  shares?: number;
}

interface ChartDataItem extends PortfolioItem {
  percentage: number;
  securityCode?: string;
  shares?: number;
}

interface PortfolioPieChartProps {
  data: PortfolioItem[];
  className?: string;
}

interface TooltipPayloadItem {
  name: string;
  value: number;
  payload: ChartDataItem;
}

// ツールチップの金額・パーセンテージ表示
const CustomTooltip = ({
  active,
  payload,
}: {
  active?: boolean;
  payload?: TooltipPayloadItem[];
}) => {
  if (active && payload && payload.length > 0) {
    const item = payload[0].payload;
    return (
      <div className="rounded border border-border bg-white px-3 py-2 shadow-sm">
        <p className="text-sm font-medium text-dark">{item.name}</p>
        {item.securityCode && (
          <p className="text-xs text-slate-500">{item.securityCode}</p>
        )}
        <p className="text-sm text-secondary">
          {formatCurrency(item.value)}
        </p>
        {item.shares !== undefined && (
          <p className="text-sm text-secondary">{formatNumber(item.shares)}株</p>
        )}
        <p className="text-sm text-secondary">{item.percentage.toFixed(1)}%</p>
      </div>
    );
  }
  return null;
};

// 画面幅を監視するフック
const useIsMobile = () => {
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    const checkMobile = () => setIsMobile(window.innerWidth < 640);
    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  }, []);

  return isMobile;
};

/**
 * ポートフォリオ構成比を表示する円グラフコンポーネント
 */
export const PortfolioPieChart: React.FC<PortfolioPieChartProps> = ({
  data,
  className = '',
}) => {
  const isMobile = useIsMobile();

  // パーセンテージを計算してデータに追加
  const chartData = useMemo(() => {
    const total = data.reduce((sum, item) => sum + item.value, 0);
    if (total === 0) return [];

    return data.map((item) => ({
      ...item,
      percentage: (item.value / total) * 100,
    }));
  }, [data]);

  if (chartData.length === 0) {
    return null;
  }

  // カスタム凡例コンポーネント（詳細情報付き）
  const CustomLegend = () => (
    <div className="space-y-3">
      {chartData.map((item, index) => (
        <div
          key={item.securityCode || item.name}
          className="rounded-lg border border-slate-200 bg-slate-50 p-3"
        >
          {/* ヘッダー: 色マーカー + 銘柄名 + 構成比 */}
          <div className="flex items-center gap-2">
            <span
              className="h-3 w-3 shrink-0 rounded-sm"
              style={{ backgroundColor: COLORS[index % COLORS.length] }}
            />
            <span className="min-w-0 flex-1 truncate font-medium text-slate-700" title={item.name}>
              {item.name}
            </span>
            <span className="shrink-0 text-sm font-bold text-slate-600">
              {item.percentage.toFixed(1)}%
            </span>
          </div>
          {/* 詳細情報: 銘柄コード(リンク)、取得総額、保有数量 */}
          <div className="mt-2 grid grid-cols-3 gap-2 text-xs">
            <div>
              <span className="text-slate-500">コード</span>
              <div className="mt-0.5">
                {item.securityCode ? (
                  <SecurityCodeLink value={item.securityCode} className="text-xs" />
                ) : (
                  <span className="text-slate-400">-</span>
                )}
              </div>
            </div>
            <div>
              <span className="text-slate-500">取得総額</span>
              <div className="mt-0.5 font-medium text-slate-700">
                {formatCurrency(item.value)}
              </div>
            </div>
            <div>
              <span className="text-slate-500">保有数量</span>
              <div className="mt-0.5 font-medium text-slate-700">
                {item.shares !== undefined ? `${formatNumber(item.shares)}株` : '-'}
              </div>
            </div>
          </div>
          {/* パーセンテージバー */}
          <div className="mt-2 h-1.5 w-full rounded-full bg-slate-200">
            <div
              className="h-full rounded-full transition-all"
              style={{
                width: `${Math.min(item.percentage, 100)}%`,
                backgroundColor: COLORS[index % COLORS.length],
              }}
            />
          </div>
        </div>
      ))}
    </div>
  );

  return (
    <div className={className} data-testid="portfolio-pie-chart">
      <div className={isMobile ? 'flex flex-col gap-4' : 'flex items-start gap-6'}>
        {/* 円グラフ */}
        <div className={isMobile ? 'mx-auto w-48' : 'w-44 shrink-0'}>
          <ResponsiveContainer width="100%" height={isMobile ? 160 : 180}>
            <PieChart>
              <Pie
                data={chartData}
                cx="50%"
                cy="50%"
                innerRadius={isMobile ? 35 : 45}
                outerRadius={isMobile ? 60 : 75}
                paddingAngle={2}
                dataKey="value"
                nameKey="name"
              >
                {chartData.map((_, index) => (
                  <Cell
                    key={`cell-${index}`}
                    fill={COLORS[index % COLORS.length]}
                  />
                ))}
              </Pie>
              <Tooltip content={<CustomTooltip />} />
            </PieChart>
          </ResponsiveContainer>
        </div>
        {/* 凡例リスト */}
        <div className="min-w-0 flex-1">
          <CustomLegend />
        </div>
      </div>
    </div>
  );
};
