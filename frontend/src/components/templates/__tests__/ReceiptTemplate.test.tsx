import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { vi, describe, test, expect, beforeEach } from 'vitest';
import '@testing-library/jest-dom';
import { ReceiptTemplate } from '../ReceiptTemplate';

// モック用のReceiptTableコンポーネント
const MockReceiptTable = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  (props, ref) => (
    <div
      ref={ref}
      data-testid="mock-receipt-table"
      {...props}
    >
      Mock Receipt Table
    </div>
  )
);
MockReceiptTable.displayName = 'ReceiptTable';

// rail内state保持テスト用のstatefulプローブ
const StatefulProbe: React.FC = () => {
  const [value, setValue] = React.useState('');
  return (
    <input
      data-testid="state-probe"
      value={value}
      onChange={(e) => setValue(e.target.value)}
    />
  );
};

// SearchCardのモック
vi.mock('@/components/organisms/SearchCard/SearchCard', () => ({
  SearchCard: ({ onSearch, onExpandToggle, initialExpanded }: {
    onSearch: (query: string) => void;
    onExpandToggle?: (isExpanded: boolean) => void;
    initialExpanded?: boolean;
  }) => (
    <div data-testid="search-card" data-initial-expanded={String(initialExpanded)}>
      <button onClick={() => onSearch('test-query')}>Search</button>
      <button onClick={() => onExpandToggle?.(true)}>Expand</button>
      <button onClick={() => onExpandToggle?.(false)}>Collapse</button>
    </div>
  )
}));

