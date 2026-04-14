import type { ComponentProps } from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { CsvUploadResult } from '@/lib/csvImport';
import { DataActionRail } from '../DataActionRail';

const { mockCSVFileInput, mockCsvSaveResultNotice } = vi.hoisted(() => ({
  mockCSVFileInput: vi.fn(),
  mockCsvSaveResultNotice: vi.fn(),
}));

vi.mock('@/components/molecules/CSVFileInput', () => ({
  CSVFileInput: (props: {
    onFileSelect: (file: File) => void;
    selectedFileName?: string;
    disabled?: boolean;
  }) => {
    mockCSVFileInput(props);
    return <div>CSVFileInput</div>;
  },
}));

vi.mock('@/components/molecules/CsvSaveResultNotice', () => ({
  CsvSaveResultNotice: (props: {
    result: CsvUploadResult;
    modeLabel: string;
  }) => {
    mockCsvSaveResultNotice(props);
    return <div>CsvSaveResultNotice</div>;
  },
}));

const createProps = (
  overrides: Partial<ComponentProps<typeof DataActionRail>> = {},
): ComponentProps<typeof DataActionRail> => ({
  onFileSelect: vi.fn(),
  selectedFileName: 'import.csv',
  fileInputDisabled: false,
  hasCsvFile: false,
  saveLabel: '保存する',
  onSave: vi.fn(),
  saveDisabled: false,
  hasDbData: false,
  deleteLabel: '削除する',
  onDeleteRequest: vi.fn(),
  deleteDisabled: false,
  saveResult: null,
  saveModeLabel: '追加保存',
  ...overrides,
});

describe('DataActionRail', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('hasDbData=false かつ hasCsvFile=false のとき CSVFileInput のみ表示し保存/削除ボタンを表示しない', () => {
    render(<DataActionRail {...createProps()} />);

    expect(screen.getByText('CSVFileInput')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '保存する' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '削除する' })).not.toBeInTheDocument();
  });

  it('hasCsvFile=true のとき保存ボタンを表示しクリックで onSave を呼ぶ', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();

    render(<DataActionRail {...createProps({ hasCsvFile: true, onSave })} />);

    const saveButton = screen.getByRole('button', { name: '保存する' });

    expect(saveButton).toBeInTheDocument();

    await user.click(saveButton);

    expect(onSave).toHaveBeenCalledTimes(1);
  });

  it('hasDbData=true のとき削除ボタンを表示しクリックで onDeleteRequest を呼ぶ', async () => {
    const user = userEvent.setup();
    const onDeleteRequest = vi.fn();

    render(
      <DataActionRail
        {...createProps({ hasDbData: true, onDeleteRequest })}
      />,
    );

    const deleteButton = screen.getByRole('button', { name: '削除する' });

    expect(deleteButton).toBeInTheDocument();

    await user.click(deleteButton);

    expect(onDeleteRequest).toHaveBeenCalledTimes(1);
  });

  it('saveResult が null のとき保存結果通知を表示しない', () => {
    render(<DataActionRail {...createProps({ saveResult: null })} />);

    expect(screen.queryByText('CsvSaveResultNotice')).not.toBeInTheDocument();
    expect(mockCsvSaveResultNotice).not.toHaveBeenCalled();
  });

  it('saveResult があるとき CsvSaveResultNotice に結果を渡す', () => {
    const saveResult: CsvUploadResult = {
      inserted: 3,
      skipped: 1,
      errors: [],
    };

    render(<DataActionRail {...createProps({ saveResult })} />);

    expect(screen.getByText('CsvSaveResultNotice')).toBeInTheDocument();
    expect(mockCsvSaveResultNotice).toHaveBeenCalledWith({
      result: saveResult,
      modeLabel: '追加保存',
    });
  });

  it('saveDisabled=true のとき保存ボタンを disabled にする', () => {
    render(
      <DataActionRail
        {...createProps({ hasCsvFile: true, saveDisabled: true })}
      />,
    );

    expect(screen.getByRole('button', { name: '保存する' })).toBeDisabled();
  });
});
