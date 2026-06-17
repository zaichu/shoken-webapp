import { fireEvent, render, screen } from '@testing-library/react';
import { ReceiptHeader } from '../ReceiptHeader';

describe('ReceiptHeader', () => {
  const defaultItems = [
    {
      title: 'テスト項目1',
      value: 1000,
      format: (value: number) => `¥${value.toLocaleString()}`,
    },
    {
      title: 'テスト項目2',
      value: 50,
      format: (value: number) => `${value}件`,
    },
  ];

  it('正しくレンダリングされる', () => {
    render(<ReceiptHeader items={defaultItems} />);
    
    expect(screen.getByText('集計情報')).toBeInTheDocument();
    expect(screen.getByText('テスト項目1')).toBeInTheDocument();
    expect(screen.getByText('¥1,000')).toBeInTheDocument();
    expect(screen.getByText('テスト項目2')).toBeInTheDocument();
    expect(screen.getByText('50件')).toBeInTheDocument();
  });

  it('空の配列でも正しくレンダリングされる', () => {
    render(<ReceiptHeader items={[]} />);
    
    expect(screen.getByText('集計情報')).toBeInTheDocument();
  });

  it('itemsの配列数に応じて適切な数の項目がレンダリングされる', () => {
    const items = [
      ...defaultItems,
      {
        title: 'テスト項目3',
        value: 200,
        format: (value: number) => `${value}%`,
      },
    ];

    const { container } = render(<ReceiptHeader items={items} />);
    
    const columns = container.querySelectorAll('[data-testid="kpi-grid"] > div');
    expect(columns).toHaveLength(3);
  });

  it('異なるフォーマット関数が正しく適用される', () => {
    const items = [
      {
        title: '通貨',
        value: 1500,
        format: (value: number) => `¥${value}`,
      },
      {
        title: 'パーセント',
        value: 75,
        format: (value: number) => `${value}%`,
      },
      {
        title: '数量',
        value: 10,
        format: (value: number) => `${value}個`,
      },
    ];

    render(<ReceiptHeader items={items} />);

    expect(screen.getByText('¥1500')).toBeInTheDocument();
    expect(screen.getByText('75%')).toBeInTheDocument();
    expect(screen.getByText('10個')).toBeInTheDocument();
  });

  it('タイトルと追加コンテンツを表示できる', () => {
    render(
      <ReceiptHeader items={defaultItems} title="集計・配当情報">
        <div>配当情報</div>
      </ReceiptHeader>
    );

    expect(screen.getByText('集計・配当情報')).toBeInTheDocument();
    expect(screen.getByText('配当情報')).toBeInTheDocument();
  });

  it('collapsibleがtrueのときヘッダークリックで開閉できる', () => {
    render(<ReceiptHeader items={defaultItems} title="配当情報" collapsible />);

    const header = screen.getByTestId('receipt-header');
    expect(screen.getByText('テスト項目1')).toBeVisible();

    fireEvent.click(header);
    expect(screen.getByText('テスト項目1')).not.toBeVisible();

    fireEvent.click(header);
    expect(screen.getByText('テスト項目1')).toBeVisible();
  });

  // --- 展開状態仕様（保持 vs 初期化）の明文化 ---

  it('collapsible=falseのときコンテンツは常に表示され折りたたみ不可', () => {
    // 仕様: collapsible=false の場合、effectiveExpanded は常に true
    render(<ReceiptHeader items={defaultItems} collapsible={false} />);

    expect(screen.getByText('テスト項目1')).toBeVisible();
    // role="button" が付与されないのでクリックしても状態変化なし
    const header = screen.getByTestId('receipt-header');
    expect(header).not.toHaveAttribute('role', 'button');
  });

  it('defaultExpanded=falseのとき初期状態が折りたたまれる', () => {
    // 仕様: collapsible=true && defaultExpanded=false → 初期は閉じた状態
    render(<ReceiptHeader items={defaultItems} collapsible defaultExpanded={false} />);

    expect(screen.getByText('テスト項目1')).not.toBeVisible();
  });

  it('collapsibleがfalse→trueに切り替わっても展開状態（isExpanded）が保持される', () => {
    // 仕様: useEffect によるリセットを廃止したため、
    //        collapsible prop の変化ではなくユーザー操作のみが isExpanded を変える。
    //        collapsible=false 時は effectiveExpanded=true（常時表示）、
    //        collapsible=true に切り替わると isExpanded の前回値をそのまま使う。
    const { rerender } = render(
      <ReceiptHeader items={defaultItems} collapsible={false} />
    );
    expect(screen.getByText('テスト項目1')).toBeVisible();

    // collapsible=true に変更（初期 isExpanded は true のまま）
    rerender(<ReceiptHeader items={defaultItems} collapsible={true} />);
    // isExpanded=true が保持されるため引き続き表示
    expect(screen.getByText('テスト項目1')).toBeVisible();
  });

  it('ユーザーが折りたたんだ後、collapsibleをfalse→trueに切り替えても折りたたみ状態が保持される', () => {
    // 仕様: ユーザーによる折りたたみ操作は isExpanded に永続される。
    //        collapsible=false 期間は effectiveExpanded=true で表示されるが、
    //        再び collapsible=true になると以前の isExpanded=false が復活する。
    const { rerender } = render(
      <ReceiptHeader items={defaultItems} collapsible={true} />
    );
    expect(screen.getByText('テスト項目1')).toBeVisible();

    // ユーザーが折りたたむ
    fireEvent.click(screen.getByTestId('receipt-header'));
    expect(screen.getByText('テスト項目1')).not.toBeVisible();

    // collapsible=false に変更 → effectiveExpanded=true で強制表示
    rerender(<ReceiptHeader items={defaultItems} collapsible={false} />);
    expect(screen.getByText('テスト項目1')).toBeVisible();

    // collapsible=true に戻す → isExpanded=false が復活し再び折りたたまれる
    rerender(<ReceiptHeader items={defaultItems} collapsible={true} />);
    expect(screen.getByText('テスト項目1')).not.toBeVisible();
  });

  it('AssetPortfolioSummaryと同じ横方向のKPIグリッドで表示される', () => {
    const { container } = render(<ReceiptHeader items={defaultItems} />);

    expect(screen.getByTestId('receipt-summary-strip')).toBeInTheDocument();
    const grid = container.querySelector('[data-testid="kpi-grid"]');
    expect(grid).toHaveClass('grid-cols-1');
    expect(grid).toHaveClass('sm:grid-cols-2');
    expect(grid).toHaveClass('xl:grid-cols-3');
  });

  it('コンテナの角丸・シャドウ・背景がAssetPortfolioSummaryのKPIストリップと一致する', () => {
    render(<ReceiptHeader items={defaultItems} />);

    const strip = screen.getByTestId('receipt-summary-strip');
    expect(strip).toHaveClass('rounded-xl', 'border-slate-950/10', 'bg-white/95');
  });

  it('KPIカードの角丸・背景がAssetPortfolioSummaryのKPIカードと一致する', () => {
    const { container } = render(<ReceiptHeader items={defaultItems} />);

    const card = container.querySelector('[data-testid="kpi-grid"] > div');
    expect(card).toHaveClass('rounded-lg', 'border', 'bg-white');
  });

  it('マイナス値のitemにdata-negative属性が付く', () => {
    const itemsWithNegative = [
      {
        title: '損益',
        value: -500,
        format: (value: number) => `¥${value}`,
      },
    ];

    const { container } = render(<ReceiptHeader items={itemsWithNegative} />);

    const valueEl = container.querySelector('[data-negative="true"]');
    expect(valueEl).toBeInTheDocument();
    expect(valueEl).toHaveTextContent('¥-500');
  });
});
