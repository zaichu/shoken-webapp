import { render, screen } from '@testing-library/react';
import { vi, describe, it, expect } from 'vitest';
import { PortfolioPieChart, PortfolioItem } from '../PortfolioPieChart';

// SecurityCodeLinkのモック
vi.mock('@/components/atoms/SecurityCodeLink', () => ({
  SecurityCodeLink: ({ value }: { value: string }) => <span data-testid="security-code-link">{value}</span>,
}));

describe('PortfolioPieChart', () => {
  const mockData: PortfolioItem[] = [
    { name: 'トヨタ自動車', value: 250000, securityCode: '7203', shares: 100 },
    { name: 'ソニーグループ', value: 600000, securityCode: '6758', shares: 50 },
    { name: '任天堂', value: 150000, securityCode: '7974', shares: 30 },
  ];

  it('横棒グラフが表示される', () => {
    render(<PortfolioPieChart data={mockData} />);

    expect(screen.getByTestId('portfolio-pie-chart')).toBeInTheDocument();
  });

  it('銘柄名とパーセンテージが表示される', () => {
    render(<PortfolioPieChart data={mockData} />);

    expect(screen.getByText('トヨタ自動車')).toBeInTheDocument();
    expect(screen.getByText('ソニーグループ')).toBeInTheDocument();
    expect(screen.getByText('任天堂')).toBeInTheDocument();
    // パーセンテージが表示される
    expect(screen.getByText('60.0%')).toBeInTheDocument();
    expect(screen.getByText('25.0%')).toBeInTheDocument();
    expect(screen.getByText('15.0%')).toBeInTheDocument();
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
});
