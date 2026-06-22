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
  const seen = new Map<string, { value: string; label: string }>();
  for (const item of data) {
    const year = dateGetter(item).getFullYear().toString();
    if (!seen.has(year)) {
      seen.set(year, { value: year, label: `${year}年` });
    }
  }
  return [...seen.values()].sort((a, b) => a.value.localeCompare(b.value));
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
function matchesDate(date: Date, query: string): boolean {
  const dateStr = date.toISOString().split('T')[0];
  return dateStr === query;
}

/**
 * 日付範囲検索マッチャー
 * クエリ形式: "YYYY-MM-DD..YYYY-MM-DD" / "YYYY-MM-DD.." / "..YYYY-MM-DD"
 */
const ISO_DATE_RE = /^\d{4}-\d{2}-\d{2}$/;

function isValidIsoDate(value: string): boolean {
  if (!ISO_DATE_RE.test(value)) return false;
  const date = new Date(value + 'T00:00:00.000Z');
  return !Number.isNaN(date.getTime()) && date.toISOString().slice(0, 10) === value;
}

function matchesDateRange(date: Date, query: string): boolean {
  const sepIdx = query.indexOf('..');
  if (sepIdx === -1) return false;
  const start = query.slice(0, sepIdx);
  const end = query.slice(sepIdx + 2);
  if (start && !isValidIsoDate(start)) return false;
  if (end && !isValidIsoDate(end)) return false;
  if (!start && !end) return false;
  if (start && end && start > end) return false;
  const dateStr = date.toISOString().split('T')[0];
  if (start && end) return dateStr >= start && dateStr <= end;
  if (start) return dateStr >= start;
  return dateStr <= end;
}

// ==================== 汎用フィルタ設定 ====================

/**
 * フィルタ設定
 */
export interface FilterConfig<T> {
  /** 文字列完全一致検索対象フィールド */
  stringFields?: ((item: T) => string)[];
  /** 文字列部分一致検索対象フィールド */
  partialStringFields?: ((item: T) => string)[];
  /** 日付フィールド（年/年月/日付検索用） */
  dateField?: (item: T) => Date;
  /** 年度検索を有効にするか */
  yearSearch?: boolean;
  /** 年月検索を有効にするか */
  yearMonthSearch?: boolean;
  /** 日付検索を有効にするか */
  dateSearch?: boolean;
  /** 日付範囲検索を有効にするか（YYYY-MM-DD..YYYY-MM-DD 形式） */
  dateRangeSearch?: boolean;
  /** 金額フィールド（部分一致検索用） */
  amountFields?: ((item: T) => number)[];
}

/**
 * 設定ベースの汎用フィルタ関数
 * @param data フィルタ対象データ
 * @param query 検索クエリ
 * @param config フィルタ設定
 * @returns フィルタ結果
 */
export function filterByConfig<T>(
  data: T[],
  query: string,
  config: FilterConfig<T>
): T[] {
  if (!query) return data;

  const normalizedQuery = query.toLowerCase();

  return data.filter(item => {
    // 文字列フィールドの完全一致検索
    if (config.stringFields) {
      for (const getter of config.stringFields) {
        if (getter(item).toLowerCase() === normalizedQuery) {
          return true;
        }
      }
    }

    // 文字列フィールドの部分一致検索
    if (config.partialStringFields) {
      for (const getter of config.partialStringFields) {
        if (getter(item).toLowerCase().includes(normalizedQuery)) {
          return true;
        }
      }
    }

    // 日付関連の検索
    if (config.dateField) {
      const date = config.dateField(item);

      // 年度検索
      if (config.yearSearch && matchesYear(date, normalizedQuery)) {
        return true;
      }

      // 年月検索
      if (config.yearMonthSearch && matchesYearMonth(date, normalizedQuery)) {
        return true;
      }

      // 日付検索
      if (config.dateSearch && matchesDate(date, normalizedQuery)) {
        return true;
      }

      // 日付範囲検索
      if (config.dateRangeSearch && matchesDateRange(date, normalizedQuery)) {
        return true;
      }
    }

    // 金額の部分一致検索
    if (config.amountFields) {
      for (const getter of config.amountFields) {
        if (getter(item).toString().includes(normalizedQuery)) {
          return true;
        }
      }
    }

    return false;
  });
}
