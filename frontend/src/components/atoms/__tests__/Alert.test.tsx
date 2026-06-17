import { render, screen } from '@testing-library/react';
import { Alert } from '../Alert';

describe('Alert', () => {
  it('デフォルトの info で children を表示する', () => {
    render(<Alert>お知らせ</Alert>);

    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('お知らせ');
    expect(alert).toHaveClass('bg-blue-50');
  });

  it('warning variant のクラスを適用する', () => {
    render(<Alert variant="warning">警告</Alert>);

    expect(screen.getByRole('alert')).toHaveClass('bg-amber-50');
  });

  it('danger variant のクラスを適用する', () => {
    render(<Alert variant="danger">危険</Alert>);

    expect(screen.getByRole('alert')).toHaveClass('bg-red-50');
  });

  it('success variant のクラスを適用する', () => {
    render(<Alert variant="success">成功</Alert>);

    expect(screen.getByRole('alert')).toHaveClass('bg-teal-50');
  });
});
