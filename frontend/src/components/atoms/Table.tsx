import { ReactNode, HTMLAttributes, TableHTMLAttributes } from 'react';

interface TableProps extends TableHTMLAttributes<HTMLTableElement> {
  children: ReactNode;
  striped?: boolean;
  bordered?: boolean;
  hover?: boolean;
  small?: boolean;
  responsive?: boolean | 'sm' | 'md' | 'lg' | 'xl' | 'xxl';
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'info' | 'light' | 'dark';
  className?: string;
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
}

export function Table({
  children,
  striped = false,
  bordered = false,
  hover = false,
  small = false,
  responsive = false,
  variant,
  className = '',
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

  const renderTable = () => (
    <table className={tableClasses}>
      {children}
    </table>
  );

  if (responsive) {
    const responsiveClass = typeof responsive === 'boolean'
      ? 'table-responsive'
      : `table-responsive-${responsive}`;

    return (
      <div className={responsiveClass} {...rest}>
        {renderTable()}
      </div>
    );
  }

  return (
    <div {...rest}>
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
  ...rest
}: TableCellProps) {
  const Cell = as;
  const scopeAttr = as === 'th' ? { scope } : {};

  return (
    <Cell className={className} {...scopeAttr} {...rest} colSpan={colSpan}>
      {children}
    </Cell>
  );
}