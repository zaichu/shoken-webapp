import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';
import { TableWithContext } from '../TableWithContext';
import * as ResizeContext from '@/contexts/ResizeContext';

// useForceResizeのモック
const mockUseForceResize = vi.fn();
vi.spyOn(ResizeContext, 'useForceResize').mockImplementation(mockUseForceResize);

// useTableAutoResizeのモック
vi.mock('../../../hooks/common/useTableAutoResize', () => ({
  useTableAutoResize: vi.fn(() => ({
    containerRef: { current: null },
    height: '400px'
  }))
}));

describe('TableWithContext', () => {
  beforeEach(() => {
    mockUseForceResize.mockReturnValue(0);
    vi.clearAllMocks();
  });

  it('基本的なテーブルをレンダリングする', () => {
    render(
      <TableWithContext>
        <tbody>
          <tr>
            <td>テストデータ</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    expect(screen.getByRole('table')).toBeInTheDocument();
    expect(screen.getByText('テストデータ')).toBeInTheDocument();
  });

  it('stripedプロパティが適用される', () => {
    render(
      <TableWithContext striped>
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('table-striped');
  });

  it('borderedプロパティが適用される', () => {
    render(
      <TableWithContext bordered>
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('table-bordered');
  });

  it('hoverプロパティが適用される', () => {
    render(
      <TableWithContext hover>
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('table-hover');
  });

  it('smallプロパティが適用される', () => {
    render(
      <TableWithContext small>
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('table-sm');
  });

  it('variantプロパティが適用される', () => {
    render(
      <TableWithContext variant="dark">
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('table-dark');
  });

  it('複数のプロパティが同時に適用される', () => {
    render(
      <TableWithContext striped bordered hover small variant="primary">
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('table');
    expect(table).toHaveClass('table-striped');
    expect(table).toHaveClass('table-bordered');
    expect(table).toHaveClass('table-hover');
    expect(table).toHaveClass('table-sm');
    expect(table).toHaveClass('table-primary');
  });

  it('responsiveがtrueの場合にラッパーが追加される', () => {
    render(
      <TableWithContext responsive>
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    const wrapper = screen.getByRole('table').parentElement;
    expect(wrapper).toHaveClass('table-responsive');
  });

  it('responsive breakpointが指定された場合に適切なクラスが適用される', () => {
    render(
      <TableWithContext responsive="md">
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    const wrapper = screen.getByRole('table').parentElement;
    expect(wrapper).toHaveClass('table-responsive-md');
  });

  it('autoHeightがfalseの場合はラッパーが追加されない', () => {
    render(
      <TableWithContext autoHeight={false} responsive={false}>
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    const table = screen.getByRole('table');
    expect(table.parentElement?.tagName).toBe('DIV'); // テストコンテナ
    expect(table.parentElement).not.toHaveClass('table-responsive');
  });

  it('カスタムクラスが適用される', () => {
    render(
      <TableWithContext className="custom-table">
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    const table = screen.getByRole('table');
    expect(table).toHaveClass('custom-table');
  });

  it('Context からの forceResize が useTableAutoResize に渡される', () => {
    const mockUseTableAutoResize = vi.fn(() => ({
      containerRef: { current: null },
      height: '400px'
    }));
    
    vi.doMock('../../../hooks/common/useTableAutoResize', () => ({
      useTableAutoResize: mockUseTableAutoResize
    }));

    mockUseForceResize.mockReturnValue(5);

    render(
      <TableWithContext>
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    expect(mockUseTableAutoResize).toHaveBeenCalledWith({
      enabled: true,
      minHeight: 200,
      maxHeight: undefined,
      bottomMargin: 20,
      forceResize: 5,
    });
  });

  it('プロパティが useTableAutoResize に正しく渡される', () => {
    const mockUseTableAutoResize = vi.fn(() => ({
      containerRef: { current: null },
      height: '400px'
    }));
    
    vi.doMock('../../../hooks/common/useTableAutoResize', () => ({
      useTableAutoResize: mockUseTableAutoResize
    }));

    render(
      <TableWithContext 
        autoHeight={false}
        minHeight={300}
        maxHeight={600}
        bottomMargin={30}
      >
        <tbody>
          <tr>
            <td>テスト</td>
          </tr>
        </tbody>
      </TableWithContext>
    );

    expect(mockUseTableAutoResize).toHaveBeenCalledWith({
      enabled: false,
      minHeight: 300,
      maxHeight: 600,
      bottomMargin: 30,
      forceResize: 0,
    });
  });
});
