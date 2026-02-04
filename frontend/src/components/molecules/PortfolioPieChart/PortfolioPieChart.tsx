import React, { useMemo, useState, useEffect } from 'react';
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from 'recharts';

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
}

interface ChartDataItem extends PortfolioItem {
  percentage: number;
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
        <p className="text-sm text-secondary">
          ¥ {item.value.toLocaleString('ja-JP')}
        </p>
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

  // カスタム凡例コンポーネント（パーセンテージバー付き）
  const CustomLegend = () => (
    <div className="space-y-2">
      {chartData.map((item, index) => (
        <div key={item.name} className="flex items-center gap-3 text-sm">
          {/* 色マーカー */}
          <span
            className="h-3 w-3 shrink-0 rounded-sm"
            style={{ backgroundColor: COLORS[index % COLORS.length] }}
          />
          {/* 銘柄名とパーセンテージ */}
          <div className="min-w-0 flex-1">
            <div className="flex items-baseline justify-between gap-2">
              <span className="truncate text-slate-700" title={item.name}>
                {item.name}
              </span>
              <span className="shrink-0 text-xs font-medium text-slate-500">
                {item.percentage.toFixed(1)}%
              </span>
            </div>
            {/* パーセンテージバー */}
            <div className="mt-1 h-1.5 w-full rounded-full bg-slate-100">
              <div
                className="h-full rounded-full transition-all"
                style={{
                  width: `${Math.min(item.percentage, 100)}%`,
                  backgroundColor: COLORS[index % COLORS.length],
                }}
              />
            </div>
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
