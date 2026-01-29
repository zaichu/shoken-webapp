import { ReactNode, HTMLAttributes, TableHTMLAttributes, forwardRef } from 'react';
import { useTableAutoResize } from '../../hooks/common/useTableAutoResize';

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
      className = '',
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

    // CSSクラスの構築
    const buildTableClasses = () => {
      const classes = ['w-full text-left border-collapse'];

      if (bordered) classes.push('[&_th]:border [&_th]:border-gray-200 [&_td]:border [&_td]:border-gray-200 print:[&_th]:border-black print:[&_td]:border-black');
      if (small) classes.push('text-sm [&_th]:py-1 [&_th]:px-2 [&_td]:py-1 [&_td]:px-2');
      else classes.push('[&_th]:py-2 [&_th]:px-3 [&_td]:py-2 [&_td]:px-3');
      if (variant) classes.push(variantBgColors[variant] || '');
      if (className) classes.push(className);

      return classes.join(' ');
    };

    // striped と hover は tbody に適用
    const stripedClass = striped ? '[&_tbody_tr:nth-child(even)]:bg-gray-50' : '';
    const hoverClass = hover ? '[&_tbody_tr:hover]:bg-gray-100' : '';

    const table = (
      <table ref={ref} className={`${buildTableClasses()} ${stripedClass} ${hoverClass}`} {...rest}>
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
          className="w-full overflow-x-auto rounded-md"
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
    const buildHeaderClasses = () => {
      const classes = [];

      if (variant === 'light') classes.push('bg-gray-100');
      if (variant === 'dark') classes.push('bg-gray-800 text-white');
      if (stickyTop) classes.push('sticky top-0 z-10 bg-white');
      if (className) classes.push(className);

      return classes.join(' ');
    };

    return (
      <thead ref={ref} className={buildHeaderClasses()} {...rest}>
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
    const buildRowClasses = () => {
      const classes = ['[&_th]:align-middle [&_td]:align-middle'];

      if (active) classes.push('bg-primary/10');
      if (variant) classes.push(variantBgColors[variant] || '');
      if (className) classes.push(className);

      return classes.join(' ');
    };

    return (
      <tr ref={ref} className={buildRowClasses()} {...rest}>
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
    const thClasses = as === 'th' ? 'font-semibold text-gray-700' : '';

    if (dangerouslySetInnerHTML) {
      return (
        <Cell
          ref={ref}
          className={`${thClasses} ${className}`}
          {...scopeAttr}
          {...rest}
          colSpan={colSpan}
          dangerouslySetInnerHTML={dangerouslySetInnerHTML}
        />
      );
    }

    return (
      <Cell ref={ref} className={`${thClasses} ${className}`} {...scopeAttr} {...rest} colSpan={colSpan}>
        {children}
      </Cell>
    );
  }
);

TableCell.displayName = 'TableCell';

export { Table, TableHeader, TableBody, TableRow, TableCell };
