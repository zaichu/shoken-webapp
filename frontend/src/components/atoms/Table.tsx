import React, { ReactNode, HTMLAttributes, TableHTMLAttributes, forwardRef, useEffect, useRef, useState, useCallback } from 'react';

// ウィンドウサイズの型を定義
interface WindowSize {
  width: number;
  height: number;
}

export interface TableProps extends TableHTMLAttributes<HTMLTableElement> {
  children: ReactNode;
  striped?: boolean;
  bordered?: boolean;
  hover?: boolean;
  small?: boolean;
  responsive?: boolean | 'sm' | 'md' | 'lg' | 'xl' | 'xxl';
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'info' | 'light' | 'dark';
  autoHeight?: boolean;
  minHeight?: number;
  maxHeight?: number | string;
  bottomMargin?: number;
}

export interface TableHeaderProps extends HTMLAttributes<HTMLTableSectionElement> {
  children: ReactNode;
  variant?: 'light' | 'dark';
  stickyTop?: boolean;
  className?: string;
}

export interface TableRowProps extends HTMLAttributes<HTMLTableRowElement> {
  children: ReactNode;
  active?: boolean;
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'info' | 'light' | 'dark';
}

export interface TableCellProps extends HTMLAttributes<HTMLTableCellElement> {
  children?: ReactNode;
  as?: 'td' | 'th';
  scope?: 'col' | 'row' | 'colgroup' | 'rowgroup';
  colSpan?: number;
  dangerouslySetInnerHTML?: { __html: string };
}

const Table = forwardRef<HTMLTableElement, TableProps>(
  (
    {
      children,
      striped = false,
      bordered = false,
      hover = false,
      small = false,
      responsive,
      variant,
      autoHeight = true,
      minHeight = 200,
      maxHeight,
      bottomMargin = 20,
      className = '',
      ...rest
    },
    ref
  ) => {
    const [tableHeight, setTableHeight] = useState<string>('auto');
    const [windowSize, setWindowSize] = useState<WindowSize>(() => ({
      width: typeof window !== 'undefined' ? window.innerWidth : 0,
      height: typeof window !== 'undefined' ? window.innerHeight : 0,
    }));
    const containerRef = useRef<HTMLDivElement>(null);

    // テーブルの高さを計算する関数をuseCallbackでメモ化
    const calculateTableHeight = useCallback(() => {
      if (!autoHeight || !containerRef.current || typeof window === 'undefined') return;

      const rect = containerRef.current.getBoundingClientRect();
      const availableHeight = window.innerHeight - rect.top - bottomMargin;

      // 最小高さと最大高さの制約を適用
      let finalHeight = Math.max(availableHeight, minHeight);
      if (typeof maxHeight === 'number') {
        finalHeight = Math.min(finalHeight, maxHeight);
      }

      setTableHeight(`${finalHeight}px`);
    }, [autoHeight, minHeight, maxHeight, bottomMargin]);

    // リサイズハンドラーをuseCallbackでメモ化
    const handleResize = useCallback(() => {
      if (typeof window === 'undefined') return;

      setWindowSize({
        width: window.innerWidth,
        height: window.innerHeight,
      });
      calculateTableHeight();
    }, [calculateTableHeight]);

    // 初期レンダリング時にテーブルの高さを計算
    useEffect(() => {
      if (typeof window === 'undefined') return;

      // ResizeObserverを使用してコンテナのサイズ変更を監視
      const currentContainer = containerRef.current;
      if (currentContainer && 'ResizeObserver' in window) {
        const resizeObserver = new ResizeObserver(() => {
          calculateTableHeight();
        });

        resizeObserver.observe(currentContainer);

        // クリーンアップ関数でObserverを解除
        return () => {
          resizeObserver.unobserve(currentContainer);
          resizeObserver.disconnect();
        };
      }
    }, [calculateTableHeight]);

    // ウィンドウのリサイズイベントを監視
    useEffect(() => {
      if (typeof window === 'undefined') return;

      // ウィンドウサイズ変更時に高さを再計算
      window.addEventListener('resize', handleResize);

      // 初回計算
      calculateTableHeight();

      // クリーンアップ
      return () => {
        window.removeEventListener('resize', handleResize);
      };
    }, [handleResize, calculateTableHeight]);

    // ウィンドウサイズ変更時に高さを再計算
    useEffect(() => {
      calculateTableHeight();
    }, [windowSize, calculateTableHeight]);

    const baseClasses = 'table';
    const stripedClass = striped ? 'table-striped' : '';
    const borderedClass = bordered ? 'table-bordered' : '';
    const hoverClass = hover ? 'table-hover' : '';
    const smallClass = small ? 'table-sm' : '';
    const variantClass = variant ? `table-${variant}` : '';

    const tableClasses = [
      baseClasses,
      stripedClass,
      borderedClass,
      hoverClass,
      smallClass,
      variantClass,
      className
    ].filter(Boolean).join(' ');

    const table = (
      <table ref={ref} className={tableClasses} {...rest}>
        {children}
      </table>
    );

    if (responsive || autoHeight) {
      const responsiveClass = responsive === true
        ? 'table-responsive'
        : responsive
          ? `table-responsive-${responsive}`
          : autoHeight
            ? 'table-responsive'
            : '';

      // スタイルオブジェクトの作成
      const containerStyle = autoHeight
        ? {
          position: 'relative' as const,
          overflowY: 'auto' as const,
          maxHeight: tableHeight
        }
        : {};

      const containerClasses = [responsiveClass].filter(Boolean).join(' ');

      return (
        <div
          ref={containerRef}
          className={containerClasses}
          style={containerStyle}
        >
          {table}
        </div>
      );
    }

    return table;
  }
);

