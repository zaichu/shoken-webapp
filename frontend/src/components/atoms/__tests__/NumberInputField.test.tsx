import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { NumberInputField } from '../NumberInputField';

const defaultProps = {
  label: '数値入力',
  value: undefined as number | undefined,
  onChange: vi.fn(),
};

describe('NumberInputField', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('基本的な数値フィールドを表示する', () => {
    render(<NumberInputField {...defaultProps} />);
    
    const input = screen.getByLabelText('数値入力');
    expect(input).toBeInTheDocument();
    expect(input).toHaveAttribute('type', 'number');
  });

  it('初期値が正しく表示される', () => {
    render(<NumberInputField {...defaultProps} value={123} />);
    
    const input = screen.getByDisplayValue('123');
    expect(input).toBeInTheDocument();
  });

  it('undefinedの場合は空文字が表示される', () => {
    render(<NumberInputField {...defaultProps} value={undefined} />);
    
    const input = screen.getByLabelText('数値入力') as HTMLInputElement;
    expect(input.value).toBe('');
  });

  it('数値入力時にonChangeが呼ばれる', () => {
    const handleChange = vi.fn();
    render(<NumberInputField {...defaultProps} onChange={handleChange} />);
    
    const input = screen.getByLabelText('数値入力');
    fireEvent.change(input, { target: { value: '123' } });
    
    expect(handleChange).toHaveBeenCalledWith(123);
  });

  it('空文字入力時にonChangeがundefinedで呼ばれる', () => {
    const handleChange = vi.fn();
    render(<NumberInputField label="数値入力" value={123} onChange={handleChange} />);
    
    const input = screen.getByLabelText('数値入力');
    // まず値があることを確認
    expect(input).toHaveValue(123);
    
    // 空文字に変更
    fireEvent.change(input, { target: { value: '' } });
    
    expect(handleChange).toHaveBeenCalledWith(undefined);
  });

  it('小数点入力が許可される', () => {
    const handleChange = vi.fn();
    render(<NumberInputField {...defaultProps} onChange={handleChange} allowDecimal />);
    
    const input = screen.getByLabelText('数値入力');
    fireEvent.change(input, { target: { value: '123.45' } });
    
    expect(handleChange).toHaveBeenCalledWith(123.45);
  });

  it('小数点入力が制限される', () => {
    const handleChange = vi.fn();
    render(<NumberInputField {...defaultProps} onChange={handleChange} allowDecimal={false} />);
    
    const input = screen.getByLabelText('数値入力');
    fireEvent.change(input, { target: { value: '123.45' } });
    
    expect(handleChange).toHaveBeenCalledWith(123);
  });

  it('負の値入力が許可される', () => {
    const handleChange = vi.fn();
    render(<NumberInputField {...defaultProps} onChange={handleChange} allowNegative />);
    
    const input = screen.getByLabelText('数値入力');
    fireEvent.change(input, { target: { value: '-123' } });
    
    expect(handleChange).toHaveBeenCalledWith(-123);
  });

  it('負の値入力が制限される', () => {
    const handleChange = vi.fn();
    render(<NumberInputField {...defaultProps} onChange={handleChange} allowNegative={false} />);
    
    const input = screen.getByLabelText('数値入力');
    fireEvent.change(input, { target: { value: '-123' } });
    
    expect(handleChange).toHaveBeenCalledWith(0);
  });

  it('最小値が適用される', () => {
    const handleChange = vi.fn();
    render(<NumberInputField {...defaultProps} onChange={handleChange} min={10} />);
    
    const input = screen.getByLabelText('数値入力');
    fireEvent.change(input, { target: { value: '5' } });
    
    expect(handleChange).toHaveBeenCalledWith(10);
  });

  it('最大値が適用される', () => {
    const handleChange = vi.fn();
    render(<NumberInputField {...defaultProps} onChange={handleChange} max={100} />);
    
    const input = screen.getByLabelText('数値入力');
    fireEvent.change(input, { target: { value: '150' } });
    
    expect(handleChange).toHaveBeenCalledWith(100);
  });

  it('精度が適用される', () => {
    const handleChange = vi.fn();
    render(<NumberInputField {...defaultProps} onChange={handleChange} precision={1} />);
    
    const input = screen.getByLabelText('数値入力');
    fireEvent.change(input, { target: { value: '123.456' } });
    
    expect(handleChange).toHaveBeenCalledWith(123.5);
  });

  it('無効な文字入力時はonChangeが呼ばれない', () => {
    const handleChange = vi.fn();
    render(<NumberInputField {...defaultProps} onChange={handleChange} />);
    
    const input = screen.getByLabelText('数値入力');
    fireEvent.change(input, { target: { value: 'abc' } });
    
    expect(handleChange).not.toHaveBeenCalled();
  });

  it('エラーメッセージが表示される', () => {
    render(<NumberInputField {...defaultProps} error="必須項目です" />);
    
    expect(screen.getByText('必須項目です')).toBeInTheDocument();
    expect(screen.getByText('必須項目です')).toHaveClass('text-danger');
  });

  it('ヘルプテキストが表示される', () => {
    render(<NumberInputField {...defaultProps} helpText="数値を入力してください" />);
    
    expect(screen.getByText('数値を入力してください')).toBeInTheDocument();
  });

  it('disabledプロパティが動作する', () => {
    render(<NumberInputField {...defaultProps} disabled />);
    
    const input = screen.getByLabelText('数値入力');
    expect(input).toBeDisabled();
  });

  it('カスタムクラスが適用される', () => {
    render(<NumberInputField {...defaultProps} className="custom-class" />);
    
    const input = screen.getByLabelText('数値入力');
    expect(input).toHaveClass('custom-class');
  });

  it('step属性が適用される', () => {
    render(<NumberInputField {...defaultProps} step={0.1} />);
    
    const input = screen.getByLabelText('数値入力');
    expect(input).toHaveAttribute('step', '0.1');
  });

  it('min/max属性が適用される', () => {
    render(<NumberInputField {...defaultProps} min={0} max={100} />);
    
    const input = screen.getByLabelText('数値入力');
    expect(input).toHaveAttribute('min', '0');
    expect(input).toHaveAttribute('max', '100');
  });

  it('blurで空文字の場合は何もしない', () => {
    render(<NumberInputField {...defaultProps} />);
    const input = screen.getByLabelText('数値入力') as HTMLInputElement;
    input.value = '';
    fireEvent.blur(input);
    expect(input.value).toBe('');
  });

  it('blurで小数入力の場合に適切な形式でフォーマットされる', () => {
    render(<NumberInputField {...defaultProps} allowDecimal precision={2} />);
    const input = screen.getByLabelText('数値入力') as HTMLInputElement;
    input.value = '3.14159';
    fireEvent.blur(input);
    expect(input.value).toBe('3.14');
  });

  it('blurでallowDecimal=falseの場合に数値文字列になる', () => {
    render(<NumberInputField {...defaultProps} allowDecimal={false} />);
    const input = screen.getByLabelText('数値入力') as HTMLInputElement;
    input.value = '42.7';
    fireEvent.blur(input);
    expect(input.value).toBe('42.7');
  });

  it('blurでonBlurプロップが呼ばれる', () => {
    const onBlur = vi.fn();
    render(<NumberInputField {...defaultProps} onBlur={onBlur} />);
    const input = screen.getByLabelText('数値入力');
    fireEvent.blur(input, { target: { value: '100' } });
    expect(onBlur).toHaveBeenCalledTimes(1);
  });

  it('blurで非数値入力の場合はフォーマットされない', () => {
    render(<NumberInputField {...defaultProps} />);
    const input = screen.getByLabelText('数値入力') as HTMLInputElement;
    input.type = 'text';
    input.value = 'abc';
    fireEvent.blur(input);
    expect(input.value).toBe('abc');
  });
});
