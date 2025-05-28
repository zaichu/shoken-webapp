import { ReactNode, TableHTMLAttributes, forwardRef, useEffect } from 'react';
import { useForceResize } from '@/contexts/ResizeContext';
import { useTableAutoResize } from '../../hooks/common/useTableAutoResize';

export interface TableWithContextProps extends TableHTMLAttributes<HTMLTableElement> {
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

const TableWithContext = forwardRef<HTMLTableElement, TableWithContextProps>(
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
    // Context から forceResize を取得
    const contextForceResize = useForceResize();

    // 自動リサイズ機能（Contextからのトリガーを含む）
    const { containerRef, height } = useTableAutoResize({
      enabled: autoHeight,
      minHeight,
      maxHeight,
      bottomMargin,
      forceResize: contextForceResize,
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

TableWithContext.displayName = 'TableWithContext';

export { TableWithContext };
