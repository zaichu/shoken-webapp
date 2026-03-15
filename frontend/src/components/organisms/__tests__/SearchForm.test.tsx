import { render, screen, fireEvent } from '@testing-library/react';
import { vi } from 'vitest';
import { SearchForm } from '../SearchForm';

describe('SearchForm', () => {
  it('input に stockCode を表示する', () => {
    render(
      <SearchForm
        stockCode="7203"
        onStockCodeChange={vi.fn()}
        onSubmit={vi.fn()}
      />
    );

    expect(screen.getByRole('textbox', { name: '銘柄コードまたは銘柄名' })).toHaveValue('7203');
  });

  it('input の変更で onStockCodeChange を呼ぶ', () => {
    const handleChange = vi.fn();
    render(
      <SearchForm
        stockCode="7203"
        onStockCodeChange={handleChange}
        onSubmit={vi.fn()}
      />
    );

    fireEvent.change(screen.getByRole('textbox', { name: '銘柄コードまたは銘柄名' }), {
      target: { value: '6758' },
    });

    expect(handleChange).toHaveBeenCalledWith('6758');
  });

  it('フォーム送信で onSubmit を呼ぶ', () => {
    const handleSubmit = vi.fn();
    render(
      <SearchForm
        stockCode="7203"
        onStockCodeChange={vi.fn()}
        onSubmit={handleSubmit}
      />
    );

    const input = screen.getByRole('textbox', { name: '銘柄コードまたは銘柄名' });
    fireEvent.submit(input.closest('form') as HTMLFormElement);

    expect(handleSubmit).toHaveBeenCalledTimes(1);
  });

  it('isLoading=true で送信ボタンを無効化する', () => {
    render(
      <SearchForm
        stockCode="7203"
        onStockCodeChange={vi.fn()}
        onSubmit={vi.fn()}
        isLoading
      />
    );

    expect(screen.getByRole('button', { name: '検索中' })).toBeDisabled();
  });
});
