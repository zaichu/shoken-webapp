import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { CSVFileInput } from './CSVFileInput';

describe('CSVFileInput コンポーネント', () => {
  it('正しくレンダリングされる', () => {
    const onFileSelect = vi.fn();
    render(<CSVFileInput onFileSelect={onFileSelect} />);

    // ボタンとプレースホルダーテキストが表示されていることを確認
    expect(screen.getByText('CSVファイル選択')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('CSVファイルを選択してください。')).toBeInTheDocument();
  });

  it('ファイル選択時にコールバックが呼ばれる', () => {
    const onFileSelect = vi.fn();
    render(<CSVFileInput onFileSelect={onFileSelect} />);

    // ファイル入力を取得
    const fileInput = screen.getByLabelText('CSVファイル選択');
    expect(fileInput).toBeInTheDocument();

    // ファイル選択イベントをシミュレート
    const file = new File(['test csv content'], 'test.csv', { type: 'text/csv' });
    fireEvent.change(fileInput, { target: { files: [file] } });

    // コールバックが呼ばれたことを確認
    expect(onFileSelect).toHaveBeenCalledTimes(1);
    expect(onFileSelect).toHaveBeenCalledWith(file);

    // テキスト入力にファイル名が表示されていることを確認
    expect(screen.getByDisplayValue('test.csv')).toBeInTheDocument();
  });

  it('ファイルが選択されていない場合にコールバックが呼ばれない', () => {
    const onFileSelect = vi.fn();
    render(<CSVFileInput onFileSelect={onFileSelect} />);

    // ファイル入力を取得
    const fileInput = screen.getByLabelText('CSVファイル選択');

    // 空のファイルリストでchangeイベントをシミュレート
    fireEvent.change(fileInput, { target: { files: [] } });

    // コールバックが呼ばれないことを確認
    expect(onFileSelect).not.toHaveBeenCalled();

    // プレースホルダーテキストが残っていることを確認
    expect(screen.getByPlaceholderText('CSVファイルを選択してください。')).toBeInTheDocument();
  });
});
