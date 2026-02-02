import React, { useMemo, useState, useEffect } from 'react';
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip, Legend } from 'recharts';

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

  // 凡例のカスタムフォーマッター（銘柄名 + パーセンテージを表示）
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const legendFormatter = (value: string, entry: any) => {
    const percentage = entry.payload?.percentage as number | undefined;
    if (percentage === undefined) {
      return <span className="text-sm text-secondary">{value}</span>;
    }

    return (
      <span className="text-sm text-secondary">
        {value} ({percentage.toFixed(1)}%)
      </span>
    );
  };

  if (chartData.length === 0) {
    return null;
  }

  return (
    <div className={className} data-testid="portfolio-pie-chart">
      <ResponsiveContainer width="100%" height={isMobile ? 350 : 250}>
        <PieChart>
          <Pie
            data={chartData}
            cx="50%"
            cy={isMobile ? '30%' : '50%'}
            innerRadius={isMobile ? 40 : 50}
            outerRadius={isMobile ? 65 : 80}
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
          <Legend
            layout={isMobile ? 'horizontal' : 'vertical'}
            align={isMobile ? 'center' : 'right'}
            verticalAlign={isMobile ? 'bottom' : 'middle'}
            wrapperStyle={isMobile ? { paddingTop: 16 } : undefined}
            formatter={legendFormatter}
          />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
};
