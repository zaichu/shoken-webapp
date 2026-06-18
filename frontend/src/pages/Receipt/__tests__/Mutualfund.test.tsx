import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import { Mutualfund } from '../Mutualfund';
import { waitOpts } from '@/test/utils';

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
        {
            trade_date: new Date('2023-02-20'),
            settlement_date: new Date('2023-02-24'),
            fund_name: 'テストファンドB',
            dividends: '0',
            account: 'NISA',
            shares: 5000,
            exchange_rate: 1,
            cancellation_unit_price_yen: 18000,
            cancellation_amount_yen: 90000000,
            average_acquisition_price_yen: 14000,
            realized_profit_and_loss: 20000000,
            taxes: 0,
            realized_profit_and_loss_after_tax: 20000000,
        },
        {
            trade_date: new Date('2024-03-10'),
            settlement_date: new Date('2024-03-14'),
            fund_name: 'テストファンドC',
            dividends: '0',
            account: '特定',
            shares: 8000,
            exchange_rate: 1,
            cancellation_unit_price_yen: 12000,
            cancellation_amount_yen: 96000000,
            average_acquisition_price_yen: 12500,
            realized_profit_and_loss: -4000000,
            taxes: 0,
            realized_profit_and_loss_after_tax: -4000000,
        },
    ];

    it('空のデータでEmptyStateが表示される', () => {
        render(<Mutualfund data={[]} />);

        expect(screen.getByText('データがありません')).toBeInTheDocument();
        expect(screen.getByText('投資信託明細をCSVで追加してください')).toBeInTheDocument();
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

    it('ファンド名を右クリックするとファンド名をコピーする', () => {
        const writeText = vi.fn().mockResolvedValue(undefined);
        Object.defineProperty(navigator, 'clipboard', {
            value: { writeText },
            configurable: true,
        });

        render(<Mutualfund data={mockData} />);

        fireEvent.contextMenu(screen.getByText('テストファンドA'));

        expect(writeText).toHaveBeenCalledWith('テストファンドA');
    });

    it('ファンド名検索でフィルタリングされる', async () => {
        const user = userEvent.setup();
        const { container } = render(<Mutualfund data={mockData} />);

        fireEvent.click(screen.getByTestId('search-card-header'));

        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        }, waitOpts);

        const securitiesSelect = container.querySelector('#securities-search') as HTMLSelectElement;
        await user.selectOptions(securitiesSelect, 'テストファンドA');

        const table = screen.getByRole('table');
        await waitFor(() => {
            expect(within(table).getAllByText('テストファンドA').length).toBeGreaterThanOrEqual(1);
            expect(within(table).queryByText('テストファンドB')).not.toBeInTheDocument();
            expect(within(table).queryByText('2023年1月')).not.toBeInTheDocument();
        }, waitOpts);
    });

    it('年検索でフィルタリングされる', async () => {
        const user = userEvent.setup();
        const { container } = render(<Mutualfund data={mockData} />);

        fireEvent.click(screen.getByTestId('search-card-header'));

        await waitFor(() => {
            expect(container.querySelector('#years-search')).toBeInTheDocument();
        }, waitOpts);

        const yearSelect = container.querySelector('#years-search') as HTMLSelectElement;
        await user.selectOptions(yearSelect, '2023');

        const table = screen.getByRole('table');
        await waitFor(() => {
            expect(within(table).getByText('テストファンドA')).toBeInTheDocument();
            expect(within(table).getByText('テストファンドB')).toBeInTheDocument();
            expect(within(table).queryByText('テストファンドC')).not.toBeInTheDocument();
            expect(within(table).getByText('2023年1月')).toBeInTheDocument();
            expect(within(table).getByText('2023年2月')).toBeInTheDocument();
            expect(within(table).queryByText('2024年3月')).not.toBeInTheDocument();
        }, waitOpts);
    });
});
