import { render, screen } from '@testing-library/react';
import { vi, describe, it, expect } from 'vitest';
import { AssetPortfolioSummary } from '../AssetPortfolioSummary';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';

// SecurityCodeLinkのモック
vi.mock('@/components/atoms/SecurityCodeLink', () => ({
  SecurityCodeLink: ({ value }: { value: string }) => <span>{value}</span>,
}));

const createMockData = (overrides: Partial<AssetBalanceData>[] = []): AssetBalanceData[] => {
  const defaults: AssetBalanceData[] = [
    {
      security_code: '7203',
      security_name: 'トヨタ自動車',
      shares: 100,
      executing_shares: 0,
      average_purchase_price: 2500,
      total_purchase_amount: 250000,
      current_price: 2600,
      daily_change: 50,
      market_value: 260000,
      profit_loss_rate: 4.0,
    },
    {
      security_code: '6758',
      security_name: 'ソニーグループ',
      shares: 50,
      executing_shares: 0,
      average_purchase_price: 12000,
      total_purchase_amount: 600000,
      current_price: 13000,
      daily_change: 200,
      market_value: 650000,
      profit_loss_rate: 8.33,
    },
  ];

  if (overrides.length > 0) {
    return overrides.map((override, index) => ({
      ...defaults[index % defaults.length],
      ...override,
    }));
  }
  return defaults;
};

describe('AssetPortfolioSummary', () => {
  it('合計取得総額が正しく表示される', () => {
    const mockData = createMockData();
    render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    // 合計取得総額のラベルが表示される
    expect(screen.getByText('合計取得総額')).toBeInTheDocument();
    // 250,000 + 600,000 = 850,000
    expect(screen.getByText(/850,000/)).toBeInTheDocument();
  });

  it('円グラフが表示される', () => {
    const mockData = createMockData();
    render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    expect(screen.getByTestId('portfolio-pie-chart')).toBeInTheDocument();
    expect(screen.getByText('銘柄別構成比')).toBeInTheDocument();
  });

  it('データがない場合は空状態が表示される', () => {
    render(<AssetPortfolioSummary assetBalanceData={[]} />);

    expect(screen.getByText('資産管理データがありません')).toBeInTheDocument();
    expect(screen.getByText('CSVファイルをインポートするか、データを登録してください。')).toBeInTheDocument();
  });

  it('取得総額がすべて0の場合はサマリーが表示されない', () => {
    const mockData = createMockData([
      { total_purchase_amount: 0 },
      { total_purchase_amount: 0 },
    ]);
    const { container } = render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    expect(container.firstChild).toBeNull();
  });

  it('取得総額がnull/undefinedの場合は0として扱う', () => {
    const mockData: AssetBalanceData[] = [
      {
        security_code: '7203',
        security_name: 'トヨタ自動車',
        shares: 100,
        executing_shares: 0,
        average_purchase_price: 2500,
        total_purchase_amount: undefined as unknown as number,
        current_price: 2600,
        daily_change: 50,
        market_value: 260000,
        profit_loss_rate: 4.0,
      },
      {
        security_code: '6758',
        security_name: 'ソニーグループ',
        shares: 50,
        executing_shares: 0,
        average_purchase_price: 12000,
        total_purchase_amount: 600000,
        current_price: 13000,
        daily_change: 200,
        market_value: 650000,
        profit_loss_rate: 8.33,
      },
    ];
    render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    // undefinedは0として扱われるので、600,000のみが合計される
    // KPIとグラフカードの両方に表示されるためgetAllByTextを使用
    const matches = screen.getAllByText(/600,000/);
    expect(matches.length).toBeGreaterThanOrEqual(1);
  });

  it('data-testidが設定される', () => {
    const mockData = createMockData();
    render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    expect(screen.getByTestId('asset-portfolio-summary')).toBeInTheDocument();
  });
});
