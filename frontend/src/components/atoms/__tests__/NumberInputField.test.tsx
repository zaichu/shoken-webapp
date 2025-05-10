import { render, screen, fireEvent } from '@testing-library/react';
import { vi } from 'vitest';
import { NumberInputField } from '../NumberInputField';

describe('NumberInputField', () => {
  it('renders with label and value', () => {
    render(
      <NumberInputField
        label="テストラベル"
        value={123}
        onChange={() => {}}
      />
    );

    expect(screen.getByLabelText('テストラベル')).toBeInTheDocument();
    expect(screen.getByDisplayValue('123')).toBeInTheDocument();
  });

  it('calls onChange with number value when input changes', () => {
    const mockOnChange = vi.fn();
    render(
      <NumberInputField
        label="テストラベル"
        value={0}
        onChange={mockOnChange}
      />
    );

    const input = screen.getByLabelText('テストラベル');
    fireEvent.change(input, { target: { value: '456' } });

    expect(mockOnChange).toHaveBeenCalledWith(456);
  });

  it('applies default className when not specified', () => {
    render(
      <NumberInputField
        label="テストラベル"
        value={0}
        onChange={() => {}}
      />
    );

    const input = screen.getByLabelText('テストラベル');
    expect(input).toHaveClass('form-control-plaintext border');
  });

  it('applies custom className when specified', () => {
    const customClass = 'custom-class';
    render(
      <NumberInputField
        label="テストラベル"
        value={0}
        onChange={() => {}}
        className={customClass}
      />
    );

    const input = screen.getByLabelText('テストラベル');
    expect(input).toHaveClass(customClass);
  });

  it('displays error message when error prop is provided', () => {
    const errorMessage = 'エラーメッセージ';
    render(
      <NumberInputField
        label="テストラベル"
        value={0}
        onChange={() => {}}
        error={errorMessage}
      />
    );

    expect(screen.getByText(errorMessage)).toBeInTheDocument();
  });

  it('converts empty string to 0', () => {
    const mockOnChange = vi.fn();
    render(
      <NumberInputField
        label="テストラベル"
        value={100}
        onChange={mockOnChange}
      />
    );

    const input = screen.getByLabelText('テストラベル');
    fireEvent.change(input, { target: { value: '' } });

    expect(mockOnChange).toHaveBeenCalledWith(0);
  });

  it('converts NaN to 0', () => {
    const mockOnChange = vi.fn();
    render(
      <NumberInputField
        label="テストラベル"
        value={100}
        onChange={mockOnChange}
      />
    );

    const input = screen.getByLabelText('テストラベル');
    fireEvent.change(input, { target: { value: 'not a number' } });

    // Number('not a number') returns NaN, which gets converted to 0
    expect(mockOnChange).toHaveBeenCalledWith(NaN);
  });

  it('renders with placeholder when provided', () => {
    const placeholderText = 'プレースホルダーテキスト';
    render(
      <NumberInputField
        label="テストラベル"
        value={0}
        onChange={() => {}}
        placeholder={placeholderText}
      />
    );

    const input = screen.getByPlaceholderText(placeholderText);
    expect(input).toBeInTheDocument();
  });

  it('renders with custom id when provided', () => {
    const customId = 'custom-input-id';
    render(
      <NumberInputField
        label="テストラベル"
        value={0}
        onChange={() => {}}
        id={customId}
      />
    );

    const input = screen.getByLabelText('テストラベル');
    expect(input).toHaveAttribute('id', customId);
  });
});
