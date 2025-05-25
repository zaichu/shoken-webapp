import React, { memo } from 'react';

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
export const StatItem = memo<StatItemProps>(({
  title,
  value,
  className = 'col',
  titleClassName = 'mb-0',
  valueClassName = 'mb-0',
  variant = 'default'
}) => {
  const getVariantClasses = () => {
    switch (variant) {
      case 'card':
        return {
          container: 'card p-3 stat-item-card',
          title: 'card-title h6',
          value: 'card-text h4'
        };
      case 'inline':
        return {
          container: 'd-flex justify-content-between align-items-center stat-item-inline',
          title: 'mb-0 text-muted',
          value: 'mb-0 fw-bold'
        };
      default:
        return {
          container: '',
          title: 'h6',
          value: 'h4'
        };
    }
  };

  const variantClasses = getVariantClasses();

  return (
    <div className={`${className} ${variantClasses.container}`.trim()}>
      <div className={`${variantClasses.title} ${titleClassName}`.trim()}>
        {title}
      </div>
      <div className={`${variantClasses.value} ${valueClassName}`.trim()}>
        {value}
      </div>
    </div>
  );
});

StatItem.displayName = 'StatItem';

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
export const StatItemWithRate = memo<StatItemWithRateProps>(({
  title,
  value,
  rate,
  format = (v) => v.toLocaleString('ja-JP'),
  rateFormat = (r) => `${r.toFixed(2)}%`,
  className = 'col',
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
});

StatItemWithRate.displayName = 'StatItemWithRate';
