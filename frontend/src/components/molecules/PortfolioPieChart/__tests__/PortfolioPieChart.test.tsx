import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi, describe, it, expect } from 'vitest';
import { PortfolioPieChart, PortfolioItem } from '../PortfolioPieChart';

// SecurityCodeLinkのモック
vi.mock('@/components/atoms/SecurityCodeLink', () => ({
  SecurityCodeLink: ({ value }: { value: string }) => <span data-testid="security-code-link">{value}</span>,
}));

describe('PortfolioPieChart', () => {
  const mockData: PortfolioItem[] = [
    { name: 'トヨタ自動車', value: 250000, securityCode: '7203', shares: 100, averagePrice: 2500 },
    { name: 'ソニーグループ', value: 600000, securityCode: '6758', shares: 50, averagePrice: 12000 },
    { name: '任天堂', value: 150000, securityCode: '7974', shares: 30, averagePrice: 5000 },
  ];
  const createLargeData = (count: number): PortfolioItem[] =>
    Array.from({ length: count }, (_, index) => {
      const itemNumber = index + 1;
      const paddedNumber = String(itemNumber).padStart(2, '0');

      return {
        name: `銘柄${paddedNumber}`,
        value: (count - index) * 1000,
        securityCode: `${1000 + itemNumber}`,
        shares: itemNumber * 10,
        averagePrice: itemNumber * 100,
      };
    });

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
    expect(screen.queryAllByText('構成比')).toHaveLength(0);
  });

  it('銘柄コードはタイトル直下の code chip として表示される', () => {
    render(<PortfolioPieChart data={mockData} />);

    expect(screen.queryAllByText('コード:')).toHaveLength(0);
    const codeChips = screen.getAllByTestId('portfolio-card-code');
    expect(codeChips).toHaveLength(3);
    expect(codeChips.map((chip) => chip.textContent)).toEqual(expect.arrayContaining(['7203', '6758', '7974']));
  });

  it('取得総額・取得単価・数量は3列の acquisition strip で表示される', () => {
    const { container } = render(<PortfolioPieChart data={mockData} />);

    const acquisitionStrips = container.querySelectorAll('[data-testid="portfolio-card-acquisition-stats"]');
    expect(acquisitionStrips).toHaveLength(3);
    acquisitionStrips.forEach((strip) => {
      expect(strip).toHaveClass('grid', 'grid-cols-3');
    });
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
      { name: '銘柄A', value: 0, securityCode: '0001', shares: 0, averagePrice: 0 },
      { name: '銘柄B', value: 0, securityCode: '0002', shares: 0, averagePrice: 0 },
    ];
    const { container } = render(<PortfolioPieChart data={zeroData} />);

    expect(container.firstChild).toBeNull();
  });

  it('3件以上のデータでは保有内訳カードを xl:grid-cols-3 で表示する', () => {
    const { container } = render(<PortfolioPieChart data={mockData} />);
    const grid = container.querySelector('[data-testid="portfolio-items-grid"]');
    expect(grid).toHaveClass('xl:grid-cols-3');
    expect(grid).not.toHaveClass('xl:grid-cols-4');
  });

  it('カスタムクラス名が適用される', () => {
    render(<PortfolioPieChart data={mockData} className="custom-chart" />);

    const chartContainer = screen.getByTestId('portfolio-pie-chart');
    expect(chartContainer).toHaveClass('custom-chart');
  });

  describe('上位20件表示トグル', () => {
    it('21件以上のとき初期表示では上位20件とその他にまとめ、全件表示ボタンを出す', () => {
      const largeData = createLargeData(21);

      render(<PortfolioPieChart data={largeData} />);

      expect(screen.getAllByTestId('portfolio-card-code')).toHaveLength(20);
      expect(screen.getByText('その他 1銘柄')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: '残り1銘柄を表示（全21）' })).toBeInTheDocument();
      expect(screen.queryByText('銘柄21')).not.toBeInTheDocument();
    });

    it('全件表示ボタンのクリックで showAll が切り替わり、全件表示と折りたたみ表示を往復できる', async () => {
      const largeData = createLargeData(21);
      const user = userEvent.setup();

      render(<PortfolioPieChart data={largeData} />);

      await user.click(screen.getByRole('button', { name: '残り1銘柄を表示（全21）' }));

      expect(screen.getAllByTestId('portfolio-card-code')).toHaveLength(21);
      expect(screen.getByText('銘柄21')).toBeInTheDocument();
      expect(screen.queryByText('その他 1銘柄')).not.toBeInTheDocument();
      expect(screen.getByRole('button', { name: '上位20件のみ表示' })).toBeInTheDocument();

      await user.click(screen.getByRole('button', { name: '上位20件のみ表示' }));

      expect(screen.getAllByTestId('portfolio-card-code')).toHaveLength(20);
      expect(screen.queryByText('銘柄21')).not.toBeInTheDocument();
      expect(screen.getByText('その他 1銘柄')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: '残り1銘柄を表示（全21）' })).toBeInTheDocument();
    });
  });

  describe('配当金額・配当利回り', () => {
    it('配当データがある銘柄は1株配当・年間配当・利回りが表示される', () => {
      // 1株配当マップ（J-Quants予想値）
      const dividendMap = new Map([
        ['7203', 50],   // 1株配当50円、年間: 50*100=5000、利回り: 50/2500*100=2.00%
        ['6758', 360],  // 1株配当360円、年間: 360*50=18000、利回り: 360/12000*100=3.00%
      ]);
      render(<PortfolioPieChart data={mockData} dividendPerShareMap={dividendMap} />);

      // 1株配当ラベルが各カードに表示される
      const perShareLabels = screen.getAllByText('1株配当');
      expect(perShareLabels.length).toBeGreaterThanOrEqual(2);
      // 年間配当ラベルが各カードに表示される
      const dividendLabels = screen.getAllByText('年間配当');
      expect(dividendLabels.length).toBeGreaterThanOrEqual(2);
      // 利回りが表示される
      expect(screen.getByText('2.00%')).toBeInTheDocument();
      expect(screen.getByText('3.00%')).toBeInTheDocument();
    });

    it('配当データがない銘柄は---が表示される', () => {
      // 7974（任天堂）の配当データなし
      const dividendMap = new Map([
        ['7203', 50],
      ]);
      render(<PortfolioPieChart data={mockData} dividendPerShareMap={dividendMap} />);

      // 配当データなしの銘柄で---が表示される
      const dashes = screen.getAllByText('---');
      expect(dashes.length).toBeGreaterThanOrEqual(2);
    });

    it('pending ステータスの銘柄は "取得中..." が表示される', () => {
      const dividendMap = new Map<string, number>();
      const statusMap = new Map([['7203', 'pending' as const]]);
      render(<PortfolioPieChart data={mockData} dividendPerShareMap={dividendMap} dividendStatusMap={statusMap} />);

      expect(screen.getAllByText('取得中...')).not.toHaveLength(0);
    });

    it('error ステータスの銘柄は "取得失敗" が表示される', () => {
      const dividendMap = new Map<string, number>();
      const statusMap = new Map([['7203', 'error' as const]]);
      render(<PortfolioPieChart data={mockData} dividendPerShareMap={dividendMap} dividendStatusMap={statusMap} />);

      expect(screen.getAllByText('取得失敗')).not.toHaveLength(0);
    });

    it('zero ステータスの銘柄は 0円 が表示される（error と区別）', () => {
      const dividendMap = new Map<string, number>();
      const statusMap = new Map([['7203', 'zero' as const], ['6758', 'error' as const]]);
      render(<PortfolioPieChart data={mockData} dividendPerShareMap={dividendMap} dividendStatusMap={statusMap} />);

      // zero: 0円表示（formatCurrency(0) = "¥ 0"）
      expect(screen.getAllByText(/¥\s*0/).length).toBeGreaterThanOrEqual(1);
      // error: 取得失敗表示
      expect(screen.getAllByText('取得失敗').length).toBeGreaterThanOrEqual(1);
    });

    it('dividendPerShareMapが未指定の場合は全銘柄---表示', () => {
      render(<PortfolioPieChart data={mockData} />);

      // 1株配当・年間配当・配当利回りラベルが各カードに表示される
      const perShareLabels = screen.getAllByText('1株配当');
      expect(perShareLabels).toHaveLength(3);
      const dividendLabels = screen.getAllByText('年間配当');
      expect(dividendLabels).toHaveLength(3);
      const yieldLabels = screen.getAllByText('配当利回り');
      expect(yieldLabels).toHaveLength(3);
      // 全て---表示（1株配当 + 年間配当 + 配当利回り = 9個）
      const dashes = screen.getAllByText('---');
      expect(dashes).toHaveLength(9);
    });
  });
});
