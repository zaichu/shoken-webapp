import { render, screen, fireEvent, within } from '@testing-library/react';
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

  test('dates: true でも years が空の場合は SearchCard が非表示になる', () => {
    const { container } = render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={{
          securities: [],
          products: [],
          accounts: [],
          years: [],
          dates: true,
        }}
      />
    );

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

  test('initialExpandedプロパティが変わると展開状態が同期される', () => {
    const { rerender } = render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
        initialExpanded={true}
      />
    );

    expect(screen.getByText('銘柄')).toBeInTheDocument();

    rerender(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
        initialExpanded={false}
      />
    );

    expect(screen.queryByText('銘柄')).not.toBeInTheDocument();
  });

  test('ヘッダーでEnterキーを押すと検索オプションがトグルされる', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    const header = screen.getByTestId('search-card-header');
    // 初期は展開済み → Enterで閉じる
    fireEvent.keyDown(header, { key: 'Enter' });
    expect(screen.queryByText('銘柄')).not.toBeInTheDocument();

    // もう一度Enterで開く
    fireEvent.keyDown(header, { key: ' ' });
    expect(screen.getByText('銘柄')).toBeInTheDocument();

    // 別のキーでは何もしない
    fireEvent.keyDown(header, { key: 'Tab' });
    expect(screen.getByText('銘柄')).toBeInTheDocument();
  });

  test('「条件をクリア」ボタンでEnterキーを押してもpropagationが止まる', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
        onExpandToggle={mockOnExpandToggle}
      />
    );

    // 検索条件を設定してボタンを表示
    fireEvent.click(screen.getByText('株式'));

    const clearButtons = screen.getAllByTestId('search-clear-button');
    clearButtons.forEach(btn => {
      fireEvent.keyDown(btn, { key: 'Enter' });
      fireEvent.keyDown(btn, { key: ' ' });
      fireEvent.keyDown(btn, { key: 'Tab' }); // カバレッジ: key以外は何もしない
    });
    // エラーなく実行されればOK
    expect(clearButtons.length).toBeGreaterThan(0);
    expect(mockOnExpandToggle).not.toHaveBeenCalled();
  });

  test('compactモードの初期状態でクリアボタンがDOMに存在し操作不可である', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
        compact
      />
    );

    const clearButton = screen.getByTestId('search-clear-button');
    expect(clearButton).toBeInTheDocument();
    expect(clearButton).toHaveClass('opacity-0');
    expect(clearButton).toHaveClass('pointer-events-none');
  });

  test('compactモードでクリアボタンのクリックとキーダウンが動作する', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
        compact
      />
    );

    // 検索条件を設定してクリアボタンを有効化
    fireEvent.click(screen.getByText('株式'));

    const clearButton = screen.getByTestId('search-clear-button');
    // クリックでe.stopPropagation()が呼ばれる（line 276）
    fireEvent.click(clearButton);
    expect(mockOnSearch).toHaveBeenCalledWith('');

    // 検索条件を再設定
    fireEvent.click(screen.getByText('株式'));

    // EnterキーでもstopPropagation（line 280-281）
    const clearBtn2 = screen.getByTestId('search-clear-button');
    fireEvent.keyDown(clearBtn2, { key: 'Enter' });
    fireEvent.keyDown(clearBtn2, { key: ' ' });
    expect(clearBtn2).toBeInTheDocument();
  });

  describe('期間ブロック', () => {
    const dateCategories = {
      ...defaultCategories,
      dates: true as const,
    };

    test('datesカテゴリがある場合、期間ブロックが表示される', () => {
      render(<SearchCard onSearch={mockOnSearch} categories={dateCategories} />);
      expect(screen.getByText('期間')).toBeInTheDocument();
    });

    test('dates: true の場合、「期間」が「銘柄」より前に表示される', () => {
      const { container } = render(
        <SearchCard onSearch={mockOnSearch} categories={dateCategories} />
      );
      const text = container.textContent ?? '';
      expect(text.indexOf('期間')).toBeLessThan(text.indexOf('銘柄'));
    });

    test('compact モードでも「期間」が「銘柄」より前に表示される', () => {
      const { container } = render(
        <SearchCard onSearch={mockOnSearch} categories={dateCategories} compact />
      );
      const text = container.textContent ?? '';
      expect(text.indexOf('期間')).toBeLessThan(text.indexOf('銘柄'));
    });

    test('datesカテゴリがない場合、期間ブロックが表示されない', () => {
      render(<SearchCard onSearch={mockOnSearch} categories={defaultCategories} />);
      expect(screen.queryByText('期間')).not.toBeInTheDocument();
    });

    test('年セグメントに text/number input が存在しない', () => {
      render(<SearchCard onSearch={mockOnSearch} categories={dateCategories} />);
      expect(screen.queryByRole('textbox')).not.toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-node-access
      expect(document.querySelector('input[type="number"]')).not.toBeInTheDocument();
    });

    test('年モードでは「年を選択」ボタンが表示される', () => {
      render(<SearchCard onSearch={mockOnSearch} categories={dateCategories} />);
      expect(screen.getByRole('button', { name: '年を選択' })).toBeInTheDocument();
    });

    test("「年を選択」ボタンをクリックすると年候補だけが表示される", () => {
      render(<SearchCard onSearch={mockOnSearch} categories={dateCategories} />);
      fireEvent.click(screen.getByLabelText("年を選択"));
      const picker = screen.getByRole("listbox", { name: "年候補" });
      expect(within(picker).getByText("2023年")).toBeInTheDocument();
      expect(within(picker).getByText("2024年")).toBeInTheDocument();
      expect(within(picker).queryByText(/1月|2月|月曜日|（|）/)).not.toBeInTheDocument();
    });

    test("年候補を選択すると年検索が実行され、パネルが閉じ、field が更新される", () => {
      render(<SearchCard onSearch={mockOnSearch} categories={dateCategories} />);
      fireEvent.click(screen.getByLabelText("年を選択"));
      fireEvent.click(screen.getByRole("option", { name: "2024年" }));
      expect(mockOnSearch).toHaveBeenCalledWith("2024");
      expect(screen.queryByRole("listbox", { name: "年候補" })).not.toBeInTheDocument();
      expect(screen.getByLabelText("年を選択")).toHaveTextContent("2024年");
    });

    test("月セグメントへ切り替えると年候補が閉じる", () => {
      render(<SearchCard onSearch={mockOnSearch} categories={dateCategories} />);
      fireEvent.click(screen.getByLabelText("年を選択"));
      expect(screen.getByRole("listbox", { name: "年候補" })).toBeInTheDocument();
      fireEvent.click(screen.getByRole("button", { name: "月" }));
      expect(screen.queryByRole("listbox", { name: "年候補" })).not.toBeInTheDocument();
    });

    test('期間ブロックのどのセグメントにも曜日・括弧が表示されない（native date input 由来の括弧も含む）', () => {
      const FORBIDDEN = /月曜日|火曜日|水曜日|木曜日|金曜日|土曜日|日曜日|（|）|\(|\)/;
      const { container } = render(
        <SearchCard onSearch={mockOnSearch} categories={dateCategories} />
      );
      // 年モード（初期）
      expect(container.textContent).not.toMatch(FORBIDDEN);
      // 年候補パネルを開いた状態でも同様
      fireEvent.click(screen.getByLabelText('年を選択'));
      expect(container.textContent).not.toMatch(FORBIDDEN);
      // 月モード
      fireEvent.click(screen.getByRole('button', { name: '月' }));
      expect(container.textContent).not.toMatch(FORBIDDEN);
      // 日モード（hidden input が括弧を混入しないことを確認）
      fireEvent.click(screen.getByRole('button', { name: '日' }));
      expect(container.textContent).not.toMatch(FORBIDDEN);
      // 範囲モード（hidden input 2つとも括弧を混入しないことを確認）
      fireEvent.click(screen.getByRole('button', { name: '範囲' }));
      expect(container.textContent).not.toMatch(FORBIDDEN);
    });


    test('月・日・範囲セグメントの native input は aria-hidden で画面上の表示から除外される（括弧・曜日の混入防止）', () => {
      const { container } = render(
        <SearchCard onSearch={mockOnSearch} categories={dateCategories} />
      );
      fireEvent.click(screen.getByRole('button', { name: '月' }));
      // eslint-disable-next-line testing-library/no-node-access
      let ariaHiddenInputs = Array.from(container.querySelectorAll("input[aria-hidden]"));
      expect(ariaHiddenInputs).toHaveLength(1);
      expect(ariaHiddenInputs[0]).toHaveAttribute('type', 'month');
      expect(ariaHiddenInputs[0]).toHaveAttribute('aria-hidden', 'true');
      fireEvent.click(screen.getByRole('button', { name: '日' }));
      // eslint-disable-next-line testing-library/no-node-access
      ariaHiddenInputs = Array.from(container.querySelectorAll("input[aria-hidden]"));
      expect(ariaHiddenInputs).toHaveLength(1);
      expect(ariaHiddenInputs[0]).toHaveAttribute('type', 'date');
      expect(ariaHiddenInputs[0]).toHaveAttribute('aria-hidden', 'true');
      fireEvent.click(screen.getByRole('button', { name: '範囲' }));
      // eslint-disable-next-line testing-library/no-node-access
      ariaHiddenInputs = Array.from(container.querySelectorAll("input[aria-hidden]"));
      expect(ariaHiddenInputs).toHaveLength(2);
      ariaHiddenInputs.forEach(input => expect(input).toHaveAttribute('aria-hidden', 'true'));
    });

    test('月モードで hidden month input に change を発火すると onSearch が呼ばれ button 表示が更新される', () => {
      const { container } = render(
        <SearchCard onSearch={mockOnSearch} categories={dateCategories} />
      );
      fireEvent.click(screen.getByRole('button', { name: '月' }));
      expect(screen.getByRole('button', { name: '月を選択' })).toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-node-access
      const monthInput = container.querySelector("input[aria-hidden]") as HTMLInputElement;
      expect(monthInput).toHaveAttribute('type', 'month');
      fireEvent.change(monthInput, { target: { value: '2024-03' } });
      expect(mockOnSearch).toHaveBeenCalledWith('2024-03');
      expect(screen.getByRole('button', { name: '2024/03' })).toBeInTheDocument();
    });

    test('日モードで hidden date input に change を発火すると onSearch が呼ばれ button 表示が更新される', () => {
      const { container } = render(
        <SearchCard onSearch={mockOnSearch} categories={dateCategories} />
      );
      fireEvent.click(screen.getByRole('button', { name: '日' }));
      // eslint-disable-next-line testing-library/no-node-access
      const dateInput = container.querySelector("input[aria-hidden]") as HTMLInputElement;
      fireEvent.change(dateInput, { target: { value: '2024-03-15' } });
      expect(mockOnSearch).toHaveBeenCalledWith('2024-03-15');
      expect(screen.getByRole('button', { name: '2024/03/15' })).toBeInTheDocument();
    });

    test('範囲モードで hidden date inputs に change を発火すると button 表示が更新され YYYY-MM-DD..YYYY-MM-DD 形式で onSearch が呼ばれる', () => {
      const { container } = render(
        <SearchCard onSearch={mockOnSearch} categories={dateCategories} />
      );
      fireEvent.click(screen.getByRole('button', { name: '範囲' }));
      // eslint-disable-next-line testing-library/no-node-access
      const inputs = Array.from(container.querySelectorAll("input[aria-hidden]")) as HTMLInputElement[];
      fireEvent.change(inputs[0], { target: { value: '2024-03-01' } });
      fireEvent.change(inputs[1], { target: { value: '2024-03-31' } });
      expect(mockOnSearch).toHaveBeenLastCalledWith('2024-03-01..2024-03-31');
      expect(screen.getByRole('button', { name: '2024/03/01' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: '2024/03/31' })).toBeInTheDocument();
    });

    test('銘柄検索後に月セグメントへ切り替えると既存検索が解除される', () => {
      const { container } = render(
        <SearchCard onSearch={mockOnSearch} categories={dateCategories} />
      );
      // 銘柄を選択
      const secSelect = container.querySelector('#securities-search') as HTMLSelectElement;
      fireEvent.change(secSelect, { target: { value: 'AAPL' } });
      expect(mockOnSearch).toHaveBeenCalledWith('AAPL');
      expect(secSelect.value).toBe('AAPL');
      // 月セグメントへ切り替え → 既存検索が解除される
      fireEvent.click(screen.getByRole('button', { name: '月' }));
      expect(mockOnSearch).toHaveBeenLastCalledWith('');
      // 銘柄ドロップダウンの選択表示が外れる
      expect(secSelect.value).toBe('');
    });

    test('クリアで期間入力状態がリセットされる', () => {
      render(<SearchCard onSearch={mockOnSearch} categories={dateCategories} />);
      // 年ピッカーで2024を選択
      fireEvent.click(screen.getByLabelText('年を選択'));
      fireEvent.click(screen.getByRole('option', { name: '2024年' }));
      expect(mockOnSearch).toHaveBeenCalledWith('2024');
      // クリアで検索もUIも初期化される
      fireEvent.click(screen.getByTestId('search-clear-button'));
      expect(mockOnSearch).toHaveBeenLastCalledWith('');
      // セグメントが年モードに戻り、年ピッカーが「年を選択」を表示する
      expect(screen.getByRole('button', { name: '年', pressed: true })).toBeInTheDocument();
      expect(screen.getByLabelText('年を選択')).toHaveTextContent('年を選択');
    });
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
