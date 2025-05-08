import { ReactNode, HTMLAttributes, TableHTMLAttributes, useState, useRef, useEffect, useCallback } from 'react';

// ウィンドウサイズの型を定義
interface WindowSize {
  width: number;
  height: number;
}

interface TableProps extends TableHTMLAttributes<HTMLTableElement> {
  children: ReactNode;
  striped?: boolean;
  bordered?: boolean;
  hover?: boolean;
  small?: boolean;
  responsive?: 'sm' | 'md' | 'lg' | 'xl' | 'xxl';
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'info' | 'light' | 'dark';
  className?: string;
  autoHeight?: boolean;
  minHeight?: number;
  maxHeight?: number | string;
  bottomMargin?: number;
}

interface TableHeaderProps extends HTMLAttributes<HTMLTableSectionElement> {
  children: ReactNode;
  variant?: 'light' | 'dark';
  stickyTop?: boolean;
  className?: string;
}

interface TableRowProps extends HTMLAttributes<HTMLTableRowElement> {
  children: ReactNode;
  active?: boolean;
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'info' | 'light' | 'dark';
  className?: string;
}

interface TableCellProps extends HTMLAttributes<HTMLTableCellElement> {
  children: ReactNode;
  as?: 'td' | 'th';
  scope?: 'col' | 'row' | 'colgroup' | 'rowgroup';
  colSpan?: number;
  className?: string;
  dangerouslySetInnerHTML?: { __html: string };
}

export function Table({
  children,
  striped = false,
  bordered = false,
  hover = false,
  small = false,
  responsive,
  variant,
  className = '',
  autoHeight = true,
  minHeight = 200,
  maxHeight,
  bottomMargin = 20,
  ...rest
}: TableProps) {
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

  const [tableHeight, setTableHeight] = useState<string>('auto');
  const [windowSize, setWindowSize] = useState<WindowSize>({
    width: typeof window !== 'undefined' ? window.innerWidth : 0,
    height: typeof window !== 'undefined' ? window.innerHeight : 0,
  });
  const containerRef = useRef<HTMLDivElement>(null);

  // テーブルの高さを計算する関数をuseCallbackでメモ化
  const calculateTableHeight = useCallback(() => {
    if (!autoHeight || !containerRef.current) return;

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
    setWindowSize({
      width: window.innerWidth,
      height: window.innerHeight,
    });
    calculateTableHeight();
  }, [calculateTableHeight]);

  // 初期レンダリング時にテーブルの高さを計算
  useEffect(() => {
    // ResizeObserverを使用してコンテナのサイズ変更を監視
    if (containerRef.current) {
      const resizeObserver = new ResizeObserver(() => {
        calculateTableHeight();
      });

      resizeObserver.observe(containerRef.current);

      // クリーンアップ関数でObserverを解除
      return () => {
        if (containerRef.current) {
          resizeObserver.unobserve(containerRef.current);
        }
        resizeObserver.disconnect();
      };
    }
  }, [calculateTableHeight]);

  // ウィンドウのリサイズイベントを監視
  useEffect(() => {
    window.addEventListener('resize', handleResize);

    // 初期計算
    calculateTableHeight();

    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, [handleResize, calculateTableHeight]);

  // ウィンドウサイズ変更時に高さを再計算
  useEffect(() => {
    calculateTableHeight();
  }, [windowSize, calculateTableHeight]);

  const renderTable = () => (
    <table className={tableClasses}>
      {children}
    </table>
  );

  // スタイルオブジェクトの作成
  const containerStyle = {
    maxHeight: autoHeight ? tableHeight : maxHeight,
    overflowY: 'auto' as const,
    position: 'relative' as const,
  };

  const responsiveClass = responsive ? `table-responsive-${responsive}` : 'table-responsive';
  return (
    <div
      className={responsiveClass}
      ref={containerRef}
      style={containerStyle}
      {...rest}
    >
      {renderTable()}
    </div>
  );

}

export function TableHeader({
  children,
  variant,
  stickyTop = true,
  className = '',
  ...rest
}: TableHeaderProps) {
  const variantClass = variant ? `table-${variant}` : '';
  const stickyClass = stickyTop ? 'sticky-top' : '';

  const headClasses = [
    variantClass,
    stickyClass,
    className
  ].filter(Boolean).join(' ');

  return (
    <thead className={headClasses} {...rest}>
      {children}
    </thead>
  );
}

export function TableBody({
  children,
  className = '',
  ...rest
}: HTMLAttributes<HTMLTableSectionElement>) {
  return (
    <tbody className={className} {...rest}>
      {children}
    </tbody>
  );
}

export function TableRow({
  children,
  active = false,
  variant,
  className = '',
  ...rest
}: TableRowProps) {
  const activeClass = active ? 'table-active' : '';
  const variantClass = variant ? `table-${variant}` : '';

  const rowClasses = [
    activeClass,
    variantClass,
    className
  ].filter(Boolean).join(' ');

  return (
    <tr className={rowClasses} {...rest}>
      {children}
    </tr>
  );
}

export function TableCell({
  children,
  as = 'td',
  scope = 'col',
  colSpan = 1,
  className = '',
  dangerouslySetInnerHTML,
  ...rest
}: TableCellProps) {
  const Cell = as;
  const scopeAttr = as === 'th' ? { scope } : {};

  if (dangerouslySetInnerHTML) {
    return (
      <Cell
        className={className}
        {...scopeAttr}
        {...rest}
        colSpan={colSpan}
        dangerouslySetInnerHTML={dangerouslySetInnerHTML}
      />
    );
  }

  return (
    <Cell className={className} {...scopeAttr} {...rest} colSpan={colSpan}>
      {children}
    </Cell>
  );
}
