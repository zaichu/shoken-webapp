import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ReceiptsAlerts } from '../ReceiptsAlerts';
import type { CsvPreview } from '../receiptsReducer';

function makeCsvPreview(overrides: Partial<CsvPreview> = {}): CsvPreview {
  return {
    totalRows: 1,
    validRows: 1,
    errors: [],
    rows: [],
    ...overrides,
  };
}

describe('ReceiptsAlerts', () => {
  it('csvPreview.errors があるとエラーリストを表示する', () => {
    render(
      <ReceiptsAlerts
        dbError={null}
        hasCsvFile
        previewing={false}
        csvPreview={makeCsvPreview({
          errors: [{ row: 2, message: '金額が不正です' }],
        })}
      />
    );

    expect(screen.getByText('2行目: 金額が不正です')).toBeInTheDocument();
    expect(screen.getByRole('alert')).toHaveClass('bg-warning/15');
  });

  it('csvPreview.errors が空なら warning バリアントを使わない', () => {
    render(
      <ReceiptsAlerts
        dbError={null}
        hasCsvFile
        previewing={false}
        csvPreview={makeCsvPreview()}
      />
    );

    const alert = screen.getByRole('alert');
    expect(alert).toHaveClass('bg-info/10');
    expect(alert).not.toHaveClass('bg-warning/15');
  });

  it('dbError があるとエラーメッセージを表示する', () => {
    render(
      <ReceiptsAlerts
        dbError="保存に失敗しました"
        hasCsvFile={false}
        previewing={false}
        csvPreview={null}
      />
    );

    expect(screen.getByRole('alert')).toHaveTextContent('エラー: 保存に失敗しました');
  });

  it('hasCsvFile=false なら csvPreview セクションを表示しない', () => {
    render(
      <ReceiptsAlerts
        dbError={null}
        hasCsvFile={false}
        previewing={false}
        csvPreview={makeCsvPreview()}
      />
    );

    expect(screen.queryByRole('status')).not.toBeInTheDocument();
    expect(screen.queryByText('1件 追加で保存されます')).not.toBeInTheDocument();
  });
});
