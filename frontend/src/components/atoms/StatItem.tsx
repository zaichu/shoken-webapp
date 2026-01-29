import React from 'react';

export interface StatItemProps {
  title: string;
  value: string | React.ReactNode;
  className?: string;
  titleClassName?: string;
  valueClassName?: string;
  variant?: 'default' | 'card' | 'inline';
}

/**
 * 統計情報アイテムを表示するコンポーネント
 * タイトルと値をセットで表示
 */
export const StatItem: React.FC<StatItemProps> = ({
  title,
  value,
  className = '',
  titleClassName = '',
  valueClassName = '',
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
    <div className={`${variantClasses.container} ${className}`.trim()}>
      <div className={`${variantClasses.title} ${titleClassName}`.trim()}>
        {title}
      </div>
      <div className={`${variantClasses.value} ${valueClassName}`.trim()}>
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
  format = (v) => v.toLocaleString('ja-JP'),
  rateFormat = (r) => `${r.toFixed(2)}%`,
  className = '',
  showRate = true,
  variant = 'default'
}) => {
  const formattedValue = format(value);

  const renderValue = () => {
    if (!showRate || rate === undefined) {
      return formattedValue;
    }

    return (
      <div>
        <div>{formattedValue} ({rateFormat(rate)})</div>
      </div>
    );
  };

  return (
    <StatItem
      title={title}
      value={renderValue()}
      className={className}
      variant={variant}
    />
  );
};
