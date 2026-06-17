import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { SecurityCodeLink } from '../SecurityCodeLink';

describe('SecurityCodeLink', () => {
  it('有効な 4 桁コードをリンクとして表示する', () => {
    render(
      <MemoryRouter>
        <SecurityCodeLink value="7203" />
      </MemoryRouter>
    );

    const link = screen.getByRole('link', { name: '7203' });
    expect(link).toHaveAttribute('href', '/search?code=7203');
  });

  it.each([null, undefined, ''])('無効な値 %p の場合は - を表示する', (value) => {
    render(
      <MemoryRouter>
        <SecurityCodeLink value={value} />
      </MemoryRouter>
    );

    expect(screen.getByText('-')).toBeInTheDocument();
  });

  it('正規表現に合わない文字列はリンク化せずテキスト表示する', () => {
    render(
      <MemoryRouter>
        <SecurityCodeLink value="7203?" />
      </MemoryRouter>
    );

    expect(screen.getByText('7203?')).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: '7203?' })).not.toBeInTheDocument();
  });

  it('classNameを指定するとリンクに適用される', () => {
    render(
      <MemoryRouter>
        <SecurityCodeLink value="7203" className="custom-class" />
      </MemoryRouter>
    );

    expect(screen.getByRole('link', { name: '7203' })).toHaveClass('custom-class');
  });

  it('font weight を指定した場合はデフォルトの font-bold と競合しない', () => {
    render(
      <MemoryRouter>
        <SecurityCodeLink value="7203" className="text-[11px] font-semibold" />
      </MemoryRouter>
    );

    const link = screen.getByRole('link', { name: '7203' });
    expect(link).toHaveClass('font-semibold');
    expect(link).not.toHaveClass('font-bold');
    expect(link).toHaveClass('text-blue-700');
  });
});
