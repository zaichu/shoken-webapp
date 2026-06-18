import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import * as receiptHooks from '@/hooks/receipt/useReceiptData';
import { Dividend } from '../Dividend';

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

// J-Quants バッチ配当フックのモック
vi.mock('@/features/jquants/hooks/useDividendBatch', () => ({
  useDividendBatch: vi.fn(() => ({
    dividendPerShareMap: new Map(),
    dividendStatusMap: new Map(),
    loading: false,
    fetchedCount: 0,
    totalCount: 0,
  })),
}));

// useAssetBalanceのモック
vi.mock('@/features/assetBalance/hooks/useAssetBalance', () => ({
  useAssetBalance: vi.fn(() => ({
    assetBalanceData: [],
    isLoading: false,
    getAssetBalanceByCode: vi.fn(() => undefined),
    getTotalMarketValue: vi.fn(() => 0),
    refetch: vi.fn(),
  })),
}));

describe('Dividend', () => {
    const mockData = [
        {
            settlement_date: new Date('2023-01-15'),
            product: '株式',
            account: '特定口座',
            security_code: '1234',
            security_name: 'テスト株式1',
            unit_price: 100,
            shares: 10,
            dividends_before_tax: 1000,
            taxes: 200,
            net_amount_received: 800,
        },
        {
            settlement_date: new Date('2023-02-15'),
            product: '株式',
            account: '特定口座',
            security_code: '5678',
            security_name: 'テスト株式2',
            unit_price: 200,
            shares: 20,
            dividends_before_tax: 4000,
            taxes: 800,
            net_amount_received: 3200,
        }
    ];

    it('コンポーネントが正常にレンダリングされる', () => {
        render(<Dividend data={mockData} />);

        // タイトルが表示されることを確認（テーブルヘッダーにも「配当金」があるためgetAllByTextを使用）
        const titleElements = screen.getAllByText('配当金');
        expect(titleElements.length).toBeGreaterThan(0);

        // 集計情報が表示されることを確認（テーブルヘッダーにも同名があるためgetAllByText）
        const combinedDividendElements = screen.getAllByText('配当金');
        expect(combinedDividendElements.length).toBeGreaterThan(1);

        const totalTaxElements = screen.getAllByText('税額');
        expect(totalTaxElements.length).toBeGreaterThan(1);

        const totalReceiptElements = screen.getAllByText('受取金額');
        expect(totalReceiptElements.length).toBeGreaterThan(0);
    });

    it('テーブルのヘッダーが正しく表示される', () => {
        render(<Dividend data={mockData} />);

        // テーブルヘッダーの確認（実際のカラム定義に合わせる）
        // 「商品」「口座」は検索オプション内にも表示されるためgetAllByTextを使用
        expect(screen.getByText('入金日')).toBeInTheDocument();
        expect(screen.getAllByText('商品').length).toBeGreaterThan(0);
        expect(screen.getAllByText('口座').length).toBeGreaterThan(0);
        expect(screen.getByText('銘柄コード')).toBeInTheDocument();
        expect(screen.getByText('銘柄名')).toBeInTheDocument();
        expect(screen.getByText('単価')).toBeInTheDocument();
        expect(screen.getByText('数量')).toBeInTheDocument();
        // 「配当金」はページタイトルとしても表示されるためgetAllByTextを使用
        const dividendElements = screen.getAllByText('配当金');
        expect(dividendElements.length).toBeGreaterThan(0);
        // 「税額」「受取金額」は集計情報にも表示されるためgetAllByTextを使用
        expect(screen.getAllByText('税額').length).toBeGreaterThan(0);
        expect(screen.getByText('受取額')).toBeInTheDocument();
    });

    it('CSVデータが正しく表示される', () => {
        render(<Dividend data={mockData} />);

        // データの内容確認
        expect(screen.getByText('1234')).toBeInTheDocument();
        expect(screen.getByText('テスト株式1')).toBeInTheDocument();
        expect(screen.getByText('5678')).toBeInTheDocument();
        expect(screen.getByText('テスト株式2')).toBeInTheDocument();
    });

    it('銘柄名を右クリックすると銘柄名と銘柄コードをコピーする', () => {
        const writeText = vi.fn().mockResolvedValue(undefined);
        Object.defineProperty(navigator, 'clipboard', {
            value: { writeText },
            configurable: true,
        });

        render(<Dividend data={[{
            ...mockData[0],
            security_code: '9433',
            security_name: 'ＫＤＤＩ',
        }]} />);

        fireEvent.contextMenu(screen.getByText('ＫＤＤＩ'));

        expect(writeText).toHaveBeenCalledWith('ＫＤＤＩ(9433)');
    });

    it('検索オプションが正しく生成される', async () => {
        const { container } = render(<Dividend data={mockData} />);

        // 検索オプションヘッダーの確認
        const searchOptionsHeader = screen.getByText('検索オプション');
        expect(searchOptionsHeader).toBeInTheDocument();

        // SearchCardを展開してから銘柄検索セレクトボックスを確認
        const searchCardHeader = screen.getByTestId('search-card-header');
        fireEvent.click(searchCardHeader);

        // 銘柄検索セレクトボックスの確認（IDで指定）
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        });

        // オプションの確認（複数の「全て表示」があるためgetAllByTextを使用）
        const allOptions = screen.getAllByText('全て表示');
        expect(allOptions.length).toBeGreaterThanOrEqual(1);
        expect(screen.getByText('1234: テスト株式1')).toBeInTheDocument();
        expect(screen.getByText('5678: テスト株式2')).toBeInTheDocument();
    });

    it('銘柄を検索すると銘柄詳細ヘッダーが表示される', async () => {
        const user = userEvent.setup();
        const { container } = render(<Dividend data={mockData} />);

        // SearchCardを展開してから銘柄検索セレクトボックスを操作（IDで指定）
        const searchCardHeader = screen.getByTestId('search-card-header');
        fireEvent.click(searchCardHeader);
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        });
        const selectElement = container.querySelector('#securities-search') as HTMLSelectElement;
        await user.selectOptions(selectElement, '1234');

        // 銘柄詳細ヘッダーが表示されることを確認（初期状態は展開済み）
        await waitFor(() => {
            expect(screen.getByText('集計情報')).toBeInTheDocument();
        });

        // 展開済みなのでembeddedモードの入力とStatItemが表示される
        await waitFor(() => {
            expect(screen.getByText('平均取得価格')).toBeVisible();
            expect(screen.getByText('保有数量(株)')).toBeVisible();
            expect(screen.getByText('一株配当')).toBeVisible();
        });
        expect(screen.getByText('配当金額 (配当利回り)')).toBeVisible();
        expect(screen.getAllByText('税額').length).toBeGreaterThanOrEqual(2);
        expect(screen.getByText('受取金額 (累積利回り)')).toBeVisible();
    });

    it('銘柄詳細ヘッダーをクリックすると開閉できる', async () => {
        const user = userEvent.setup();
        const { container } = render(<Dividend data={mockData} />);

        // SearchCardを展開してから銘柄検索セレクトボックスを操作
        const searchCardHeader = screen.getByTestId('search-card-header');
        fireEvent.click(searchCardHeader);
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        });
        const selectElement = container.querySelector('#securities-search') as HTMLSelectElement;
        await user.selectOptions(selectElement, '1234');

        // 初期状態は展開済み
        await waitFor(() => {
            expect(screen.getByText('集計情報')).toBeInTheDocument();
        });
        expect(screen.getByText('平均取得価格')).toBeVisible();

        // クリックで折りたたみ
        const header = screen.getByTestId('receipt-header');
        await user.click(header);
        await waitFor(() => {
            expect(screen.getByText('平均取得価格')).not.toBeVisible();
        });

        // 再クリックで展開
        await user.click(header);
        await waitFor(() => {
            expect(screen.getByText('平均取得価格')).toBeVisible();
        });
    });

    it('検索をクリアすると通常のヘッダーに戻る', async () => {
        const user = userEvent.setup();
        const { container } = render(<Dividend data={mockData} />);

        // SearchCardを展開してから銘柄を選択（IDで指定）
        const searchCardHeader = screen.getByTestId('search-card-header');
        fireEvent.click(searchCardHeader);
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        });
        const selectElement = container.querySelector('#securities-search') as HTMLSelectElement;
        await user.selectOptions(selectElement, '1234');

        // 銘柄詳細ヘッダーが表示されることを確認
        await waitFor(() => {
            expect(screen.getByText('集計情報')).toBeInTheDocument();
        });

        // 検索をクリア
        await user.selectOptions(selectElement, '');

        // 通常の集計情報ヘッダーに戻ることを確認
        await waitFor(() => {
            expect(screen.getByText('集計情報')).toBeInTheDocument();
        });
    });

    it('空のデータでEmptyStateが表示される', () => {
        render(<Dividend data={[]} />);

        expect(screen.getByText('データがありません')).toBeInTheDocument();
        expect(screen.getByText('配当金明細をCSVで追加してください')).toBeInTheDocument();
    });

    it('数値フォーマットが正しく適用される', () => {
        render(<Dividend data={mockData} />);

        // 通貨フォーマットされた値が存在することを確認
        // 具体的な値は実装に依存するため、¥記号の存在を確認
        const currencyElements = screen.getAllByText(/¥/);
        expect(currencyElements.length).toBeGreaterThan(0);
    });

    it('レスポンシブテーブルが使用される', () => {
        const { container } = render(<Dividend data={mockData} />);

        // レスポンシブテーブルのクラスが適用されていることを確認
        const responsiveTable = container.querySelector('div.overflow-x-hidden');
        expect(responsiveTable).toBeInTheDocument();

        const table = screen.getByRole('table');
        expect(table).toBeInTheDocument();
        expect(table).toHaveClass('w-full', 'table-fixed');
    });

    it('銘柄選択後にread-onlyの集計情報が表示される', async () => {
        const user = userEvent.setup();
        const { container } = render(<Dividend data={mockData} />);

        // SearchCardを展開してから銘柄を選択（IDで指定）
        const searchCardHeader = screen.getByTestId('search-card-header');
        fireEvent.click(searchCardHeader);
        await waitFor(() => {
            expect(container.querySelector('#securities-search')).toBeInTheDocument();
        });
        const selectElement = container.querySelector('#securities-search') as HTMLSelectElement;
        await user.selectOptions(selectElement, '1234');

        // 銘柄詳細ヘッダーが表示されることを確認
        await waitFor(() => {
            expect(screen.getByText('集計情報')).toBeInTheDocument();
        });

        // embedded モードでは read-only KPI カードが表示される
        expect(screen.getByText('平均取得価格')).toBeVisible();
        expect(screen.getByText('保有数量(株)')).toBeVisible();
        expect(screen.getByText('一株配当')).toBeVisible();

        // 入力フォーム（spinbutton）は表示されない（embedded モードは read-only）
        expect(screen.queryAllByRole('spinbutton').length).toBe(0);
    });

    it('商品で検索すると商品名でグループ化される', async () => {
        const user = userEvent.setup();
        render(<Dividend data={mockData} />);

        await user.click(screen.getByTestId('search-card-header'));
        await user.click(await screen.findByRole('button', { name: '株式' }));

        await waitFor(() => {
            const summaryCell = screen.getByRole('table').querySelector('tbody tr td');
            expect(summaryCell).toHaveTextContent('株式');
            expect(summaryCell).toHaveTextContent('2件');
        });
    });

    it('口座で検索すると口座名でグループ化される', async () => {
        const user = userEvent.setup();
        render(<Dividend data={mockData} />);

        await user.click(screen.getByTestId('search-card-header'));
        await user.click(await screen.findByRole('button', { name: '特定口座' }));

        await waitFor(() => {
            const summaryCell = screen.getByRole('table').querySelector('tbody tr td');
            expect(summaryCell).toHaveTextContent('特定口座');
            expect(summaryCell).toHaveTextContent('2件');
        });
    });

    it('年で検索すると年月単位のグループが維持される', async () => {
        const user = userEvent.setup();
        const { container } = render(<Dividend data={mockData} />);

        const searchCardHeader = screen.getByTestId('search-card-header');
        fireEvent.click(searchCardHeader);
        await waitFor(() => {
            expect(container.querySelector('#years-search')).toBeInTheDocument();
        });

        const yearSelect = container.querySelector('#years-search') as HTMLSelectElement;
        await user.selectOptions(yearSelect, '2023');

        await waitFor(() => {
            expect(screen.getByText('2023年1月')).toBeInTheDocument();
            expect(screen.getByText('2023年2月')).toBeInTheDocument();
        });
    });

    it('年月検索クエリでは年月単位でグループ化される', () => {
        const useReceiptBaseDataSpy = vi.spyOn(receiptHooks, 'useReceiptBaseData').mockReturnValue({
            sortedData: mockData,
            searchQuery: '2023-01',
            setSearchQuery: vi.fn(),
            filteredData: [mockData[0]],
        } as ReturnType<typeof receiptHooks.useReceiptBaseData>);

        try {
            render(<Dividend data={mockData} />);

            expect(screen.getByText('2023年1月')).toBeInTheDocument();
            expect(screen.queryByText('2023年2月')).not.toBeInTheDocument();
        } finally {
            useReceiptBaseDataSpy.mockRestore();
        }
    });

    it('検索種別に一致しないクエリでも年月単位でグループ化される', () => {
        const useReceiptBaseDataSpy = vi.spyOn(receiptHooks, 'useReceiptBaseData').mockReturnValue({
            sortedData: mockData,
            searchQuery: '該当なし',
            setSearchQuery: vi.fn(),
            filteredData: [mockData[0]],
        } as ReturnType<typeof receiptHooks.useReceiptBaseData>);

        try {
            render(<Dividend data={mockData} />);

            expect(screen.getByText('2023年1月')).toBeInTheDocument();
            expect(screen.queryByText('2023年2月')).not.toBeInTheDocument();
        } finally {
            useReceiptBaseDataSpy.mockRestore();
        }
    });

    it('コード付きラベル形式の検索クエリでも銘柄詳細ヘッダーが表示される', async () => {
        const useReceiptBaseDataSpy = vi.spyOn(receiptHooks, 'useReceiptBaseData').mockReturnValue({
            sortedData: mockData,
            searchQuery: '1234: テスト株式1',
            setSearchQuery: vi.fn(),
            filteredData: [],
        } as ReturnType<typeof receiptHooks.useReceiptBaseData>);

        try {
            render(<Dividend data={mockData} />);

            await waitFor(() => {
                expect(screen.getByText('平均取得価格')).toBeVisible();
                expect(screen.getByText('一株配当')).toBeVisible();
                expect(screen.getByText('受取金額 (累積利回り)')).toBeVisible();
            });
        } finally {
            useReceiptBaseDataSpy.mockRestore();
        }
    });
});
