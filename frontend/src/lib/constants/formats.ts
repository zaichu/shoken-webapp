/**
 * 日本の日付フォーマット用のオプション
 */
export const JP_DATE_FORMAT_OPTIONS = { 
  year: 'numeric', 
  month: '2-digit', 
  day: '2-digit' 
} as const;

/**
 * 税率定数
 */
export const TAX_RATE = 0.20315;

/**
 * 日付をJP形式でフォーマットする関数
 */
export const formatJPDate = (date: Date): string => {
  if (!date || !(date instanceof Date) || isNaN(date.getTime())) {
    return '-';
  }
  return date.toLocaleDateString('ja-JP', JP_DATE_FORMAT_OPTIONS);
};

/**
 * 日付から年月のグループキーを生成する関数
 */
export const createYearMonthKey = (date: Date): string => {
  if (!date || !(date instanceof Date) || isNaN(date.getTime())) {
    return '';
  }
  return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}`;
};

/**
 * 日付からISO日付文字列のグループキーを生成する関数
 */
export const createISODateKey = (date: Date): string => {
  if (!date || !(date instanceof Date) || isNaN(date.getTime())) {
    return '';
  }
  return date.toISOString().split('T')[0];
};
