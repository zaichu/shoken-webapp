import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { vi, describe, test, expect, beforeEach } from 'vitest';
import '@testing-library/jest-dom';
import { ReceiptTemplate } from '../ReceiptTemplate';

// モック用のReceiptTableコンポーネント
const MockReceiptTable = React.forwardRef<HTMLDivElement, { forceResize?: number }>(
  ({ forceResize, ...props }, ref) => (
    <div
      ref={ref}
      data-testid="mock-receipt-table"
      data-force-resize={forceResize}
      {...props}
    >
      Mock Receipt Table
    </div>
  )
);
MockReceiptTable.displayName = 'ReceiptTable';

// SearchCardのモック
vi.mock('@/components/organisms/SearchCard/SearchCard', () => ({
  SearchCard: ({ onSearch, onExpandToggle }: {
    onSearch: (query: string) => void;
    onExpandToggle?: (isExpanded: boolean) => void;
  }) => (
    <div data-testid="search-card">
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

  test('複数の子要素がある場合、ReceiptTableのみがenhanceされる', () => {
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
    
    // ReceiptTableにはforceResizeが設定されている
    expect(receiptTable.getAttribute('data-force-resize')).toBeDefined();
    
    // 他のコンポーネントにはforceResizeが設定されていない
    expect(otherComponent.getAttribute('data-force-resize')).toBeNull();
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
});
