import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';
import { Mutualfund } from '../Mutualfund';

// React Router DOM のモック
vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: '/' }),
}));

// React Query のモック
vi.mock('@tanstack/react-query', () => ({
  QueryClient: vi.fn(() => ({
    setQueryData: vi.fn(),
    getQueryData: vi.fn(),
  })),
  QueryClientProvider: ({ children }: { children: React.ReactNode }) => children,
  useQuery: () => ({
    data: null,
    isLoading: false,
    error: null,
  }),
}));

describe('Mutualfund', () => {
    const mockData = [
        {
            trade_date: new Date('2023-01-15'),
            settlement_date: new Date('2023-01-18'),
            fund_name: 'テストファンドA',
            dividends: '0',
            account: '特定',
            shares: 10000,
            exchange_rate: 1,
            cancellation_unit_price_yen: 15000,
            cancellation_amount_yen: 150000000,
            average_acquisition_price_yen: 12000,
            realized_profit_and_loss: 30000000,
            taxes: 6123000,
            realized_profit_and_loss_after_tax: 23877000,
        },
    ];

    it('空のデータでEmptyStateが表示される', () => {
        render(<Mutualfund data={[]} />);

        expect(screen.getByText('データがありません')).toBeInTheDocument();
    });

    it('空データ時もReceiptTemplateが維持される', () => {
        const { container } = render(<Mutualfund data={[]} />);

        // ReceiptTemplate の receipt-container が存在する（テンプレート全体が維持されている）
        expect(container.querySelector('[data-testid="receipt-container"]')).toBeInTheDocument();
    });

    it('データがある場合にテーブルヘッダーが表示される', () => {
        render(<Mutualfund data={mockData} />);

        expect(screen.getByText('約定日')).toBeInTheDocument();
        expect(screen.getByText('ファンド名')).toBeInTheDocument();
    });
});
