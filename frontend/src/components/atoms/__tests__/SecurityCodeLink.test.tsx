import { render, screen, fireEvent } from '@testing-library/react';
import { vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import { SecurityCodeLink, CopyableInstrumentName } from '../SecurityCodeLink';

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

  it('prefix 付き font weight を指定した場合はデフォルトの font-bold と競合しない', () => {
    render(
      <MemoryRouter>
        <SecurityCodeLink value="7203" className="hover:font-semibold" />
      </MemoryRouter>
    );

    const link = screen.getByRole('link', { name: '7203' });
    expect(link).toHaveClass('hover:font-semibold');
    expect(link).not.toHaveClass('font-bold');
  });
});

describe('CopyableInstrumentName', () => {
  it('銘柄名を表示する', () => {
    render(<CopyableInstrumentName name="テスト株式" code="9433" />);
    expect(screen.getByText('テスト株式')).toBeInTheDocument();
  });

  it('コピーボタンが accessible name を持つ（銘柄名+コード）', () => {
    render(<CopyableInstrumentName name="ＫＤＤＩ" code="9433" />);
    expect(screen.getByRole('button', { name: 'ＫＤＤＩ(9433) をコピー' })).toBeInTheDocument();
  });

  it('コードなしの場合はファンド名のみの accessible name を持つ', () => {
    render(<CopyableInstrumentName name="テストファンドA" />);
    expect(screen.getByRole('button', { name: 'テストファンドA をコピー' })).toBeInTheDocument();
  });

  it('右クリックしてもコピーされない', () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText },
      configurable: true,
    });
    render(<CopyableInstrumentName name="ＫＤＤＩ" code="9433" />);
    fireEvent.contextMenu(screen.getByText('ＫＤＤＩ'));
    expect(writeText).not.toHaveBeenCalled();
  });

  it('コピーボタンをクリックすると clipboard.writeText が呼ばれる', () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText },
      configurable: true,
    });
    render(<CopyableInstrumentName name="ＫＤＤＩ" code="9433" />);
    fireEvent.click(screen.getByRole('button', { name: 'ＫＤＤＩ(9433) をコピー' }));
    expect(writeText).toHaveBeenCalledWith('ＫＤＤＩ(9433)');
  });

  it('navigator.clipboard がない環境でもエラーにならない', () => {
    Object.defineProperty(navigator, 'clipboard', {
      value: undefined,
      configurable: true,
    });
    render(<CopyableInstrumentName name="ＫＤＤＩ" code="9433" />);
    expect(() => {
      fireEvent.click(screen.getByRole('button', { name: 'ＫＤＤＩ(9433) をコピー' }));
    }).not.toThrow();
  });

  it('表示文字列をクリックすると clipboard.writeText が呼ばれる', () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText },
      configurable: true,
    });
    render(<CopyableInstrumentName name="ＫＤＤＩ" code="9433" />);
    fireEvent.click(screen.getByText('ＫＤＤＩ'));
    expect(writeText).toHaveBeenCalledWith('ＫＤＤＩ(9433)');
  });

  it('SVGアイコンに group-focus-visible:opacity-100 が含まれ focus:opacity-100 が含まれない', () => {
    const { container } = render(<CopyableInstrumentName name="ＫＤＤＩ" code="9433" />);
    const svg = container.querySelector('svg');
    expect(svg?.getAttribute('class')).toContain('group-focus-visible:opacity-100');
    expect(svg?.getAttribute('class')).not.toContain('focus:opacity-100');
  });
});
