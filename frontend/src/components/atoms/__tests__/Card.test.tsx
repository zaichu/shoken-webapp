import { render, screen } from '@testing-library/react';
import { Card, CardBody, CardHeader } from '../Card';

describe('Card', () => {
  it('children を表示する', () => {
    render(<Card>カード本文</Card>);

    expect(screen.getByText('カード本文')).toBeInTheDocument();
  });
});

describe('CardHeader', () => {
  it('children を表示し variant クラスを適用する', () => {
    render(<CardHeader variant="primary">ヘッダー</CardHeader>);

    const header = screen.getByText('ヘッダー');
    expect(header).toBeInTheDocument();
    expect(header).toHaveClass('bg-slate-700');
  });
});

describe('CardBody', () => {
  it('children を表示する', () => {
    render(<CardBody>本文</CardBody>);

    expect(screen.getByText('本文')).toBeInTheDocument();
  });
});
