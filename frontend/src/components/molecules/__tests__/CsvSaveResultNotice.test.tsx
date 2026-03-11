import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CsvSaveResultNotice } from '../CsvSaveResultNotice';

describe('CsvSaveResultNotice', () => {
  it('保存結果をシンプルな confirmation strip として表示する', () => {
    render(
      <CsvSaveResultNotice
        modeLabel="追加保存"
        result={{ inserted: 12, skipped: 3, errors: [] }}
      />
    );

    expect(screen.getByTestId('csv-save-result-notice')).toBeInTheDocument();
    expect(screen.getByText('保存しました')).toBeInTheDocument();
    expect(screen.getByText('12件反映')).toBeInTheDocument();
    expect(screen.getByText('追加保存')).toBeInTheDocument();
    expect(screen.getByText('3件スキップ')).toBeInTheDocument();
  });

  it('エラーがある場合だけ詳細を開ける', () => {
    render(
      <CsvSaveResultNotice
        modeLabel="全件置換"
        result={{ inserted: 5, skipped: 0, errors: [{ row: 8, message: '銘柄コードが不正です' }] }}
      />
    );

    expect(screen.getByText('1件エラー')).toBeInTheDocument();
    expect(screen.getByText('エラー詳細を表示')).toBeInTheDocument();
    expect(screen.getByText('8行目: 銘柄コードが不正です')).toBeInTheDocument();
  });
});
