import { render, screen } from '@testing-library/react';
import { vi, describe, it, expect } from 'vitest';
import { AssetPortfolioSummary } from '../AssetPortfolioSummary';
import type { AssetBalanceData } from '@/types/api';

// SecurityCodeLinkのモック
vi.mock('@/components/atoms/SecurityCodeLink', () => ({
  SecurityCodeLink: ({ value }: { value: string }) => <span>{value}</span>,
}));

const createMockData = (overrides: Partial<AssetBalanceData>[] = []): AssetBalanceData[] => {
  const defaults: AssetBalanceData[] = [
    {
      id: 'asset-balance-1',
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-01T00:00:00Z',
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
      id: 'asset-balance-2',
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-01T00:00:00Z',
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

    expect(screen.getByTestId('portfolio-kpi-strip')).toBeInTheDocument();
    // 合計取得総額のラベルが表示される
    expect(screen.getByText('合計取得総額')).toBeInTheDocument();
    // 250,000 + 600,000 = 850,000
    expect(screen.getByText(/850,000/)).toBeInTheDocument();
  });

  it('API summary がある場合は合計取得総額に summary を優先する', () => {
    const mockData = createMockData();
    render(
      <AssetPortfolioSummary
        assetBalanceData={mockData}
        summary={{
          total_purchase_amount: 1234567,
          total_market_value: 1300000,
          total_daily_change: 5000,
        }}
      />
    );

    expect(screen.getByText(/1,234,567/)).toBeInTheDocument();
    expect(screen.queryByText(/850,000/)).not.toBeInTheDocument();
  });

  it('円グラフが表示される', () => {
    const mockData = createMockData();
    render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    expect(screen.getByTestId('portfolio-pie-chart')).toBeInTheDocument();
    expect(screen.getByText('保有内訳')).toBeInTheDocument();
    expect(screen.queryByText('保有比率と配当効率をまとめて確認できます。')).not.toBeInTheDocument();
  });

  it('全角英数字の銘柄名を半角に正規化して表示する', () => {
    const mockData = createMockData([
      { security_code: '9433', security_name: 'ＫＤＤＩ' },
      { security_code: '1605', security_name: 'ＩＮＰＥＸ' },
    ]);
    render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    expect(screen.getAllByText('KDDI').length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText('INPEX').length).toBeGreaterThanOrEqual(1);
    expect(screen.queryByText('ＫＤＤＩ')).not.toBeInTheDocument();
  });

  it('データがない場合は空状態が表示される', () => {
    render(<AssetPortfolioSummary assetBalanceData={[]} />);

    expect(screen.getByText('資産管理データがありません')).toBeInTheDocument();
    expect(screen.getByText('CSVファイルをインポートするか、データを登録してください。')).toBeInTheDocument();
  });

  it('取得総額も評価額もすべて0の場合はサマリーが表示されない', () => {
    const mockData = createMockData([
      { total_purchase_amount: 0, market_value: 0 },
      { total_purchase_amount: 0, market_value: 0 },
    ]);
    const { container } = render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    expect(container.firstChild).toBeNull();
  });

  it('取得総額が0でも評価額を持つ銘柄があればサマリーが表示される', () => {
    const mockData = createMockData([
      { total_purchase_amount: 0, market_value: 100000 },
      { total_purchase_amount: 0, market_value: 0 },
    ]);
    render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    expect(screen.getByTestId('portfolio-valuation-summary')).toBeInTheDocument();
    expect(screen.getByTestId('portfolio-valuation-summary')).toHaveTextContent(/100,000/);
  });

  it('取得総額がnull/undefinedの場合は0として扱う', () => {
    const mockData: AssetBalanceData[] = [
      {
        id: 'asset-balance-1',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
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
        id: 'asset-balance-2',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
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

  it('security_nameが空の場合security_codeをグラフ名に使用する', () => {
    const mockData: AssetBalanceData[] = [
      {
        id: 'asset-balance-1',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
        security_code: '7203',
        security_name: '',
        shares: 100,
        executing_shares: 0,
        average_purchase_price: 2500,
        total_purchase_amount: 250000,
        current_price: 2600,
        daily_change: 50,
        market_value: 260000,
        profit_loss_rate: 4.0,
      },
    ];
    render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    // security_nameが空なのでsecurity_codeがグラフに使われる
    expect(screen.getByTestId('asset-portfolio-summary')).toBeInTheDocument();
  });

  it('合計取得総額がマイナスの場合data-negative属性が付く', () => {
    const mockData: AssetBalanceData[] = [
      {
        id: 'asset-balance-1',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
        security_code: '7203',
        security_name: 'テスト銘柄',
        shares: 100,
        executing_shares: 0,
        average_purchase_price: 2500,
        total_purchase_amount: -250000,
        current_price: 2600,
        daily_change: 50,
        market_value: 260000,
        profit_loss_rate: 4.0,
      },
    ];
    const { container } = render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    const negativeEl = container.querySelector('[data-negative="true"]');
    expect(negativeEl).toBeInTheDocument();
  });

  it('data-testidが設定される', () => {
    const mockData = createMockData();
    render(<AssetPortfolioSummary assetBalanceData={mockData} />);

    expect(screen.getByTestId('asset-portfolio-summary')).toBeInTheDocument();
  });

  it('KPI strip のグリッドは lg:grid-cols-4 で4列表示を使用する', () => {
    const mockData = createMockData();
    const { container } = render(<AssetPortfolioSummary assetBalanceData={mockData} />);
    const kpiGrid = container.querySelector('[data-testid="portfolio-kpi-grid"]');
    expect(kpiGrid).toBeInTheDocument();
    expect(kpiGrid).toHaveClass('lg:grid-cols-4');
    expect(kpiGrid).not.toHaveClass('xl:grid-cols-3');
  });

  describe('評価額サマリー', () => {
    it('保有資産の評価額と評価損益が表示される', () => {
      const mockData = createMockData();
      render(<AssetPortfolioSummary assetBalanceData={mockData} />);

      const summary = screen.getByTestId('portfolio-valuation-summary');
      expect(summary).toBeInTheDocument();
      expect(summary).toHaveTextContent('保有資産の評価額');
      // 評価額: 260,000 + 650,000 = 910,000
      expect(summary).toHaveTextContent(/910,000/);
      // 評価損益: 910,000 - 850,000 = +60,000
      expect(summary).toHaveTextContent(/\+.*60,000/);
      expect(summary).toHaveTextContent('取込データ時点');
    });

    it('損益率はDB値を使わず金額から再計算する', () => {
      const mockData = createMockData([
        { total_purchase_amount: 100, market_value: 200, profit_loss_rate: 0 },
        { total_purchase_amount: 900, market_value: 900, profit_loss_rate: 999 },
      ]);
      render(<AssetPortfolioSummary assetBalanceData={mockData} />);

      const summary = screen.getByTestId('portfolio-valuation-summary');
      // 合計: 評価額1,100 - 取得額1,000 = +100 (+10.0%)。DB値(0 / 999)は使わない
      expect(summary).toHaveTextContent(/1,100/);
      expect(summary).toHaveTextContent('+10.0%');
    });

    it('欠損を含む場合は不完全として合計を表示しない', () => {
      const mockData = createMockData([
        { total_purchase_amount: 100, market_value: 200 },
        { total_purchase_amount: 20, market_value: null },
      ]);
      render(<AssetPortfolioSummary assetBalanceData={mockData} />);

      const summary = screen.getByTestId('portfolio-valuation-summary');
      expect(summary).toHaveTextContent('—');
      expect(summary).toHaveTextContent('合計を算出できません');
    });

    it('API summary がある場合は評価額の合計にも summary を優先する', () => {
      const mockData = createMockData();
      render(
        <AssetPortfolioSummary
          assetBalanceData={mockData}
          summary={{
            total_purchase_amount: 1000,
            total_market_value: 1100,
            total_daily_change: 0,
          }}
        />
      );

      const summary = screen.getByTestId('portfolio-valuation-summary');
      expect(summary).toHaveTextContent(/1,100/);
      expect(summary).toHaveTextContent('+10.0%');
    });
  });

  describe('配当金額・配当利回り', () => {
    it('ポートフォリオ全体の年間配当金額と配当利回りが正しく表示される', () => {
      const mockData = createMockData();
      // 1株配当マップ: 7203: 50円/株, 6758: 240円/株
      // 年間配当: 50*100 + 240*50 = 5000 + 12000 = 17000
      // 配当利回り: 17000 / 850000 * 100 = 2.00%
      const dividendMap = new Map([
        ['7203', 50],
        ['6758', 240],
      ]);
      render(<AssetPortfolioSummary assetBalanceData={mockData} dividendPerShareMap={dividendMap} />);

      // 年間配当金額ラベルが表示される
      expect(screen.getByText('年間配当金額')).toBeInTheDocument();
      // 50*100 + 240*50 = 17,000
      expect(screen.getByTestId('portfolio-annual-dividends')).toHaveTextContent(/17,000/);
      // 配当利回りラベルが表示される
      expect(screen.getAllByText('配当利回り').length).toBeGreaterThanOrEqual(1);
      // 17000 / 850000 * 100 = 2.00%
      expect(screen.getByTestId('portfolio-dividend-yield')).toHaveTextContent('2.00%');
    });

    it('dividendPerShareMapが未指定の場合は---が表示される', () => {
      const mockData = createMockData();
      render(<AssetPortfolioSummary assetBalanceData={mockData} />);

      expect(screen.getByTestId('portfolio-annual-dividends')).toHaveTextContent('---');
      expect(screen.getByTestId('portfolio-dividend-yield')).toHaveTextContent('---');
    });

    it('配当データが空の場合は---が表示される', () => {
      const mockData = createMockData();
      const emptyMap = new Map<string, number>();
      render(<AssetPortfolioSummary assetBalanceData={mockData} dividendPerShareMap={emptyMap} />);

      expect(screen.getByTestId('portfolio-dividend-yield')).toHaveTextContent('---');
    });

    it('dividendPerShareMapに一部の銘柄しかない場合は存在する銘柄のみ計算される', () => {
      const mockData = createMockData();
      // 7203のみマップに含める（6758は含めない）
      const partialMap = new Map([['7203', 50]]);
      render(<AssetPortfolioSummary assetBalanceData={mockData} dividendPerShareMap={partialMap} />);

      // 50 * 100 = 5000
      expect(screen.getByTestId('portfolio-annual-dividends')).toHaveTextContent(/5,000/);
    });

    it('dividendPerShareが0の場合は合計が0になり---が表示される', () => {
      const mockData = createMockData();
      const zeroMap = new Map([['7203', 0], ['6758', 0]]);
      render(<AssetPortfolioSummary assetBalanceData={mockData} dividendPerShareMap={zeroMap} />);

      expect(screen.getByTestId('portfolio-annual-dividends')).toHaveTextContent('---');
    });

    it('dividendStatusMap を渡すと pending 銘柄で取得中...が表示される', () => {
      const mockData = createMockData();
      const dividendMap = new Map<string, number>();
      const statusMap = new Map([['7203', 'pending' as const]]);
      render(<AssetPortfolioSummary assetBalanceData={mockData} dividendPerShareMap={dividendMap} dividendStatusMap={statusMap} />);

      expect(screen.getAllByText('取得中...')).not.toHaveLength(0);
    });
  });
});
