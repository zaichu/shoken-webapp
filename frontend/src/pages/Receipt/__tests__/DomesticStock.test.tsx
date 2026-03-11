import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import { DomesticStock } from '../DomesticStock';
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

describe('DomesticStock', () => {
    const mockData = [
        {
            trade_date: new Date('2023-01-15'),
            settlement_date: new Date('2023-01-18'),
            security_code: '1234',
            security_name: 'テスト株式',
            account: '特定',
            shares: 100,
            asked_price: 1000,
            proceeds: 100000,
            purchase_price: 900,
            realized_profit_and_loss: 10000,
            taxes: 2031,
            realized_profit_and_loss_after_tax: 7969,
        },
        {
            trade_date: new Date('2023-01-20'),
            settlement_date: new Date('2023-01-23'),
            security_code: '5678',
            security_name: 'テスト株式2',
            account: 'NISA',
            shares: 200,
            asked_price: 2000,
            proceeds: 400000,
            purchase_price: 1800,
            realized_profit_and_loss: 40000,
            taxes: 0,
            realized_profit_and_loss_after_tax: 40000,
        }
    ];

    it('コンポーネントが正常にレンダリングされる', () => {
        render(<DomesticStock data={mockData} />);

        // 集計情報が表示されることを確認（複数ある場合は最初のものをチェック）
        const totalProfitElements = screen.getAllByText('実現損益');
        expect(totalProfitElements.length).toBeGreaterThan(0);

        const totalTaxElements = screen.getAllByText('税額');
        expect(totalTaxElements.length).toBeGreaterThan(0);

        const netProfitElements = screen.getAllByText('実現損益(税引)');
        expect(netProfitElements.length).toBeGreaterThan(0);
    });

    it('テーブルのヘッダーが正しく表示される', () => {
        render(<DomesticStock data={mockData} />);

        // テーブルヘッダーの確認（実際のカラム定義に合わせる）
        // 「口座」は検索オプション内にも表示されるためgetAllByTextを使用
        expect(screen.getByText('約定日')).toBeInTheDocument();
        expect(screen.getByText('銘柄コード')).toBeInTheDocument();
        expect(screen.getByText('銘柄名')).toBeInTheDocument();
        expect(screen.getAllByText('口座').length).toBeGreaterThan(0);
        expect(screen.getByText('数量')).toBeInTheDocument();
        expect(screen.getByText('売却単価')).toBeInTheDocument();
        expect(screen.getByText('売却額')).toBeInTheDocument();
        expect(screen.getByText('取得価額')).toBeInTheDocument();
        expect(screen.getByText('損益')).toBeInTheDocument();
        // 「税額」は集計情報にも表示されるためgetAllByTextを使用
        expect(screen.getAllByText('税額').length).toBeGreaterThan(0);
        expect(screen.getByText('税引後')).toBeInTheDocument();
    });

    it('CSVデータが正しく表示される', () => {
        render(<DomesticStock data={mockData} />);
        
        // データの内容確認
        expect(screen.getByText('1234')).toBeInTheDocument();
        expect(screen.getByText('テスト株式')).toBeInTheDocument();
        expect(screen.getByText('5678')).toBeInTheDocument();
        expect(screen.getByText('テスト株式2')).toBeInTheDocument();
    });

    it('検索オプションが正しく生成される', async () => {
        const { container } = render(<DomesticStock data={mockData} />);

        // 検索オプションヘッダーの確認
        const searchOptionsHeader = screen.getByText('検索オプション');
        expect(searchOptionsHeader).toBeInTheDocument();

        // SearchCardを展開してから銘柄検索セレクトボックスを確認
        const searchCardHeader = screen.getByTestId('search-card-header');
        searchCardHeader.click();

        // 銘柄検索セレクトボックスの確認（IDで指定）
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        });

        // オプションの確認（複数の「全て表示」があるためgetAllByTextを使用）
        const allOptions = screen.getAllByText('全て表示');
        expect(allOptions.length).toBeGreaterThanOrEqual(1);
        expect(screen.getByText('1234: テスト株式')).toBeInTheDocument();
        expect(screen.getByText('5678: テスト株式2')).toBeInTheDocument();
    });

    it('銘柄検索時に0件サマリーが表示されない', async () => {
        const user = userEvent.setup();
        const { container } = render(<DomesticStock data={mockData} />);

        // SearchCardを展開してから銘柄検索セレクトボックスを操作
        const searchCardHeader = screen.getByTestId('search-card-header');
        searchCardHeader.click();
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        }, waitOpts);
        const securitiesSelect = container.querySelector('#securities-search') as HTMLSelectElement;

        await user.selectOptions(securitiesSelect, '1234');

        await waitFor(() => {
            expect(screen.queryByText('0件')).not.toBeInTheDocument();
        }, waitOpts);
    }, 20000);

    it('空のデータでEmptyStateが表示される', () => {
        render(<DomesticStock data={[]} />);

        expect(screen.getByText('データがありません')).toBeInTheDocument();
        expect(screen.getByText('国内株式明細をCSVで追加してください')).toBeInTheDocument();
    });

    it('数値フォーマットが正しく適用される', () => {
        render(<DomesticStock data={mockData} />);
        
        // 通貨フォーマットされた値が存在することを確認
        // 具体的な値は実装に依存するため、¥記号の存在を確認
        const currencyElements = screen.getAllByText(/¥/);
        expect(currencyElements.length).toBeGreaterThan(0);
    });

    it('レスポンシブテーブルが使用される', () => {
        const { container } = render(<DomesticStock data={mockData} />);
        
        // レスポンシブテーブルのクラスが適用されていることを確認
        const responsiveTable = container.querySelector('div.overflow-x-hidden');
        expect(responsiveTable).toBeInTheDocument();
        
        const table = screen.getByRole('table');
        expect(table).toBeInTheDocument();
        expect(table).toHaveClass('w-full', 'table-fixed');
    });
});
