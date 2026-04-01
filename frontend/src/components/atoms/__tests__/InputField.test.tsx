import { render, screen } from '@testing-library/react';
import { InputField } from '../InputField';

describe('InputField', () => {
  it('基本的なフィールドを表示する', () => {
    render(<InputField label="テスト項目" placeholder="値を入力" />);
    
    expect(screen.getByLabelText('テスト項目')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('値を入力')).toBeInTheDocument();
  });

  it('エラーメッセージを表示する', () => {
    render(<InputField label="テスト項目" error="必須項目です" />);
    
    expect(screen.getByText('必須項目です')).toBeInTheDocument();
    expect(screen.getByText('必須項目です')).toHaveClass('text-danger');
  });

  it('ヘルプテキストを表示する', () => {
    render(<InputField id="amount" label="テスト項目" helpText="半角数字で入力" />);
    
    const input = screen.getByLabelText('テスト項目');
    expect(screen.getByText('半角数字で入力')).toBeInTheDocument();
    expect(screen.getByText('半角数字で入力')).toHaveClass('text-secondary');
    expect(input).toHaveAttribute('aria-describedby', 'amount-help');
  });

  it('ヘルプテキストとしてReactコンポーネントを表示する', () => {
    const helpTextComponent = <span data-testid="custom-help">カスタムヘルプ</span>;
    render(<InputField label="テスト項目" helpText={helpTextComponent} />);
    
    expect(screen.getByTestId('custom-help')).toBeInTheDocument();
    expect(screen.getByText('カスタムヘルプ')).toBeInTheDocument();
  });

  it('エラーとヘルプテキストの両方を表示する', () => {
    render(
      <InputField
        id="amount"
        label="テスト項目"
        error="必須項目です"
        helpText="半角数字で入力"
      />
    );
    
    const input = screen.getByLabelText('テスト項目');
    expect(screen.getByText('必須項目です')).toBeInTheDocument();
    expect(screen.getByText('半角数字で入力')).toBeInTheDocument();
    expect(input).toHaveAttribute('aria-describedby', 'amount-error amount-help');
  });

  it('フルワイドオプションが動作する', () => {
    const { container } = render(<InputField label="テスト項目" fullWidth />);
    
    const wrapper = container.querySelector('div.w-full');
    const input = screen.getByLabelText('テスト項目');
    expect(wrapper).toBeInTheDocument();
    expect(input).toHaveClass('w-full');
  });

  it('fullWidth のデフォルト値では w-full クラスが付かない', () => {
    const { container } = render(<InputField label="テスト項目" />);

    const input = screen.getByLabelText('テスト項目');
    expect(container.firstChild).not.toHaveClass('w-full');
    expect(input).not.toHaveClass('w-full');
  });

  it('追加のクラス名が適用される', () => {
    render(<InputField label="テスト項目" className="custom-class" />);
    
    const input = screen.getByLabelText('テスト項目');
    expect(input).toHaveClass('custom-class');
  });

  it('disabledプロパティが動作する', () => {
    render(<InputField label="テスト項目" disabled />);
    
    const input = screen.getByLabelText('テスト項目');
    expect(input).toBeDisabled();
  });
});
