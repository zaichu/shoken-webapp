import { render, screen, fireEvent, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ReceiptTable } from '../ReceiptTable';
import type { TableColumnConfig, SummaryColumnConfig } from '@/features/receipt/types';

// ResizeObserver のモック（class形式で定義）
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
global.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;

describe('ReceiptTable', () => {
  const mockColumns: TableColumnConfig[] = [
    { header: '日付', key: 'date', width: '100px', textAlign: 'center' },
    { header: '銘柄', key: 'name', width: '200px', textAlign: 'left' },
    { header: '金額', key: 'amount', width: '120px', textAlign: 'right', format: (value) => `¥${(value as number).toLocaleString()}` },
  ];

  const mockSummaryColumns: SummaryColumnConfig[] = [
    { key: 'name', colSpan: 2, textAlign: 'right', format: () => '合計:' },
    { key: 'amount', colSpan: 1, textAlign: 'right', format: (value) => `¥${(value as number).toLocaleString()}` },
  ];

  const mockData = [
    { date: '2024-01-01', name: '銘柄A', amount: 1000, group: 'A' },
    { date: '2024-01-02', name: '銘柄B', amount: 2000, group: 'A' },
    { date: '2024-01-03', name: '銘柄C', amount: 3000, group: 'B' },
  ];

  const mockSummary = [
    { filter: 'A', amount: 3000, name: 'グループA' },
    { filter: 'B', amount: 3000, name: 'グループB' },
  ];

  const getGroupKey = (item: typeof mockData[0]) => item.group;

  // PCテーブル内のみを対象にするヘルパー（スマホカードとテキストが重複するため）
  const getTableQueries = () => within(screen.getByRole('table'));

  it('指定した銘柄と主要金額で読み上げ、EnterとSpaceで明細を開閉する', async () => {
    const user = userEvent.setup();
    render(<ReceiptTable data={mockData} summary={[]} columns={mockColumns}
      summaryColumns={[]} getGroupKey={getGroupKey} nameKey="name" primaryKey="amount" dateKey="date" />);
    const button = screen.getByRole('button', { name: '銘柄A ¥1,000' });
    expect(button).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByRole('region', { name: '銘柄A ¥1,000' })).not.toBeInTheDocument();
    button.focus();
    await user.keyboard('{Enter}');
    const region = screen.getByRole('region', { name: '銘柄A ¥1,000' });
    expect(region.id).toBe(button.getAttribute('aria-controls'));
    expect(within(region).getByText('日付')).toBeVisible();
    expect(button).toHaveFocus();
    await user.keyboard(' ');
    expect(button).toHaveAttribute('aria-expanded', 'false');
    expect(region).not.toBeVisible();
  });

  it('データが正しくレンダリングされる', () => {
    render(
      <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
        data={mockData}
        summary={mockSummary}
        columns={mockColumns}
        summaryColumns={mockSummaryColumns}
        getGroupKey={getGroupKey}
      />
    );

    const table = getTableQueries();
    // ヘッダー
    expect(table.getByText('日付')).toBeInTheDocument();
    expect(table.getByText('銘柄')).toBeInTheDocument();
    expect(table.getByText('金額')).toBeInTheDocument();

    // データ
    expect(table.getByText('銘柄A')).toBeInTheDocument();
    expect(table.getByText('銘柄B')).toBeInTheDocument();
    expect(table.getByText('銘柄C')).toBeInTheDocument();
    expect(table.getByText('¥1,000')).toBeInTheDocument();
    expect(table.getByText('¥2,000')).toBeInTheDocument();

    // ¥3,000は複数あるので、getAllByTextを使用
    const threeThousandElements = table.getAllByText('¥3,000');
    expect(threeThousandElements).toHaveLength(3); // データ1つ + サマリー2つ

    // サマリー
    expect(table.getAllByText('合計:')).toHaveLength(2);
  });

  it('HTMLタグが含まれる文字列はエスケープされてテキストとして表示される（XSS対策）', () => {
    const columnsWithHtml: TableColumnConfig[] = [
      ...mockColumns,
      {
        header: 'リンク',
        key: 'link',
        width: '100px',
        textAlign: 'center',
        format: (value) => `<a href="${value}">リンク</a>`
      },
    ];

    const dataWithHtml = mockData.map(item => ({ ...item, link: 'https://example.com' }));

    render(
      <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
        data={dataWithHtml}
        summary={mockSummary}
        columns={columnsWithHtml}
        summaryColumns={mockSummaryColumns}
        getGroupKey={getGroupKey}
      />
    );

    const table = getTableQueries();
    // ヘッダーの「リンク」のみ表示
    expect(table.getByText('リンク')).toBeInTheDocument();

    // HTML文字列はリンクとしてレンダリングされず、テキストとして表示される
    expect(screen.queryAllByRole('link')).toHaveLength(0);

    // エスケープされたHTML文字列がテキストとして表示される
    expect(table.getAllByText('<a href="https://example.com">リンク</a>')).toHaveLength(3);
  });

  it('負の値のセルにはdata-negative属性が付与される', () => {
    const dataWithNegative = [
      { date: '2024-01-01', name: '銘柄A', amount: -1200, group: 'A' },
    ];
    const summaryWithNegative = [
      { filter: 'A', amount: -1200, name: 'グループA' },
    ];

    render(
      <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
        data={dataWithNegative}
        summary={summaryWithNegative}
        columns={mockColumns}
        summaryColumns={mockSummaryColumns}
        getGroupKey={getGroupKey}
      />
    );

    const table = getTableQueries();
    const negativeCells = table.getAllByText('¥-1,200').map((element) => element.closest('td'));
    expect(negativeCells).toHaveLength(2); // 明細行 + サマリー行
    negativeCells.forEach((cell) => {
      expect(cell).toHaveAttribute('data-negative', 'true');
    });
  });

  it('空のデータでも正しくレンダリングされる', () => {
    render(
      <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
        data={[]}
        summary={[]}
        columns={mockColumns}
        summaryColumns={mockSummaryColumns}
        getGroupKey={getGroupKey}
      />
    );

    const table = getTableQueries();
    // ヘッダーは表示される
    expect(table.getByText('日付')).toBeInTheDocument();
    expect(table.getByText('銘柄')).toBeInTheDocument();
    expect(table.getByText('金額')).toBeInTheDocument();

    // データ行は表示されない
    expect(table.queryByText('銘柄A')).not.toBeInTheDocument();
    // カードも表示されない
    expect(screen.queryByTestId('receipt-card')).not.toBeInTheDocument();
  });

  it('nullやundefinedやbooleanの値が適切にレンダリングされる', () => {
    const columnsWithBoolean: TableColumnConfig[] = [
      { header: '日付', key: 'date', width: '100px', textAlign: 'center' },
      { header: 'フラグ', key: 'flag', width: '80px', textAlign: 'center' }, // no format, boolean value
    ];

    const dataWithMixedTypes = [
      { date: null, flag: true, group: 'A' },
      { date: undefined, flag: false, group: 'A' },
    ];

    render(
      <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
        data={dataWithMixedTypes as unknown as typeof mockData}
        summary={[{ filter: 'A', amount: 0, name: 'グループA' }]}
        columns={columnsWithBoolean}
        summaryColumns={mockSummaryColumns}
        getGroupKey={(item) => (item as { group: string }).group}
      />
    );

    const table = getTableQueries();
    // true → 'true', false → 'false' が表示される
    expect(table.getByText('true')).toBeInTheDocument();
    expect(table.getByText('false')).toBeInTheDocument();
    expect(table.queryByText('null')).not.toBeInTheDocument();
    expect(table.queryByText('undefined')).not.toBeInTheDocument();
  });

  it('4桁年グループキーが「YYYY年」形式でフォーマットされる', () => {
    const dataWithYear = [
      { date: '2024-01-01', name: '銘柄A', amount: 1000, group: '2024' },
    ];
    const summaryWithYear = [
      { filter: '2024', amount: 1000, name: '2024' },
    ];

    render(
      <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
        data={dataWithYear}
        summary={summaryWithYear}
        columns={mockColumns}
        summaryColumns={mockSummaryColumns}
        getGroupKey={(item) => (item as { group: string }).group}
      />
    );

    expect(getTableQueries().getByText('2024年')).toBeInTheDocument();
  });

  it('security-code-linkクリックでonSearchが呼ばれる', () => {
    const mockOnSearch = vi.fn();

    const { container } = render(
      <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
        data={mockData}
        summary={mockSummary}
        columns={mockColumns}
        summaryColumns={mockSummaryColumns}
        getGroupKey={getGroupKey}
        onSearch={mockOnSearch}
      />
    );

    // テーブルを取得してsecurity-code-link要素を動的に作成してクリックをシミュレート
    const table = container.querySelector('table');
    expect(table).not.toBeNull();
    const td = table!.querySelector('td');
    expect(td).not.toBeNull();

    const linkEl = document.createElement('span');
    linkEl.classList.add('security-code-link');
    linkEl.dataset.search = '1234';
    td!.appendChild(linkEl);
    fireEvent.click(linkEl);
    expect(mockOnSearch).toHaveBeenCalledWith('1234');
  });

  it('グループごとにデータが正しくフィルタリングされる', () => {
    render(
      <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
        data={mockData}
        summary={mockSummary}
        columns={mockColumns}
        summaryColumns={mockSummaryColumns}
        getGroupKey={getGroupKey}
      />
    );

    // 実装ではサマリー行が先に表示され、その後にデータ行が続く
    const table = screen.getByRole('table');
    const rows = table.querySelectorAll('tbody tr');

    // グループAのサマリーが最初
    expect(rows[0]).toHaveTextContent('A');
    expect(rows[0]).toHaveTextContent('2件');
    expect(rows[0]).toHaveTextContent('¥3,000');

    // グループAのデータ行
    expect(rows[1]).toHaveTextContent('銘柄A');
    expect(rows[2]).toHaveTextContent('銘柄B');

    // グループBのサマリー
    expect(rows[3]).toHaveTextContent('B');
    expect(rows[3]).toHaveTextContent('1件');
    expect(rows[3]).toHaveTextContent('¥3,000');

    // グループBのデータ行
    expect(rows[4]).toHaveTextContent('銘柄C');
  });

  describe('スマホカード表示（sm未満）', () => {
    it('PCテーブルとスマホカードの両方がレンダリングされ、出し分けクラスが付く', () => {
      const { container } = render(
        <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
          data={mockData}
          summary={mockSummary}
          columns={mockColumns}
          summaryColumns={mockSummaryColumns}
          getGroupKey={getGroupKey}
        />
      );

      const tableWrapper = screen.getByRole('table').parentElement?.parentElement;
      expect(tableWrapper?.className).toContain('hidden');

      const cardList = screen.getByTestId('receipt-card-list');
      expect(cardList.className).toContain('sm:hidden');
      expect(container.querySelector('table')).not.toBeNull();
    });

    it('カードに全列のラベルと値が省略なく含まれる', () => {
      render(
        <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
          data={mockData}
          summary={mockSummary}
          columns={mockColumns}
          summaryColumns={mockSummaryColumns}
          getGroupKey={getGroupKey}
        />
      );

      const cardList = screen.getByTestId('receipt-card-list');
      const cardQueries = within(cardList);
      // 全3件分のカード
      expect(cardQueries.getAllByTestId('receipt-card')).toHaveLength(3);
      // 折り畳み時は詳細を描画せず、展開後に全列を表示する
      expect(cardQueries.queryByText('日付')).not.toBeInTheDocument();
      fireEvent.click(cardQueries.getByRole('button', { name: '銘柄A ¥1,000' }));
      // 各列のラベル（header）がカード内に存在する
      expect(cardQueries.getAllByText('日付').length).toBeGreaterThan(0);
      expect(cardQueries.getAllByText('銘柄').length).toBeGreaterThan(0);
      expect(cardQueries.getAllByText('金額').length).toBeGreaterThan(0);
      // 値も省略なく含まれる
      expect(within(cardQueries.getByRole('region', { name: '銘柄A ¥1,000' })).getByText('銘柄A')).toBeVisible();
      expect(within(cardQueries.getByRole('region', { name: '銘柄A ¥1,000' })).getByText('¥1,000')).toBeVisible();
    });

    it('グループ見出しは1行に集約され、展開すると集計値を確認できる', () => {
      render(
        <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
          data={mockData}
          summary={mockSummary}
          columns={mockColumns}
          summaryColumns={mockSummaryColumns}
          getGroupKey={getGroupKey}
        />
      );

      const cardList = screen.getByTestId('receipt-card-list');
      const groups = within(cardList).getAllByTestId('receipt-card-group');
      expect(groups).toHaveLength(2);
      // 折り畳み時: 件数と主要集計（金額）のみ表示し、明細ラベルは隠す
      expect(groups[0]).toHaveTextContent('2件');
      expect(groups[0]).toHaveTextContent('¥3,000');
      const toggle = within(groups[0]).getByRole('button', { name: /2件/ });
      const collapsedRegion = groups[0].querySelector(`#${CSS.escape(toggle.getAttribute('aria-controls') ?? '')}`);
      expect(collapsedRegion).not.toBeVisible();
      // 展開すると全集計を確認できる
      fireEvent.click(toggle);
      expect(collapsedRegion).toBeVisible();
      expect(collapsedRegion).toHaveTextContent('¥3,000');
      expect(groups[1]).toHaveTextContent('1件');
    });

    it('グループ見出しはbutton+aria-expanded/aria-controlsでEnterとSpaceで開閉する', async () => {
      const user = userEvent.setup();
      render(
        <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
          data={mockData}
          summary={mockSummary}
          columns={mockColumns}
          summaryColumns={mockSummaryColumns}
          getGroupKey={getGroupKey}
        />
      );

      const cardList = screen.getByTestId('receipt-card-list');
      const groups = within(cardList).getAllByTestId('receipt-card-group');
      const button = within(groups[0]).getByRole('button', { name: /2件/ });
      // タップターゲット44px以上
      expect(button.className).toContain('min-h-[44px]');
      expect(button).toHaveAttribute('aria-expanded', 'false');
      button.focus();
      await user.keyboard('{Enter}');
      expect(button).toHaveAttribute('aria-expanded', 'true');
      const controlsId = button.getAttribute('aria-controls') ?? '';
      const region = groups[0].querySelector(`#${CSS.escape(controlsId)}`);
      expect(region).toHaveAttribute('role', 'region');
      expect(region).toBeVisible();
      expect(button).toHaveFocus();
      await user.keyboard(' ');
      expect(button).toHaveAttribute('aria-expanded', 'false');
      expect(region).not.toBeVisible();
    });

    it('グループ見出しの主要集計はprimaryKeyに対応する列を表示する', () => {
      const columnsWithProfit: TableColumnConfig[] = [
        { header: '銘柄', key: 'name', width: '200px' },
        { header: '損益', key: 'realized_profit_and_loss', width: '120px', textAlign: 'right', format: (value) => `¥${(value as number).toLocaleString()}` },
        { header: '税額', key: 'taxes', width: '120px', textAlign: 'right', format: (value) => `¥${(value as number).toLocaleString()}` },
      ];
      const summaryWithTotal: SummaryColumnConfig[] = [
        { key: 'total_realized_profit_and_loss', textAlign: 'right', format: (value) => `¥${(value as number).toLocaleString()}` },
        { key: 'total_taxes', textAlign: 'right', format: (value) => `¥${(value as number).toLocaleString()}` },
      ];
      const summaryData = [{ filter: 'A', total_realized_profit_and_loss: 30000, total_taxes: 6090 }];
      const dataWithProfit = mockData.map((item, index) => ({
        ...item,
        realized_profit_and_loss: [10000, 20000, 30000][index],
        taxes: [2000, 3000, 6090][index],
      }));

      render(
        <ReceiptTable
        primaryKey="realized_profit_and_loss" nameKey="name" dateKey="date"
          data={dataWithProfit}
          summary={summaryData}
          columns={columnsWithProfit}
          summaryColumns={summaryWithTotal}
          getGroupKey={getGroupKey}
        />
      );

      const cardList = screen.getByTestId('receipt-card-list');
      // total_プレフィックス差異を吸収し、損益の値を1行に表示する
      const button = within(cardList).getByRole('button', { name: /損益 ¥30,000/ });
      expect(button).toHaveAttribute('aria-expanded', 'false');
      fireEvent.click(button);
      // 展開すると税額も確認できる
      const region = within(cardList).getByRole('region');
      expect(within(region).getByText('¥6,090')).toBeVisible();
    });

    it('total_プレフィックス付き集計キーのラベルが列定義から解決される', () => {
      const columnsWithProfit: TableColumnConfig[] = [
        { header: '銘柄', key: 'name', width: '200px' },
        { header: '損益', key: 'realized_profit_and_loss', width: '120px', textAlign: 'right', format: (value) => `¥${(value as number).toLocaleString()}` },
      ];
      const summaryWithTotal: SummaryColumnConfig[] = [
        { key: 'total_realized_profit_and_loss', textAlign: 'right', format: (value) => `¥${(value as number).toLocaleString()}` },
      ];
      const summaryData = [{ filter: 'A', total_realized_profit_and_loss: 30000 }];
      const dataWithProfit = mockData.map((item, index) => ({
        ...item,
        realized_profit_and_loss: [10000, 20000, 30000][index],
      }));

      render(
        <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
          data={dataWithProfit}
          summary={summaryData}
          columns={columnsWithProfit}
          summaryColumns={summaryWithTotal}
          getGroupKey={getGroupKey}
        />
      );

      const cardList = screen.getByTestId('receipt-card-list');
      // 集計ラベルが列定義のheader「損益」で表示される
      expect(within(cardList).getAllByText('損益').length).toBeGreaterThan(0);
      expect(within(cardList).getByText('¥30,000')).toBeInTheDocument();
    });

    it('カードの負の主要金額を濃い赤で表示する', () => {
      const dataWithNegative = [
        { date: '2024-01-01', name: '銘柄A', amount: -1200, group: 'A' },
      ];
      const summaryWithNegative = [
        { filter: 'A', amount: -1200, name: 'グループA' },
      ];

      render(
        <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
          data={dataWithNegative}
          summary={summaryWithNegative}
          columns={mockColumns}
          summaryColumns={mockSummaryColumns}
          getGroupKey={getGroupKey}
        />
      );

      const cardList = screen.getByTestId('receipt-card-list');
      const button = within(cardList).getByRole('button', { name: '銘柄A ¥-1,200' });
      expect(within(button).getByText('¥-1,200')).toHaveClass('text-red-800');
    });

    it('カード内のsecurity-code-linkクリックでonSearchが呼ばれる', () => {
      const mockOnSearch = vi.fn();

      render(
        <ReceiptTable
        primaryKey="amount" nameKey="name" dateKey="date"
          data={mockData}
          summary={mockSummary}
          columns={mockColumns}
          summaryColumns={mockSummaryColumns}
          getGroupKey={getGroupKey}
          onSearch={mockOnSearch}
        />
      );

      const cardList = screen.getByTestId('receipt-card-list');
      const linkEl = document.createElement('span');
      linkEl.classList.add('security-code-link');
      linkEl.dataset.search = '5678';
      cardList.appendChild(linkEl);
      fireEvent.click(linkEl);
      expect(mockOnSearch).toHaveBeenCalledWith('5678');
    });
  });
});
