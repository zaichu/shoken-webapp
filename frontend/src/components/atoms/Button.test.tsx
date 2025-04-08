import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Button } from './Button';

describe('Button コンポーネント', () => {
  it('子要素をレンダリングする', () => {
    render(<Button>テストボタン</Button>);
    expect(screen.getByText('テストボタン')).toBeInTheDocument();
  });

  it('クリックイベントを発火する', () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>クリック</Button>);
    fireEvent.click(screen.getByText('クリック'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('disabledの場合にクリックイベントが発火しない', () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick} disabled>クリック</Button>);
    const button = screen.getByText('クリック');
    expect(button).toBeDisabled();
    fireEvent.click(button);
    expect(handleClick).not.toHaveBeenCalled();
  });

  it('正しくvariantクラスを適用する', () => {
    render(<Button variant="success">成功ボタン</Button>);
    const button = screen.getByText('成功ボタン');
    expect(button.className).toContain('btn-success');
  });

  it('正しくsizeクラスを適用する', () => {
    render(<Button size="lg">大きいボタン</Button>);
    const button = screen.getByText('大きいボタン');
    expect(button.className).toContain('btn-lg');
  });

  it('fullWidthが適用される', () => {
    render(<Button fullWidth>幅広ボタン</Button>);
    const button = screen.getByText('幅広ボタン');
    expect(button.className).toContain('w-100');
  });
});
