import { fireEvent, render, screen } from '@testing-library/react';
import { vi, describe, it, expect } from 'vitest';
import { CSVFileInput } from '../CSVFileInput';

describe('CSVFileInput', () => {
  it('アップロードトリガーと選択済みファイル名を縦積みで表示する', () => {
    render(
      <CSVFileInput
        onFileSelect={vi.fn()}
        selectedFileName="asset-balance.csv"
      />
    );

    expect(screen.getByTestId('csv-file-trigger')).toBeInTheDocument();
    expect(screen.getByLabelText('選択されたファイル名')).toHaveValue('asset-balance.csv');
    expect(screen.queryByText('選択したファイルをこのまま取り込みできます')).not.toBeInTheDocument();
  });

  it('ファイルを選択すると onFileSelect が呼ばれる', () => {
    const handleFileSelect = vi.fn();
    render(<CSVFileInput onFileSelect={handleFileSelect} />);

    const input = screen.getByLabelText('CSVファイルを選択') as HTMLInputElement;
    const file = new File(['test'], 'receipts.csv', { type: 'text/csv' });
    fireEvent.change(input, { target: { files: [file] } });

    expect(handleFileSelect).toHaveBeenCalledWith(file);
  });
});
