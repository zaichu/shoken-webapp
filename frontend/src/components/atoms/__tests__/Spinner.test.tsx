import { render, screen } from '@testing-library/react';
import { Spinner } from '../Spinner';

describe('Spinner', () => {
  it('デフォルト label を aria-label で確認できる', () => {
    render(<Spinner />);

    expect(screen.getByRole('status', { name: '読み込み中...' })).toBeInTheDocument();
  });

  it('label props を aria-label に反映する', () => {
    render(<Spinner label="保存中" />);

    expect(screen.getByRole('status', { name: '保存中' })).toBeInTheDocument();
  });

  it('size が lg のときクラスを適用する', () => {
    render(<Spinner size="lg" />);

    expect(screen.getByRole('status')).toHaveClass('h-8', 'w-8');
  });
});
