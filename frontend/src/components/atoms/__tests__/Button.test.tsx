import { render, screen, fireEvent } from '@testing-library/react';
import { vi } from 'vitest';
import { Button } from '../Button';

const defaultProps = {
  children: 'テストボタン',
};

describe('Button', () => {
  it('基本的なボタンを表示する', () => {
    render(<Button {...defaultProps} />);
    
    const button = screen.getByRole('button', { name: 'テストボタン' });
    expect(button).toBeInTheDocument();
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

  it('アイコンが表示される', () => {
    const icon = <span data-testid="icon">🚀</span>;
    render(<Button {...defaultProps} icon={icon} iconPosition="left" />);
    
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('disabledプロパティが動作する', () => {
    render(<Button {...defaultProps} disabled />);
    
    const button = screen.getByRole('button');
    expect(button).toBeDisabled();
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
