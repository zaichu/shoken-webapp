import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ReceiptsAlerts } from '../ReceiptsAlerts';

const csvPreview = {
  totalRows: 4,
  validRows: 3,
  errors: [{ row: 2, message: '金額が不正です' }],
  rows: [],
};

describe('ReceiptsAlerts', () => {
  it('dbError があればエラーアラートが表示される', () => {
    render(
      <ReceiptsAlerts
        dbError="DB からの取得に失敗しました"
        hasCsvFile={false}
        previewing={false}
        csvPreview={null}
      />
    );

    expect(screen.getByRole('alert')).toHaveTextContent('エラー:');
    expect(screen.getByRole('alert')).toHaveTextContent('DB からの取得に失敗しました');
  });

  it('CSV プレビュー条件を満たせばプレビュー情報が表示される', () => {
    render(
      <ReceiptsAlerts
        dbError={null}
        hasCsvFile
        previewing={false}
        csvPreview={{ ...csvPreview, errors: [] }}
      />
    );

    expect(screen.getByRole('status')).toHaveTextContent('3件 追加で保存されます');
    expect(screen.getByRole('status')).toHaveTextContent('（保存モード: 追加）');
  });

  it('エラー行がある場合はエラーリストが表示される', () => {
    render(
      <ReceiptsAlerts
        dbError={null}
        hasCsvFile
        previewing={false}
        csvPreview={csvPreview}
      />
    );

    expect(screen.getByText('2行目: 金額が不正です')).toBeInTheDocument();
    expect(screen.getByRole('alert')).toHaveTextContent('1件エラー');
  });

  it('条件が満たされない場合は何も表示しない', () => {
    const { container } = render(
      <ReceiptsAlerts
        dbError={null}
        hasCsvFile={false}
        previewing={false}
        csvPreview={null}
      />
    );

    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
    expect(container).toBeEmptyDOMElement();
  });
});
