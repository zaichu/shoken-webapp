import React from 'react';
import { cn } from '../../lib/utils/classNames';

const defaultValueFormat = (value: number): string => value.toLocaleString('ja-JP');
const defaultRateFormat = (rate: number): string => `${rate.toFixed(2)}%`;

export interface StatItemProps {
  title: string;
  value: string | React.ReactNode;
  className?: string;
  titleClassName?: string;
  valueClassName?: string;
  variant?: 'default' | 'card' | 'inline' | 'flat';
}

/**
 * 統計情報アイテムを表示するコンポーネント
 * タイトルと値をセットで表示
 */
export const StatItem: React.FC<StatItemProps> = ({
  title,
  value,
  className,
  titleClassName,
  valueClassName,
  variant = 'default'
}) => {
  const getVariantClasses = () => {
    switch (variant) {
      case 'card':
        return {
          container: 'bg-white rounded-lg shadow-sm border border-border p-3 transition-transform hover:-translate-y-0.5',
          title: 'text-base font-semibold text-secondary',
          value: 'text-xl font-bold'
        };
      case 'inline':
        return {
          container: 'flex justify-between items-center py-3 border-b border-gray-200 last:border-b-0',
          title: 'text-secondary',
          value: 'font-bold'
        };
      case 'flat':
        return {
          container: 'rounded-lg px-4 py-3',
          title: 'text-xs font-medium text-slate-600 mb-1',
          value: 'text-2xl font-bold tabular-nums'
        };
      default:
        return {
          container: '',
          title: 'text-base font-semibold',
          value: 'text-xl font-bold'
        };
    }
  };

  const variantClasses = getVariantClasses();

  return (
    <div className={cn(variantClasses.container, className)}>
      <div className={cn(variantClasses.title, titleClassName)}>
        {title}
      </div>
      <div className={cn(variantClasses.value, valueClassName)}>
        {value}
      </div>
    </div>
  );
};

export interface StatItemWithRateProps {
  title: string;
  value: number;
  rate?: number;
  format?: (value: number) => string;
  rateFormat?: (rate: number) => string;
  className?: string;
  showRate?: boolean;
  rateClassName?: string;
  variant?: StatItemProps['variant'];
}

/**
 * 統計情報アイテムを表示するコンポーネント（レート表示付き）
 * 金額などの値とレート（パーセンテージ）を併せて表示
 */
export const StatItemWithRate: React.FC<StatItemWithRateProps> = ({
  title,
  value,
  rate,
  format = defaultValueFormat,
  rateFormat = defaultRateFormat,
  className = '',
  showRate = true,
  variant = 'default'
}) => {
  const formattedValue = format(value);

  return (
    <StatItem
      title={title}
      value={
        !showRate || rate === undefined ? (
          formattedValue
        ) : (
          <div>
            <div>{formattedValue} ({rateFormat(rate)})</div>
          </div>
        )
      }
      className={className}
      variant={variant}
    />
  );
};
