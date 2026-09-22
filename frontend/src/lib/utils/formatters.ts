const JP_DATE_FORMAT_OPTIONS = {
  year: 'numeric',
  month: '2-digit',
  day: '2-digit'
} as const;

/**
 * 税率定数
 * 配当金や分配金の源泉徴収税率（20.315%）
 */
export const TAX_RATE = 0.20315;

export const SECURITY_CODE_REGEX = /^[0-9A-Za-z.]+$/;

/**
 * 銘柄コードの正規化
 * - 前後の空白を除去
 * - ラベル形式("1234: 銘柄名")はコード部分のみ抽出
 * - 比較用に大文字化
 */
export function normalizeSecurityCode(value: unknown): string {
  if (value === null || value === undefined) return '';
  const raw = String(value).trim();
  if (!raw) return '';
  const token = raw.split(/[:：]/)[0];
  return token.replace(/\s+/g, '').toUpperCase();
}

/**
 * 銘柄名の表示を正規化する
 * - 全角英数字を半角に変換する
 */
export function normalizeSecurityName(name: string): string {
  return name.replace(/[Ａ-Ｚａ-ｚ０-９]/g, (char) =>
    String.fromCharCode(char.charCodeAt(0) - 0xFEE0)
  );
}

export function formatJPDate(value: unknown): string {
  if (!value || !(value instanceof Date) || isNaN(value.getTime())) {
    return '-';
  }
  return value.toLocaleDateString('ja-JP', JP_DATE_FORMAT_OPTIONS);
}

export function createYearMonthKey(date: Date): string {
  if (!date || !(date instanceof Date) || isNaN(date.getTime())) {
    return '';
  }
  return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}`;
}

export function createISODateKey(date: Date): string {
  if (!date || !(date instanceof Date) || isNaN(date.getTime())) {
    return '';
  }
  const year = date.getFullYear();
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const day = date.getDate().toString().padStart(2, '0');
  return `${year}-${month}-${day}`;
}

/** 任意の値を数値に変換する（カンマ区切り対応） */
export function parseNumber(value: unknown): number {
  return Number(String(value || '0').replace(/,/g, ''));
}

/** 値を数値に変換する（文字列・数値以外はnull） */
function toNumberOrNull(value: unknown): number | null {
  let num: number;

  if (typeof value === 'string') {
    num = Number(value.replace(/,/g, ''));
  } else if (typeof value === 'number') {
    num = value;
  } else {
    return null;
  }

  return isNaN(num) ? null : num;
}

/** 数値の絶対値をIntl.NumberFormatで文字列化し、符号を付与する */
function formatSignedAbsNumber(
  num: number,
  formatOptions: Intl.NumberFormatOptions
): { formatted: string; isNegative: boolean } {
  const isNegative = num < 0;
  const absNum = Math.abs(num);
  const formatted = new Intl.NumberFormat('ja-JP', formatOptions).format(absNum);
  return { formatted, isNegative };
}

/** 数値を日本語形式でフォーマットする（通貨記号なし） */
export function formatNumber(
  value: unknown,
  options: {
    minimumFractionDigits?: number;
    maximumFractionDigits?: number;
    useGrouping?: boolean;
  } = {}
): string {
  const {
    minimumFractionDigits = 0,
    maximumFractionDigits = 2,
    useGrouping = true
  } = options;

  try {
    const num = toNumberOrNull(value);
    if (num === null) {
      return '-';
    }

    const { formatted, isNegative } = formatSignedAbsNumber(num, {
      style: 'decimal',
      useGrouping,
      minimumFractionDigits,
      maximumFractionDigits
    });

    return isNegative ? `-${formatted}` : formatted;
  } catch (error) {
    console.error('数値のフォーマットに失敗しました:', error instanceof Error ? error.message : String(error));
    return '-';
  }
}

export function formatCurrency(
  value: unknown,
  options: {
    currency?: string;
    minimumFractionDigits?: number;
    maximumFractionDigits?: number;
  } = {}
): string {
  const {
    currency = '¥',
    minimumFractionDigits = 0,
    maximumFractionDigits = 15
  } = options;

  try {
    const num = toNumberOrNull(value);
    if (num === null) {
      return '-';
    }

    const { formatted, isNegative } = formatSignedAbsNumber(num, {
      style: 'decimal',
      useGrouping: true,
      minimumFractionDigits,
      maximumFractionDigits
    });

    return isNegative ? `${currency} -${formatted}` : `${currency} ${formatted}`;
  } catch (error) {
    console.error('通貨のフォーマットに失敗しました:', error instanceof Error ? error.message : String(error));
    return '-';
  }
}

export function formatPercentage(value: number, total: number, decimals = 2): string {
  if (total === 0) return '0%';
  const percentage = (value / total) * 100;
  return `${percentage.toFixed(decimals)}%`;
}

export function formatPercentageValue(value: number, decimals = 2): string {
  if (isNaN(value)) return '-';
  return `${value.toFixed(decimals)}%`;
}

export function safeAdd(a: number, b: number): number {
  return Number((a + b).toFixed(10));
}

export function safeSubtract(a: number, b: number): number {
  return Number((a - b).toFixed(10));
}

export function safeMultiply(a: number, b: number): number {
  return Number((a * b).toFixed(10));
}

export function safeDivide(a: number, b: number): number {
  if (b === 0) return 0;
  return Number((a / b).toFixed(10));
}

export function calculatePercentage(value: number, total: number, decimals = 2): number {
  if (total === 0) return 0;
  return Number(((value / total) * 100).toFixed(decimals));
}
