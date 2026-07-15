import { render, screen, waitFor } from '@testing-library/react';
import { vi, describe, test, expect, beforeEach } from 'vitest';
import '@testing-library/jest-dom';
import { Table, TableHeader, TableBody, TableRow, TableCell } from '../Table';

// ResizeObserverのモック（class形式で定義）
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
global.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;

// windowオブジェクトのモック
Object.defineProperty(window, 'innerHeight', {
  writable: true,
  configurable: true,
  value: 1024,
});

Object.defineProperty(window, 'innerWidth', {
  writable: true,
  configurable: true,
  value: 1280,
});

describe('Table', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  test('基本的なテーブルが正しくレンダリングされる', () => {
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
            <TableCell>データ1</TableCell>
            <TableCell>データ2</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    expect(screen.getByRole('table')).toBeInTheDocument();
    expect(screen.getByText('ヘッダー1')).toBeInTheDocument();
    expect(screen.getByText('ヘッダー2')).toBeInTheDocument();
    expect(screen.getByText('データ1')).toBeInTheDocument();
    expect(screen.getByText('データ2')).toBeInTheDocument();
  });

  test('stripedプロパティが正しく適用される', () => {
    render(
      <Table striped>
        <TableBody>
          <TableRow>
            <TableCell>テストデータ</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('[&_tbody_tr:nth-child(even)]:bg-slate-50/80');
  });

  test('borderedプロパティが正しく適用される', () => {
    render(
      <Table bordered>
        <TableBody>
          <TableRow>
            <TableCell>テストデータ</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('[&_th]:border');
  });

  test('hoverプロパティが正しく適用される', () => {
    render(
      <Table hover>
        <TableBody>
          <TableRow>
            <TableCell>テストデータ</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('[&_tbody_tr:hover]:bg-amber-50/60');
  });

  test('smallプロパティが正しく適用される', () => {
    render(
      <Table small>
        <TableBody>
          <TableRow>
            <TableCell>テストデータ</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('text-[12px]');
  });

  test('responsiveプロパティが正しく適用される', () => {
    render(
      <Table responsive>
        <TableBody>
          <TableRow>
            <TableCell>テストデータ</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const container = screen.getByRole('table').parentElement;
    expect(container).toHaveClass('overflow-x-hidden');
    expect(container).not.toHaveClass('overflow-x-auto');
    expect(screen.getByRole('table')).toHaveClass('table-fixed');
  });

  test('responsive="md"プロパティが正しく適用される', () => {
    render(
      <Table responsive="md">
        <TableBody>
          <TableRow>
            <TableCell>テストデータ</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const container = screen.getByRole('table').parentElement;
    expect(container).toHaveClass('overflow-x-hidden');
  });

  test('variantプロパティが正しく適用される', () => {
    render(
      <Table variant="primary">
        <TableBody>
          <TableRow>
            <TableCell>テストデータ</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('bg-slate-950/10');
  });

  test('autoHeight テーブルが再レンダリング後も表示される', async () => {
    const { rerender } = render(
      <Table autoHeight>
        <TableBody>
          <TableRow>
            <TableCell>テストデータ</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    // autoHeight のまま再レンダリング
    rerender(
      <Table autoHeight>
        <TableBody>
          <TableRow>
            <TableCell>テストデータ</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    // コンポーネントが再レンダリングされることを確認
    await waitFor(() => {
      expect(screen.getByRole('table')).toBeInTheDocument();
    }, { timeout: 200 });
  });

  test('autoHeightがfalseの場合でも正常にレンダリングされる', () => {
    render(
      <Table autoHeight={false}>
        <TableBody>
          <TableRow>
            <TableCell>テストデータ</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    expect(screen.getByRole('table')).toBeInTheDocument();
  });

  test('childrenが正しく表示される', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell><strong>HTML内容</strong></TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const cell = screen.getByRole('cell');
    expect(cell.innerHTML).toBe('<strong>HTML内容</strong>');
  });

  test('colSpanが正しく適用される', () => {
    render(
      <Table>
        <TableBody>
          <TableRow>
            <TableCell colSpan={2}>結合セル</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const cell = screen.getByRole('cell');
    expect(cell).toHaveAttribute('colspan', '2');
  });

  test('TableRowのactiveプロパティが正しく適用される', () => {
    render(
      <Table>
        <TableBody>
          <TableRow active>
            <TableCell>アクティブ行</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const row = screen.getByRole('row');
    expect(row).toHaveClass('bg-amber-50');
  });

  test('TableRowのvariantプロパティが正しく適用される', () => {
    render(
      <Table>
        <TableBody>
          <TableRow variant="success">
            <TableCell>成功行</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    const row = screen.getByRole('row');
    expect(row).toHaveClass('bg-teal-50');
  });

  test('TableHeaderのstickyTopプロパティが正しく適用される', () => {
    render(
      <Table>
        <TableHeader stickyTop>
          <TableRow>
            <TableCell as="th">スティッキーヘッダー</TableCell>
          </TableRow>
        </TableHeader>
      </Table>
    );

    const thead = screen.getByRole('rowgroup');
    expect(thead).toHaveClass('sticky', 'top-0');
  });

  test('TableHeaderのvariantプロパティが正しく適用される', () => {
    render(
      <Table>
        <TableHeader variant="dark">
          <TableRow>
            <TableCell as="th">ダークヘッダー</TableCell>
          </TableRow>
        </TableHeader>
      </Table>
    );

    const thead = screen.getByRole('rowgroup');
    expect(thead).toHaveClass('bg-slate-950', 'text-white');
  });

  test('TableCellのscopeプロパティがthの場合に正しく適用される', () => {
    render(
      <Table>
        <TableHeader>
          <TableRow>
            <TableCell as="th" scope="col">列ヘッダー</TableCell>
          </TableRow>
        </TableHeader>
      </Table>
    );

    const th = screen.getByRole('columnheader');
    expect(th).toHaveAttribute('scope', 'col');
  });
});
