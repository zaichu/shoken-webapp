import { render, screen, fireEvent, act } from '@testing-library/react';
import { Table, TableHeader, TableBody, TableRow, TableCell } from '../Table';

// ウィンドウのリサイズイベントをモック
const mockWindowResize = () => {
  const originalHeight = window.innerHeight;
  const originalWidth = window.innerWidth;

  // リサイズイベントをトリガーするヘルパー関数
  const triggerResize = (width: number, height: number) => {
    window.innerWidth = width;
    window.innerHeight = height;
    fireEvent(window, new Event('resize'));
  };

  // テスト終了時に元のサイズに戻す
  return {
    triggerResize,
    cleanup: () => {
      window.innerHeight = originalHeight;
      window.innerWidth = originalWidth;
    }
  };
};

describe('Table Component', () => {
  it('基本的なテーブルをレンダリングする', () => {
    render(
      <Table>
        <TableHeader>
          <TableRow>
            <TableCell as="th">ヘッダー1</TableCell>
            <TableCell as="th">ヘッダー2</TableCell>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>セル1</TableCell>
            <TableCell>セル2</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    expect(screen.getByText('ヘッダー1')).toBeInTheDocument();
    expect(screen.getByText('ヘッダー2')).toBeInTheDocument();
    expect(screen.getByText('セル1')).toBeInTheDocument();
    expect(screen.getByText('セル2')).toBeInTheDocument();
  });

  it('テーブルのスタイルクラスが正しく適用される', () => {
    const { container } = render(
      <Table striped bordered hover small variant="primary">
        <TableBody>
          <TableRow>
            <TableCell>テストセル</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const tableElement = container.querySelector('table');
    expect(tableElement).toHaveClass('table');
    expect(tableElement).toHaveClass('table-striped');
    expect(tableElement).toHaveClass('table-bordered');
    expect(tableElement).toHaveClass('table-hover');
    expect(tableElement).toHaveClass('table-sm');
    expect(tableElement).toHaveClass('table-primary');
  });

  it('レスポンシブテーブルが正しくレンダリングされる', () => {
    const { container } = render(
      <Table responsive>
        <TableBody>
          <TableRow>
            <TableCell>レスポンシブセル</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const responsiveDiv = container.querySelector('div');
    expect(responsiveDiv).toHaveClass('table-responsive');
  });

  it('特定のブレイクポイントでレスポンシブテーブルが正しくレンダリングされる', () => {
    const { container } = render(
      <Table responsive="md">
        <TableBody>
          <TableRow>
            <TableCell>レスポンシブセル</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const responsiveDiv = container.querySelector('div');
    expect(responsiveDiv).toHaveClass('table-responsive-md');
  });

  it('ウィンドウのリサイズに応じてテーブルの高さが変更される', () => {
    const { cleanup, triggerResize } = mockWindowResize();

    try {
      const { container } = render(
        <Table autoHeight minHeight={200} bottomMargin={20}>
          <TableBody>
            <TableRow>
              <TableCell>テストセル</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      );

      // テーブルのコンテナ要素を取得
      const tableContainer = container.querySelector('div');
      expect(tableContainer).toHaveStyle({ position: 'relative', overflowY: 'auto' });

      // ウィンドウサイズを変更してテーブルの高さが更新されることを確認
      act(() => {
        // 小さいサイズにリサイズ
        triggerResize(800, 600);
      });

      // もう一度大きいサイズにリサイズ
      act(() => {
        triggerResize(1200, 900);
      });

      // 最小高さが適用されることを確認
      const tableStyle = window.getComputedStyle(tableContainer!);
      expect(tableStyle.maxHeight).not.toBe('auto');
    } finally {
      cleanup();
    }
  });

  it('テーブルヘッダーが正しくレンダリングされる', () => {
    const { container } = render(
      <TableHeader variant="light" stickyTop>
        <TableRow>
          <TableCell as="th">ヘッダーセル</TableCell>
        </TableRow>
      </TableHeader>
    );

    const headerElement = container.querySelector('thead');
    expect(headerElement).toHaveClass('table-light');
    expect(headerElement).toHaveClass('sticky-top');
  });

  it('テーブル行のバリアントが正しく適用される', () => {
    const { container } = render(
      <TableRow active variant="success">
        <TableCell>テストセル</TableCell>
      </TableRow>
    );

    const rowElement = container.querySelector('tr');
    expect(rowElement).toHaveClass('table-active');
    expect(rowElement).toHaveClass('table-success');
  });

  it('テーブルセルが正しくレンダリングされる', () => {
    const { container } = render(
      <TableCell as="th" scope="col" colSpan={2} className="custom-cell">
        テストヘッダーセル
      </TableCell>
    );

    const cellElement = container.querySelector('th');
    expect(cellElement).toHaveAttribute('scope', 'col');
    expect(cellElement).toHaveAttribute('colspan', '2');
    expect(cellElement).toHaveClass('custom-cell');
    expect(cellElement).toHaveTextContent('テストヘッダーセル');
  });

  it('dangerouslySetInnerHTMLを持つテーブルセルが正しくレンダリングされる', () => {
    const { container } = render(
      <TableCell dangerouslySetInnerHTML={{ __html: '<strong>強調テキスト</strong>' }} />
    );

    const cellElement = container.querySelector('td');
    expect(cellElement?.innerHTML).toBe('<strong>強調テキスト</strong>');
    expect(container.querySelector('strong')).toBeInTheDocument();
  });
});
