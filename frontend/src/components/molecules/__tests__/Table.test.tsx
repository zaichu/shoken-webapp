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

  test('responsiveのときテーブルがスクロール用コンテナでラップされる', () => {
    render(
      <Table responsive>
        <TableBody>
          <TableRow>
            <TableCell>テストデータ</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    // 横スクロール可能にするためのラッパー構造（table が div の子になること）を検証する。
    // クラス名ではなく DOM 構造を見る。実寸（scrollWidth > clientWidth）の検証は
    // jsdom にレイアウトがないため不可で、実ブラウザの E2E で補う。
    const container = screen.getByRole('table').parentElement;
    expect(container?.tagName).toBe('DIV');
    expect(container?.contains(screen.getByRole('table'))).toBe(true);
  });

  test('responsive="md"のときテーブルがスクロール用コンテナでラップされる', () => {
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
    expect(container?.tagName).toBe('DIV');
    expect(container?.contains(screen.getByRole('table'))).toBe(true);
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

  test('TableRowは行としてレンダリングされる', () => {
    render(
      <Table>
        <TableBody>
          <TableRow active>
            <TableCell>アクティブ行</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    expect(screen.getByRole('row')).toHaveTextContent('アクティブ行');
  });

  test('variant指定のTableRowは行としてレンダリングされる', () => {
    render(
      <Table>
        <TableBody>
          <TableRow variant="success">
            <TableCell>成功行</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    );

    expect(screen.getByRole('row')).toHaveTextContent('成功行');
  });

  test('TableHeaderは行グループとしてレンダリングされる', () => {
    render(
      <Table>
        <TableHeader stickyTop>
          <TableRow>
            <TableCell as="th">スティッキーヘッダー</TableCell>
          </TableRow>
        </TableHeader>
      </Table>
    );

    expect(screen.getByRole('rowgroup')).toHaveTextContent('スティッキーヘッダー');
  });

  test('variant指定のTableHeaderは行グループとしてレンダリングされる', () => {
    render(
      <Table>
        <TableHeader variant="dark">
          <TableRow>
            <TableCell as="th">ダークヘッダー</TableCell>
          </TableRow>
        </TableHeader>
      </Table>
    );

    expect(screen.getByRole('rowgroup')).toHaveTextContent('ダークヘッダー');
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