Table.displayName = 'Table';

const TableHeader = forwardRef<HTMLTableSectionElement, TableHeaderProps>(
  ({ children, variant, stickyTop = true, className = '', ...rest }, ref) => {
    const variantClass = variant ? `table-${variant}` : '';
    const stickyClass = stickyTop ? 'sticky-top' : '';

    const headClasses = [variantClass, stickyClass, className].filter(Boolean).join(' ');

    return (
      <thead ref={ref} className={headClasses} {...rest}>
        {children}
      </thead>
    );
  }
);

TableHeader.displayName = 'TableHeader';

const TableBody = forwardRef<HTMLTableSectionElement, HTMLAttributes<HTMLTableSectionElement>>(
  ({ children, className = '', ...rest }, ref) => {
    return (
      <tbody ref={ref} className={className} {...rest}>
        {children}
      </tbody>
    );
  }
);

TableBody.displayName = 'TableBody';

const TableRow = forwardRef<HTMLTableRowElement, TableRowProps>(
  ({ children, active = false, variant, className = '', ...rest }, ref) => {
    const activeClass = active ? 'table-active' : '';
    const variantClass = variant ? `table-${variant}` : '';

    const rowClasses = [activeClass, variantClass, className].filter(Boolean).join(' ');

    return (
      <tr ref={ref} className={rowClasses} {...rest}>
        {children}
      </tr>
    );
  }
);

TableRow.displayName = 'TableRow';

const TableCell = forwardRef<HTMLTableCellElement, TableCellProps>(
  (
    {
      children,
      as = 'td',
      scope = 'col',
      colSpan = 1,
      className = '',
      dangerouslySetInnerHTML,
      ...rest
    },
    ref
  ) => {
    const Cell = as;
    const scopeAttr = as === 'th' ? { scope } : {};

    if (dangerouslySetInnerHTML) {
      return (
        <Cell
          ref={ref}
          className={className}
          {...scopeAttr}
          {...rest}
          colSpan={colSpan}
          dangerouslySetInnerHTML={dangerouslySetInnerHTML}
        />
      );
    }

    return (
      <Cell ref={ref} className={className} {...scopeAttr} {...rest} colSpan={colSpan}>
        {children}
      </Cell>
    );
  }
);

TableCell.displayName = 'TableCell';

export { Table, TableHeader, TableBody, TableRow, TableCell };
