/**
 * 検索カテゴリーとフィルタリングに関する共通ユーティリティ
 */

/**
 * 年度の検索オプションを生成
 */
export function createYearOptions<T>(
  data: T[],
  dateGetter: (item: T) => Date
): { value: string; label: string }[] {
  return [...new Set(data.map(item => {
    const year = dateGetter(item).getFullYear().toString();
    const label = `${year}年`;
    return { value: year, label };
  }))].filter((item, index, self) =>
    index === self.findIndex(t => t.value === item.value)
  ).sort((a, b) => a.value.localeCompare(b.value));
}

/**
 * 年月の検索オプションを生成
 */
export function createYearMonthOptions<T>(
  data: T[],
  dateGetter: (item: T) => Date
): { value: string; label: string }[] {
  return [...new Set(data.map(item => {
    const date = dateGetter(item);
    const year = date.getFullYear();
    const month = date.getMonth() + 1;
    const value = `${year}-${month.toString().padStart(2, '0')}`;
    const label = `${year}年${month.toString().padStart(2, '0')}月`;
    return { value, label };
  }))].filter((item, index, self) =>
    index === self.findIndex(t => t.value === item.value)
  ).sort((a, b) => a.value.localeCompare(b.value));
}

/**
 * 配列から重複を除いたユニーク値を取得
 */
export function getUniqueValues<T>(
  data: T[],
  fieldGetter: (item: T) => string
): string[] {
  return [...new Set(data.map(fieldGetter))]
    .filter(value => value && value.trim() !== '');
}

/**
 * 年度検索マッチャー
 */
export function matchesYear(date: Date, query: string): boolean {
  return date.getFullYear().toString() === query;
}

/**
 * 年月検索マッチャー
 */
export function matchesYearMonth(date: Date, query: string): boolean {
  const year = date.getFullYear();
  const month = date.getMonth() + 1;
  const yearMonth = `${year}-${month.toString().padStart(2, '0')}`;
  return yearMonth === query;
}

/**
 * 日付検索マッチャー
 */
export function matchesDate(date: Date, query: string): boolean {
  const dateStr = date.toISOString().split('T')[0];
  return dateStr === query;
}

/**
 * 金額配列の部分一致検索
 */
export function matchesAmounts(amounts: number[], query: string): boolean {
  return amounts.some(amount => amount.toString().includes(query));
}