describe('ReceiptTemplate', () => {
  const mockOnSearch = vi.fn();
  const mockOnSearchExpandToggle = vi.fn();

  const defaultProps = {
    children: <MockReceiptTable />
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  test('基本的なレイアウトが正しくレンダリングされる', () => {
    render(<ReceiptTemplate {...defaultProps} />);

    expect(screen.getByTestId('receipt-container')).toBeInTheDocument();
    expect(screen.getByTestId('mock-receipt-table')).toBeInTheDocument();
  });

  test('検索機能がある場合SearchCardが表示される', () => {
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
        searchCategories={{
          securities: [{ value: 'AAPL', label: 'Apple' }],
          products: ['株式'],
          accounts: ['一般口座'],
          years: [{ value: '2024', label: '2024年' }]
        }}
      />
    );

    expect(screen.getByTestId('search-card')).toBeInTheDocument();
  });

  test('検索機能がない場合SearchCardが表示されない', () => {
    render(<ReceiptTemplate {...defaultProps} />);

    expect(screen.queryByTestId('search-card')).not.toBeInTheDocument();
  });

  test('ヘッダーが提供された場合表示される', () => {
    const headerContent = <div data-testid="header-content">カスタムヘッダー</div>;
    
    render(
      <ReceiptTemplate 
        {...defaultProps}
        header={headerContent}
      />
    );

    expect(screen.getByTestId('header-content')).toBeInTheDocument();
    expect(screen.getByText('カスタムヘッダー')).toBeInTheDocument();
  });

  test('フッターが提供された場合表示される', () => {
    const footerContent = <div data-testid="footer-content">カスタムフッター</div>;
    
    render(
      <ReceiptTemplate 
        {...defaultProps}
        footer={footerContent}
      />
    );

    expect(screen.getByTestId('footer-content')).toBeInTheDocument();
    expect(screen.getByText('カスタムフッター')).toBeInTheDocument();
  });

  test('SearchCard展開時にonSearchExpandToggleが呼ばれる', () => {
    render(
      <ReceiptTemplate
        {...defaultProps}
        onSearch={mockOnSearch}
        onSearchExpandToggle={mockOnSearchExpandToggle}
      />
    );

    const expandButton = screen.getByText('Expand');
    fireEvent.click(expandButton);

    expect(mockOnSearchExpandToggle).toHaveBeenCalledWith(true);
  });

  test('SearchCard折りたたみ時にonSearchExpandToggleが呼ばれる', () => {
    render(
      <ReceiptTemplate
        {...defaultProps}
        onSearch={mockOnSearch}
        onSearchExpandToggle={mockOnSearchExpandToggle}
      />
    );

    const collapseButton = screen.getByText('Collapse');
    fireEvent.click(collapseButton);

    expect(mockOnSearchExpandToggle).toHaveBeenCalledWith(false);
  });

  test('複数の子要素がある場合も子要素をそのまま描画する', () => {
    const NonReceiptTableChild = () => <div data-testid="non-receipt-table">Other Component</div>;
    
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
      >
        <MockReceiptTable />
        <NonReceiptTableChild />
      </ReceiptTemplate>
    );

    const receiptTable = screen.getByTestId('mock-receipt-table');
    const otherComponent = screen.getByTestId('non-receipt-table');
    
    expect(receiptTable).toBeInTheDocument();
    expect(otherComponent).toBeInTheDocument();
  });

  test('ReceiptTableがない場合でもエラーが発生しない', () => {
    const NonReceiptTableChild = () => <div data-testid="non-receipt-table">Other Component</div>;
    
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
      >
        <NonReceiptTableChild />
      </ReceiptTemplate>
    );

    expect(screen.getByTestId('non-receipt-table')).toBeInTheDocument();
    
    // SearchCard操作でもエラーが発生しない
    const expandButton = screen.getByText('Expand');
    fireEvent.click(expandButton);
    
    expect(screen.getByTestId('non-receipt-table')).toBeInTheDocument();
  });

  test('searchCategoriesが部分的に提供された場合正しく処理される', () => {
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
        searchCategories={{
          securities: [{ value: 'AAPL', label: 'Apple' }],
          // 他のプロパティは未定義
        }}
      />
    );

    expect(screen.getByTestId('search-card')).toBeInTheDocument();
  });

  test('onSearchExpandToggleが提供されない場合でもエラーが発生しない', () => {
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
      />
    );

    // SearchCard操作でもエラーが発生しない
    const expandButton = screen.getByText('Expand');
    fireEvent.click(expandButton);
    
    expect(screen.getByTestId('mock-receipt-table')).toBeInTheDocument();
  });

  test('カードのレイアウト構造が正しい', () => {
    render(<ReceiptTemplate {...defaultProps} />);

    // メインカードの存在確認
    const mainCard = screen.getByTestId('receipt-card');
    expect(mainCard).toBeInTheDocument();

    // ボディの存在確認
    const cardBody = screen.getByTestId('receipt-card-body');
    expect(cardBody).toBeInTheDocument();
  });

  test('receipt-containerクラスが適用される', () => {
    render(<ReceiptTemplate {...defaultProps} />);
    const receiptContainer = screen.getByTestId('receipt-container');
    expect(receiptContainer).toBeInTheDocument();
  });

  test('utilityRailのみ指定時は自動的にworkspaceレイアウトになる', () => {
    const railTools = <div data-testid="auto-rail">CSV操作</div>;

    render(
      <ReceiptTemplate
        {...defaultProps}
        utilityRail={railTools}
      />
    );

    // layoutPropなしでもworkspaceレイアウトが選択される
    expect(screen.getByTestId('receipt-workspace')).toBeInTheDocument();
    expect(screen.getByTestId('auto-rail')).toBeInTheDocument();
  });

  test('workspaceレイアウトでフッターが表示される', () => {
    const footerContent = <div data-testid="workspace-footer">フッター</div>;
    const railTools = <div data-testid="rail">サイド</div>;

    render(
      <ReceiptTemplate
        {...defaultProps}
        layout="workspace"
        utilityRail={railTools}
        footer={footerContent}
      />
    );

    expect(screen.getByTestId('workspace-footer')).toBeInTheDocument();
  });

  test('workspaceレイアウトではヘッダーがmain stage上部に配置される', () => {
    const headerContent = <div data-testid="workspace-header">集計ヘッダー</div>;
    const railTools = <div data-testid="workspace-tools">CSV操作</div>;

    render(
      <ReceiptTemplate
        {...defaultProps}
        layout="workspace"
        onSearch={mockOnSearch}
        header={headerContent}
        utilityRail={railTools}
      />
    );

    expect(screen.getByTestId('receipt-workspace')).toBeInTheDocument();
    expect(screen.getByTestId('receipt-utility-rail')).toBeInTheDocument();
    expect(screen.getByTestId('receipt-workspace').className).toContain('lg:grid-cols-[minmax(0,1fr)_19rem]');
    expect(screen.getByTestId('receipt-workspace').className).toContain('xl:grid-cols-[minmax(0,1fr)_20rem]');

    const utilityRail = screen.getByTestId('receipt-utility-rail');
    expect(utilityRail).toContainElement(screen.getByTestId('workspace-tools'));
    expect(utilityRail).toContainElement(screen.getByTestId('search-card'));
    expect(utilityRail).not.toContainElement(screen.getByTestId('workspace-header'));

    const mainStage = screen.getByTestId('receipt-main-stage');
    expect(mainStage).toContainElement(screen.getByTestId('workspace-header'));
    expect(mainStage).toContainElement(screen.getByTestId('mock-receipt-table'));
  });

  test('workspaceレイアウトではrailがmainより前にDOM配置される（Issue #839）', () => {
    const headerContent = <div data-testid="workspace-header">集計ヘッダー</div>;
    const railTools = <div data-testid="workspace-tools">CSV操作</div>;

    render(
      <ReceiptTemplate
        {...defaultProps}
        layout="workspace"
        onSearch={mockOnSearch}
        header={headerContent}
        utilityRail={railTools}
      />
    );

    // DOM順自体が rail → main。Tab順・読み上げ順が視覚順（検索が先）と一致する
    const utilityRail = screen.getByTestId('receipt-utility-rail');
    const mainStage = screen.getByTestId('receipt-main-stage');
    expect(utilityRail.compareDocumentPosition(mainStage) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();

    // lg以上では order で main を左列・rail を右列に戻す（PC表示は不変）
    expect(utilityRail).toHaveClass('order-1', 'lg:order-2');
    expect(mainStage).toHaveClass('order-2', 'lg:order-1');

    // 検索カードは単一インスタンスのまま rail 内に残る（DOM二重化なし）
    expect(screen.getAllByTestId('search-card')).toHaveLength(1);
    expect(utilityRail).toContainElement(screen.getByTestId('search-card'));
  });

  test('再レンダー・リサイズ後もrail内のstateが保持される（Issue #839・単一インスタンス）', () => {
    const { rerender } = render(
      <ReceiptTemplate
        {...defaultProps}
        layout="workspace"
        onSearch={mockOnSearch}
        utilityRail={<StatefulProbe />}
      />
    );

    const probe = screen.getByTestId('state-probe');
    fireEvent.change(probe, { target: { value: 'キープ' } });
    expect(probe).toHaveValue('キープ');

    // ブレークポイント跨ぎを模したリサイズ＋再レンダーでも同一ノード・state維持。
    // 条件レンダリング（matchMediaでマウント位置切替）なら再マウントで失われる。
    window.dispatchEvent(new Event('resize'));
    rerender(
      <ReceiptTemplate
        {...defaultProps}
        layout="workspace"
        onSearch={mockOnSearch}
        utilityRail={<StatefulProbe />}
      />
    );

    const probeAfter = screen.getByTestId('state-probe');
    expect(probeAfter).toBe(probe);
    expect(probeAfter).toHaveValue('キープ');
    expect(screen.getAllByTestId('search-card')).toHaveLength(1);
  });

  test('stackレイアウトではorderクラスを付けない', () => {
    render(
      <ReceiptTemplate
        {...defaultProps}
        onSearch={mockOnSearch}
      />
    );

    expect(screen.queryByTestId('receipt-workspace')).not.toBeInTheDocument();
    expect(screen.getByTestId('search-card')).toBeInTheDocument();
  });

  test('workspaceレイアウトでは検索は初期展開される（PC幅・Issue #837 PR2）', () => {
    render(
      <ReceiptTemplate
        {...defaultProps}
        layout="workspace"
        onSearch={mockOnSearch}
        utilityRail={<div>CSV操作</div>}
      />
    );

    // jsdom には matchMedia がないため、PC幅相当として初期展開のまま
    expect(screen.getByTestId('search-card')).toHaveAttribute('data-initial-expanded', 'true');
  });

  test('スマホ幅 (<640px) では検索は折り畳み入口から始める（Issue #837 PR2）', () => {
    const matchMedia = vi.fn().mockReturnValue({ matches: true });
    vi.stubGlobal('matchMedia', matchMedia);
    // window.matchMedia として参照されるよう window にも設定する
    Object.defineProperty(window, 'matchMedia', { value: matchMedia, configurable: true });
    try {
      render(
        <ReceiptTemplate
          {...defaultProps}
          layout="workspace"
          onSearch={mockOnSearch}
          utilityRail={<div>CSV操作</div>}
        />
      );

      expect(matchMedia).toHaveBeenCalledWith('(max-width: 639px)');
      expect(screen.getByTestId('search-card')).toHaveAttribute('data-initial-expanded', 'false');
    } finally {
      vi.unstubAllGlobals();
    }
  });

  test('640px以上では検索は初期展開のまま（Issue #837 PR2）', () => {
    const matchMedia = vi.fn().mockReturnValue({ matches: false });
    Object.defineProperty(window, 'matchMedia', { value: matchMedia, configurable: true });
    try {
      render(
        <ReceiptTemplate
          {...defaultProps}
          layout="workspace"
          onSearch={mockOnSearch}
          utilityRail={<div>CSV操作</div>}
        />
      );

      expect(screen.getByTestId('search-card')).toHaveAttribute('data-initial-expanded', 'true');
    } finally {
      // @ts-expect-error テスト後の後片付けとして matchMedia を未定義に戻す
      delete window.matchMedia;
    }
  });
});
