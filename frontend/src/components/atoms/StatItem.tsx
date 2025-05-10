import React from 'react';

interface StatItemProps {
  title: string;
  value: string | React.ReactNode;
  className?: string;
}

/**
 * 統計情報アイテムを表示するコンポーネント
 * タイトルと値をセットで表示
 */
export const StatItem: React.FC<StatItemProps> = ({
  title,
  value,
  className = "col"
}) => {
  return (
    <div className={className}>
      <h6 className='mb-0'>{title}</h6>
      <h4 className='mb-0'>{value}</h4>
    </div>
  );
};

interface StatItemWithRateProps {
  title: string;
  value: number;
  rate?: number;
  format?: (value: number) => string;
  rateFormat?: (rate: number) => string;
  className?: string;
}

/**
 * 統計情報アイテムを表示するコンポーネント（レート表示付き）
 * 金額などの値とレート（パーセンテージ）を併せて表示
 */
export const StatItemWithRate: React.FC<StatItemWithRateProps> = ({
  title,
  value,
  rate,
  format = (v) => v.toString(),
  rateFormat = (r) => `${r.toFixed(2)}%`,
  className = "col"
}) => {
  const formattedValue = format(value);
  const displayValue = rate !== undefined
    ? `${formattedValue} (${rateFormat(rate)})`
    : formattedValue;

  return (
    <StatItem
      title={title}
      value={displayValue}
      className={className}
    />
  );
};
