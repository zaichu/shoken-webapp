import { ReactNode, HTMLAttributes, TableHTMLAttributes, forwardRef } from 'react';
import { useTableAutoResize } from '../../hooks/common/useTableAutoResize';
import { cn } from '../../lib/utils/classNames';

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
  forceResize?: number;
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

// バリアント別の背景色
const variantBgColors: Record<string, string> = {
  primary: 'bg-primary/10',
  secondary: 'bg-secondary/10',
  success: 'bg-success/10',
  danger: 'bg-danger/10',
  warning: 'bg-warning/10',
  info: 'bg-info/10',
  light: 'bg-light',
  dark: 'bg-dark text-white',
};

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
      forceResize,
      className,
      ...rest
    },
    ref
  ) => {
    const { containerRef, height } = useTableAutoResize({
      enabled: autoHeight,
      minHeight,
      maxHeight,
      bottomMargin,
      forceResize,
    });

    // CSSクラスの構築（モバイル対応: パディング縮小、フォント調整）
    const tableClasses = cn(
      'w-full text-left border-collapse',
      small ? 'text-xs sm:text-sm' : 'text-sm sm:text-base',
      bordered && '[&_th]:border [&_th]:border-slate-200 [&_td]:border [&_td]:border-slate-200 print:[&_th]:border-black print:[&_td]:border-black',
      small
        ? '[&_th]:py-1.5 [&_th]:px-2 [&_td]:py-1.5 [&_td]:px-2 sm:[&_th]:py-2.5 sm:[&_th]:px-3 sm:[&_td]:py-2.5 sm:[&_td]:px-3'
        : '[&_th]:py-2 [&_th]:px-2 [&_td]:py-2 [&_td]:px-2 sm:[&_th]:py-3 sm:[&_th]:px-4 sm:[&_td]:py-3 sm:[&_td]:px-4',
      variant && variantBgColors[variant],
      striped && '[&_tbody_tr:nth-child(even)]:bg-slate-50',
      hover && '[&_tbody_tr:hover]:bg-slate-100',
      className
    );

    const table = (
      <table ref={ref} className={tableClasses} {...rest}>
        {children}
      </table>
    );

    // レスポンシブまたは自動高さが有効な場合はラッパーで包む
    if (responsive || autoHeight) {
      const containerStyle = autoHeight
        ? {
            position: 'relative' as const,
            overflowY: 'auto' as const,
            maxHeight: height,
          }
        : {};

      return (
        <div
          ref={containerRef}
          className={cn(
            'w-full overflow-x-auto rounded-md',
            // モバイル時のスクロールヒント（右端に影）
            'sm:shadow-none',
            '[&::-webkit-scrollbar]:h-2 [&::-webkit-scrollbar-track]:bg-slate-100 [&::-webkit-scrollbar-thumb]:bg-slate-300 [&::-webkit-scrollbar-thumb]:rounded'
          )}
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
  ({ children, variant, stickyTop = true, className, ...rest }, ref) => {
    const headerClasses = cn(
      'bg-slate-50',
      variant === 'light' && 'bg-slate-100',
      variant === 'dark' && 'bg-slate-800 text-white',
      stickyTop && 'sticky top-0 z-10',
      className
    );

    return (
      <thead ref={ref} className={headerClasses} {...rest}>
        {children}
      </thead>
    );
  }
);

TableHeader.displayName = 'TableHeader';

const TableBody = forwardRef<HTMLTableSectionElement, HTMLAttributes<HTMLTableSectionElement>>(
  ({ children, className, ...rest }, ref) => {
    return (
      <tbody ref={ref} className={cn(className)} {...rest}>
        {children}
      </tbody>
    );
  }
);

TableBody.displayName = 'TableBody';

const TableRow = forwardRef<HTMLTableRowElement, TableRowProps>(
  ({ children, active = false, variant, className, ...rest }, ref) => {
    const rowClasses = cn(
      '[&_th]:align-middle [&_td]:align-middle',
      active && 'bg-primary/10',
      variant && variantBgColors[variant],
      className
    );

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
      className,
      dangerouslySetInnerHTML,
      ...rest
    },
    ref
  ) => {
    const Cell = as;
    const scopeAttr = as === 'th' ? { scope } : {};
    const cellClasses = cn(as === 'th' && 'font-semibold text-slate-700', className);

    if (dangerouslySetInnerHTML) {
      return (
        <Cell
          ref={ref}
          className={cellClasses}
          {...scopeAttr}
          {...rest}
          colSpan={colSpan}
          dangerouslySetInnerHTML={dangerouslySetInnerHTML}
        />
      );
    }

    return (
      <Cell ref={ref} className={cellClasses} {...scopeAttr} {...rest} colSpan={colSpan}>
        {children}
      </Cell>
    );
  }
);

TableCell.displayName = 'TableCell';

export { Table, TableHeader, TableBody, TableRow, TableCell };
