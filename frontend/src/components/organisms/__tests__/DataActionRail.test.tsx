import type { ComponentProps } from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { vi } from 'vitest';
import { DataActionRail } from '../DataActionRail/DataActionRail';

const createProps = (overrides: Partial<ComponentProps<typeof DataActionRail>> = {}) => ({
  onFileSelect: vi.fn(),
  hasCsvFile: true,
  saveLabel: '保存する',
  onSave: vi.fn(),
  hasDbData: true,
  deleteLabel: '削除する',
  onDeleteRequest: vi.fn(),
  saveModeLabel: 'CSV',
  ...overrides,
});

describe('DataActionRail', () => {
  it('hasCsvFile=true で saveLabel のボタンを表示する', () => {
    render(<DataActionRail {...createProps()} />);

    expect(screen.getByRole('button', { name: '保存する' })).toBeInTheDocument();
  });

  it('hasCsvFile=false で保存ボタンを表示しない', () => {
    render(<DataActionRail {...createProps({ hasCsvFile: false })} />);

    expect(screen.queryByRole('button', { name: '保存する' })).not.toBeInTheDocument();
  });

  it('hasDbData=true で deleteLabel のボタンを表示する', () => {
    render(<DataActionRail {...createProps()} />);

    expect(screen.getByRole('button', { name: '削除する' })).toBeInTheDocument();
  });

  it('保存ボタンのクリックで onSave を呼ぶ', () => {
    const handleSave = vi.fn();
    render(<DataActionRail {...createProps({ onSave: handleSave })} />);

    fireEvent.click(screen.getByRole('button', { name: '保存する' }));

    expect(handleSave).toHaveBeenCalledTimes(1);
  });

  it('削除ボタンのクリックで onDeleteRequest を呼ぶ', () => {
    const handleDelete = vi.fn();
    render(<DataActionRail {...createProps({ onDeleteRequest: handleDelete })} />);

    fireEvent.click(screen.getByRole('button', { name: '削除する' }));

    expect(handleDelete).toHaveBeenCalledTimes(1);
  });
});
