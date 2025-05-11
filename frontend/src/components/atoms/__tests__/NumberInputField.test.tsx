import { render, screen, fireEvent } from '@testing-library/react';
import { vi } from 'vitest';
import { NumberInputField } from '../NumberInputField';

describe('NumberInputField', () => {
  const mockOnChange = vi.fn();

  beforeEach(() => {
    mockOnChange.mockClear();
  });

  it('数値入力フィールドを表示する', () => {
    render(
      <NumberInputField
        label="数量"
        value={100}
        onChange={mockOnChange}
        placeholder="数値を入力"
      />
    );
    
    const input = screen.getByLabelText('数量');
    expect(input).toBeInTheDocument();
    expect(input).toHaveAttribute('type', 'number');
    expect(input).toHaveValue(100);
    expect(input).toHaveAttribute('placeholder', '数値を入力');
  });

  it('数値入力の変更を処理する', () => {
    render(
      <NumberInputField
        label="数量"
        value={100}
        onChange={mockOnChange}
      />
    );
    
    const input = screen.getByLabelText('数量');
    fireEvent.change(input, { target: { value: '200' } });
    
    expect(mockOnChange).toHaveBeenCalledWith(200);
  });

  it('エラーメッセージを表示する', () => {
    render(
      <NumberInputField
        label="数量"
        value={100}
        onChange={mockOnChange}
        error="必須項目です"
      />
    );
    
    expect(screen.getByText('必須項目です')).toBeInTheDocument();
  });

  it('ヘルプテキストを表示する', () => {
    render(
      <NumberInputField
        label="数量"
        value={100}
        onChange={mockOnChange}
        helpText="1以上の整数を入力してください"
      />
    );
    
    expect(screen.getByText('1以上の整数を入力してください')).toBeInTheDocument();
  });

  it('disabledプロパティが動作する', () => {
    render(
      <NumberInputField
        label="数量"
        value={100}
        onChange={mockOnChange}
        disabled
      />
    );
    
    const input = screen.getByLabelText('数量');
    expect(input).toBeDisabled();
  });

  it('デフォルトのクラス名が適用される', () => {
    render(
      <NumberInputField
        label="数量"
        value={100}
        onChange={mockOnChange}
      />
    );
    
    const input = screen.getByLabelText('数量');
    expect(input).toHaveClass('form-control-plaintext', 'border');
  });

  it('カスタムクラス名が適用される', () => {
    render(
      <NumberInputField
        label="数量"
        value={100}
        onChange={mockOnChange}
        className="custom-input"
      />
    );
    
    const input = screen.getByLabelText('数量');
    expect(input).toHaveClass('custom-input');
  });

  it('空文字列の入力をNumber型に変換する', () => {
    render(
      <NumberInputField
        label="数量"
        value={100}
        onChange={mockOnChange}
      />
    );
    
    const input = screen.getByLabelText('数量');
    fireEvent.change(input, { target: { value: '' } });
    
    // Number('')は0を返す
    expect(mockOnChange).toHaveBeenCalledWith(0);
  });
});
