import { render, screen, fireEvent } from '@testing-library/react';
import { vi, describe, test, expect, beforeEach } from 'vitest';
import '@testing-library/jest-dom';
import { SearchCard } from '../SearchCard';

describe('SearchCard', () => {
  const mockOnSearch = vi.fn();
  const mockOnExpandToggle = vi.fn();

  const defaultCategories = {
    securities: [
      { value: 'AAPL', label: 'Apple Inc.' },
      { value: 'GOOGL', label: 'Alphabet Inc.' }
    ],
    products: ['株式', '投資信託'],
    accounts: ['一般口座', 'NISA口座'],
    years: [
      { value: '2023', label: '2023年' },
      { value: '2024', label: '2024年' }
    ],
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  test('検索カードが正しくレンダリングされる', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    expect(screen.getByText('検索オプション')).toBeInTheDocument();
  });

  test('初期状態では検索オプションが展開されている', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    expect(screen.getByText('銘柄')).toBeInTheDocument();
    expect(screen.getByText('西暦')).toBeInTheDocument();
    expect(screen.getByText('商品')).toBeInTheDocument();
    expect(screen.getByText('口座')).toBeInTheDocument();
  });

  test('initialExpanded=falseのとき初期状態では検索オプションが折りたたまれている', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
        initialExpanded={false}
      />
    );

    expect(screen.queryByText('銘柄')).not.toBeInTheDocument();
    expect(screen.queryByText('西暦')).not.toBeInTheDocument();
    expect(screen.queryByText('商品')).not.toBeInTheDocument();
    expect(screen.queryByText('口座')).not.toBeInTheDocument();
  });

  test('ヘッダーをクリックすると検索オプションが折りたたまれる', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    const header = screen.getByTestId('search-card-header');
    fireEvent.click(header!);

    expect(screen.queryByText('銘柄')).not.toBeInTheDocument();
    expect(screen.queryByText('西暦')).not.toBeInTheDocument();
    expect(screen.queryByText('商品')).not.toBeInTheDocument();
  });

  test('onExpandToggleコールバックが呼ばれる', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
        onExpandToggle={mockOnExpandToggle}
      />
    );

    const header = screen.getByTestId('search-card-header');
    fireEvent.click(header!);

    expect(mockOnExpandToggle).toHaveBeenCalledWith(false);

    fireEvent.click(header!);
    expect(mockOnExpandToggle).toHaveBeenCalledWith(true);
  });

  test('銘柄ドロップダウンから選択すると検索が実行される', () => {
    const { container } = render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    // 初期状態で展開済み - 銘柄選択（IDで指定）
    const select = container.querySelector('#securities-search') as HTMLSelectElement;
    fireEvent.change(select, { target: { value: 'AAPL' } });

    expect(mockOnSearch).toHaveBeenCalledWith('AAPL');
  });

  test('商品ボタンをクリックすると検索が実行される', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    // 初期状態で展開済み - 商品ボタンクリック
    const productButton = screen.getByText('株式');
    fireEvent.click(productButton);

    expect(mockOnSearch).toHaveBeenCalledWith('株式');
  });

  test('口座ボタンをクリックすると検索が実行される', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    // 初期状態で展開済み - 口座ボタンクリック
    const accountButton = screen.getByText('一般口座');
    fireEvent.click(accountButton);

    expect(mockOnSearch).toHaveBeenCalledWith('一般口座');
  });

  test('categoriesが未定義の場合SearchCardが表示されない', () => {
    const { container } = render(
      <SearchCard
        onSearch={mockOnSearch}
      />
    );

    // SearchCardがレンダーされないことを確認
    expect(container.firstChild).toBeNull();
  });

  test('空のカテゴリの場合SearchCardが表示されない', () => {
    const { container } = render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={{
          securities: [],
          products: [],
          accounts: [],
          years: []
        }}
      />
    );

    // SearchCardがレンダーされないことを確認
    expect(container.firstChild).toBeNull();
  });

  test('年度のみのデータがある場合のレイアウト', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={{
          securities: undefined,
          products: undefined,
          accounts: undefined,
          years: [{ value: '2024', label: '2024年' }]
        }}
      />
    );

    // 初期状態で展開済み
    expect(screen.getByText('西暦')).toBeInTheDocument();
    expect(screen.queryByText('銘柄')).not.toBeInTheDocument();
    expect(screen.queryByText('商品')).not.toBeInTheDocument();
    expect(screen.queryByText('口座')).not.toBeInTheDocument();
  });

  test('undefinedカテゴリの場合はSearchCardが非表示になる', () => {
    const { container } = render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={{
          securities: undefined,
          products: undefined,
          accounts: undefined,
          years: undefined
        }}
      />
    );

    // SearchCardがレンダーされないことを確認
    expect(container.firstChild).toBeNull();
  });

  test('10個を超えるボタンで改行が適用される', () => {
    const manyProducts = Array.from({ length: 25 }, (_, i) => `商品${i + 1}`);

    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={{
          ...defaultCategories,
          products: manyProducts
        }}
      />
    );

    // 初期状態で展開済み - 11番目、21番目の商品ボタンの後に改行要素があることを確認
    expect(screen.getByText('商品11')).toBeInTheDocument();
    expect(screen.getByText('商品21')).toBeInTheDocument();
  });

  test('初期状態では「条件をクリア」ボタンが操作不可である', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    const clearButton = screen.getByTestId('search-clear-button');
    expect(clearButton).toHaveClass('opacity-0');
    expect(clearButton).toHaveClass('pointer-events-none');
  });

  test('検索条件を選択すると「条件をクリア」ボタンが操作可能になる', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    // 商品ボタンをクリックして検索条件を設定
    fireEvent.click(screen.getByText('株式'));

    const clearButton = screen.getByTestId('search-clear-button');
    expect(clearButton).toHaveClass('opacity-100');
    expect(clearButton).not.toHaveClass('pointer-events-none');
  });

  test('「条件をクリア」ボタンをクリックすると検索条件が初期状態に戻る', () => {
    const { container } = render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    // 銘柄を選択
    const select = container.querySelector('#securities-search') as HTMLSelectElement;
    fireEvent.change(select, { target: { value: 'AAPL' } });
    expect(mockOnSearch).toHaveBeenCalledWith('AAPL');

    // クリアボタンをクリック
    const clearButton = screen.getByTestId('search-clear-button');
    fireEvent.click(clearButton);

    expect(mockOnSearch).toHaveBeenCalledWith('');
    // クリア後はボタンが操作不可になる
    expect(clearButton).toHaveClass('opacity-0');
    // ドロップダウンが初期値に戻る
    expect(select.value).toBe('');
  });

  test('ドロップダウンの「全て表示」オプションが正しく動作する', () => {
    const { container } = render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    // 初期状態で展開済み - IDで指定して選択
    const select = container.querySelector('#securities-search') as HTMLSelectElement;
    fireEvent.change(select, { target: { value: '' } });

    expect(mockOnSearch).toHaveBeenCalledWith('');
  });

  test('compactモードでは検索グリッドが1列表示になる', () => {
    const { container } = render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
        compact
      />
    );

    const grid = container.querySelector('#search-options-body > div');
    expect(grid).toHaveClass('grid-cols-1');
    expect(grid).not.toHaveClass('sm:grid-cols-2');
    expect(grid).not.toHaveClass('lg:grid-cols-4');
  });
});
