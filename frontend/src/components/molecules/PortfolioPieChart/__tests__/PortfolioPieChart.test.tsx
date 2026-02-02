import { render, screen } from '@testing-library/react';
import { vi, describe, it, expect } from 'vitest';
import { PortfolioPieChart, PortfolioItem } from '../PortfolioPieChart';

// rechartsのモック（ResponsiveContainerの問題を回避）
vi.mock('recharts', () => ({
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="responsive-container">{children}</div>
  ),
  PieChart: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="pie-chart">{children}</div>
  ),
  Pie: () => <div data-testid="pie" />,
  Cell: () => null,
  Tooltip: () => null,
  Legend: () => <div data-testid="legend" />,
}));

describe('PortfolioPieChart', () => {
  const mockData: PortfolioItem[] = [
    { name: 'トヨタ自動車', value: 250000 },
    { name: 'ソニーグループ', value: 600000 },
    { name: '任天堂', value: 150000 },
  ];

  it('円グラフが表示される', () => {
    render(<PortfolioPieChart data={mockData} />);

    expect(screen.getByTestId('portfolio-pie-chart')).toBeInTheDocument();
    expect(screen.getByTestId('pie-chart')).toBeInTheDocument();
  });

  it('data-testidが設定される', () => {
    render(<PortfolioPieChart data={mockData} />);

    expect(screen.getByTestId('portfolio-pie-chart')).toBeInTheDocument();
  });

  it('空データの場合は何も表示しない', () => {
    const { container } = render(<PortfolioPieChart data={[]} />);

    expect(container.firstChild).toBeNull();
  });

  it('すべての値が0の場合は何も表示しない', () => {
    const zeroData: PortfolioItem[] = [
      { name: '銘柄A', value: 0 },
      { name: '銘柄B', value: 0 },
    ];
    const { container } = render(<PortfolioPieChart data={zeroData} />);

    expect(container.firstChild).toBeNull();
  });

  it('カスタムクラス名が適用される', () => {
    render(<PortfolioPieChart data={mockData} className="custom-chart" />);

    const chartContainer = screen.getByTestId('portfolio-pie-chart');
    expect(chartContainer).toHaveClass('custom-chart');
  });

  it('Legendコンポーネントがレンダリングされる', () => {
    render(<PortfolioPieChart data={mockData} />);

    expect(screen.getByTestId('legend')).toBeInTheDocument();
  });
});
