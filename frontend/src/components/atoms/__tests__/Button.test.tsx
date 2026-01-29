import { render, screen, fireEvent } from '@testing-library/react';
import { vi } from 'vitest';
import { Button, ButtonProps } from '../Button';

const defaultProps: Partial<ButtonProps> = {
  children: 'テストボタン',
};

describe('Button', () => {
  it('基本的なボタンを表示する', () => {
    render(<Button {...defaultProps} />);
    
    const button = screen.getByRole('button', { name: 'テストボタン' });
    expect(button).toBeInTheDocument();
    expect(button).toHaveClass('bg-primary');
  });

  it('指定されたvariantのクラスが適用される', () => {
    render(<Button {...defaultProps} variant="success" />);
    
    const button = screen.getByRole('button');
    expect(button).toHaveClass('bg-success');
  });

  it('outline variantが正しく適用される', () => {
    render(<Button {...defaultProps} variant="outline-danger" />);
    
    const button = screen.getByRole('button');
    expect(button).toHaveClass('border-danger');
  });

  it('指定されたsizeのクラスが適用される', () => {
    render(<Button {...defaultProps} size="lg" />);
    
    const button = screen.getByRole('button');
    expect(button).toHaveClass('text-lg');
  });

  it('デフォルトサイズ(md)の場合はサイズクラスが追加されない', () => {
    render(<Button {...defaultProps} size="md" />);
    
    const button = screen.getByRole('button');
    expect(button).toHaveClass('text-base');
  });

  it('fullWidthプロパティが動作する', () => {
    render(<Button {...defaultProps} fullWidth />);
    
    const button = screen.getByRole('button');
    expect(button).toHaveClass('w-full');
  });

  it('loadingの場合は読み込み中表示になる', () => {
    render(<Button {...defaultProps} loading />);
    
    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
    
    const spinner = screen.getByLabelText('読み込み中...');
    expect(spinner).toBeInTheDocument();
    
    expect(screen.getByText('読み込み中...')).toBeInTheDocument();
  });

  it('loadingの場合はdata-loading属性が付与される', () => {
    render(<Button {...defaultProps} loading />);
    
    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('data-loading', 'true');
  });

  it('アイコンが左側に表示される', () => {
    const icon = <span data-testid="icon">🚀</span>;
    render(<Button {...defaultProps} icon={icon} iconPosition="left" />);
    
    expect(screen.getByTestId('icon')).toBeInTheDocument();
    const iconElement = screen.getByTestId('icon').parentElement;
    expect(iconElement).toHaveClass('mr-2');
  });

  it('アイコンが右側に表示される', () => {
    const icon = <span data-testid="icon">🚀</span>;
    render(<Button {...defaultProps} icon={icon} iconPosition="right" />);
    
    expect(screen.getByTestId('icon')).toBeInTheDocument();
    const iconElement = screen.getByTestId('icon').parentElement;
    expect(iconElement).toHaveClass('ml-2');
  });

  it('disabledプロパティが動作する', () => {
    render(<Button {...defaultProps} disabled />);
    
    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
  });

  it('カスタムクラスが適用される', () => {
    render(<Button {...defaultProps} className="custom-class" />);
    
    const button = screen.getByRole('button');
    expect(button).toHaveClass('custom-class');
  });

  it('onClickイベントが動作する', () => {
    const handleClick = vi.fn();
    render(<Button {...defaultProps} onClick={handleClick} />);
    
    const button = screen.getByRole('button');
    fireEvent.click(button);
    
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('loadingの場合はクリックイベントが実行されない', () => {
    const handleClick = vi.fn();
    render(<Button {...defaultProps} loading onClick={handleClick} />);
    
    const button = screen.getByRole('button');
    fireEvent.click(button);
    
    expect(handleClick).not.toHaveBeenCalled();
  });

  it('refが正しく転送される', () => {
    const ref = { current: null };
    render(<Button {...defaultProps} ref={ref} />);
    
    expect(ref.current).toBeInstanceOf(HTMLButtonElement);
  });

  it('追加のpropsが正しく渡される', () => {
    render(<Button {...defaultProps} data-testid="custom-button" title="カスタムタイトル" />);
    
    const button = screen.getByTestId('custom-button');
    expect(button).toHaveAttribute('title', 'カスタムタイトル');
  });
});
