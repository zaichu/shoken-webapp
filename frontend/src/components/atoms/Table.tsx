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
  forceResize?: number; // 外部からの強制リサイズトリガー
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
      forceResize,
      className = '',
      ...rest
    },
    ref
  ) => {
    // 自動リサイズ機能
    const { containerRef, height } = useTableAutoResize({
      enabled: autoHeight,
      minHeight,
      maxHeight,
      bottomMargin,
      forceResize,
    });

    // CSSクラスの構築
    const buildTableClasses = () => {
      const classes = ['table'];
      
      if (striped) classes.push('table-striped');
      if (bordered) classes.push('table-bordered');
      if (hover) classes.push('table-hover');
      if (small) classes.push('table-sm');
      if (variant) classes.push(`table-${variant}`);
      if (className) classes.push(className);
      
      return classes.join(' ');
    };

    // レスポンシブクラスの構築
    const buildResponsiveClass = () => {
      if (responsive === true) return 'table-responsive';
      if (typeof responsive === 'string') return `table-responsive-${responsive}`;
      if (autoHeight) return 'table-responsive';
      return '';
    };

    const table = (
      <table ref={ref} className={buildTableClasses()} {...rest}>
        {children}
      </table>
    );

    // レスポンシブまたは自動高さが有効な場合はラッパーで包む
    if (responsive || autoHeight) {
      const responsiveClass = buildResponsiveClass();
      
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
          className={responsiveClass}
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
      
      if (variant) classes.push(`table-${variant}`);
      if (stickyTop) classes.push('sticky-top');
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
      const classes = [];
      
      if (active) classes.push('table-active');
      if (variant) classes.push(`table-${variant}`);
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
