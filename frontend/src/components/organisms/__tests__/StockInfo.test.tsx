import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { StockInfo } from '../StockInfo';

const mockStockData = {
  date: '2026-03-16',
  code: '7203',
  name: 'トヨタ自動車',
  market_category: 'プライム',
  industry_code_33: '3700',
  industry_category_33: '輸送用機器',
  industry_code_17: '05',
  industry_category_17: '自動車・輸送機',
  size_code: '7',
  size_category: '大型株',
};

describe('StockInfo', () => {
  it('銘柄コードを表示する', () => {
    render(
      <MemoryRouter>
        <StockInfo stockData={mockStockData} />
      </MemoryRouter>
    );

    expect(screen.getAllByText('7203').length).toBeGreaterThan(0);
  });

  it('銘柄名を表示する', () => {
    render(
      <MemoryRouter>
        <StockInfo stockData={mockStockData} />
      </MemoryRouter>
    );

    expect(screen.getAllByText('トヨタ自動車').length).toBeGreaterThan(0);
  });

  it('市場カテゴリを表示する', () => {
    render(
      <MemoryRouter>
        <StockInfo stockData={mockStockData} />
      </MemoryRouter>
    );

    expect(screen.getByText('プライム')).toBeInTheDocument();
  });

  it('フィールドが空の場合は - を表示する', () => {
    render(
      <MemoryRouter>
        <StockInfo stockData={{ ...mockStockData, size_category: '' }} />
      </MemoryRouter>
    );

    expect(screen.getAllByText('-').length).toBeGreaterThan(0);
  });

  it('stockDataがnullの場合nullを返す', () => {
    const { container } = render(
      <MemoryRouter>
        <StockInfo stockData={null as unknown as typeof mockStockData} />
      </MemoryRouter>
    );

    expect(container.firstChild).toBeNull();
  });
});
