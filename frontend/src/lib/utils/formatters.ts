/**
 * 統合されたフォーマットライブラリ
 * 日付、数値、通貨の表示を統一的に処理
 */

/**
 * 日本の日付フォーマット用のオプション
 */
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

/**
 * 銘柄コードのリンクHTML生成
 * 銘柄検索ページへ遷移するリンクを生成
 */
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

// ==================== 日付関連 ====================

/**
 * 日付を日本形式でフォーマットする
 * @param value 日付値
 * @returns フォーマットされた日付文字列
 */
export function formatJPDate(value: unknown): string {
  if (!value || !(value instanceof Date) || isNaN(value.getTime())) {
    return '-';
  }
  return value.toLocaleDateString('ja-JP', JP_DATE_FORMAT_OPTIONS);
}

/**
 * 日付から年月のグループキーを生成する
 * @param date 日付オブジェクト
 * @returns 年月キー (YYYY-MM形式)
 */
export function createYearMonthKey(date: Date): string {
  if (!date || !(date instanceof Date) || isNaN(date.getTime())) {
    return '';
  }
  return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}`;
}

/**
 * 日付からISO日付文字列のグループキーを生成する
 * @param date 日付オブジェクト
 * @returns ISO日付キー (YYYY-MM-DD形式)
 */
export function createISODateKey(date: Date): string {
  if (!date || !(date instanceof Date) || isNaN(date.getTime())) {
    return '';
  }
  const year = date.getFullYear();
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const day = date.getDate().toString().padStart(2, '0');
  return `${year}-${month}-${day}`;
}

// ==================== 数値関連 ====================

/**
 * 任意の値を数値に変換する（カンマ区切り対応）
 * @param value 変換する値
 * @returns 数値
 */
export function parseNumber(value: unknown): number {
  return Number(String(value || '0').replace(/,/g, ''));
}

/**
 * 値を数値に変換する（文字列・数値以外はnull）
 * @param value 変換する値
 * @returns 数値、変換できない場合はnull
 */
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

/**
 * 数値の絶対値をIntl.NumberFormatで文字列化し、符号を付与する
 * @param num 数値
 * @param formatOptions Intl.NumberFormatOptions
 * @returns 符号付きのフォーマット済み文字列
 */
function formatSignedAbsNumber(
  num: number,
  formatOptions: Intl.NumberFormatOptions
): { formatted: string; isNegative: boolean } {
  const isNegative = num < 0;
  const absNum = Math.abs(num);
  const formatted = new Intl.NumberFormat('ja-JP', formatOptions).format(absNum);
  return { formatted, isNegative };
}

/**
 * 数値を日本語形式でフォーマットする（通貨記号なし）
 * @param value フォーマットする数値
 * @param options フォーマットオプション
 * @returns フォーマットされた数値文字列
 */
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

/**
 * 数値を通貨形式でフォーマットする
 * @param value フォーマットする数値
 * @param options フォーマットオプション
 * @returns フォーマットされた通貨文字列
 */
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

/**
 * パーセンテージを計算してフォーマットする
 * @param value 値
 * @param total 合計
 * @param decimals 小数点以下の桁数
 * @returns パーセンテージ文字列
 */
export function formatPercentage(value: number, total: number, decimals = 2): string {
  if (total === 0) return '0%';
  const percentage = (value / total) * 100;
  return `${percentage.toFixed(decimals)}%`;
}

/**
 * パーセンテージ値をフォーマットする
 * @param value パーセンテージ値（0-100）
 * @param decimals 小数点以下の桁数
 * @returns フォーマットされたパーセンテージ文字列
 */
export function formatPercentageValue(value: number, decimals = 2): string {
  if (isNaN(value)) return '-';
  return `${value.toFixed(decimals)}%`;
}

// ==================== 数値計算関連 ====================

/**
 * 安全な数値加算（小数点誤差を防ぐ）
 */
export function safeAdd(a: number, b: number): number {
  return Number((a + b).toFixed(10));
}

/**
 * 安全な数値減算（小数点誤差を防ぐ）
 */
export function safeSubtract(a: number, b: number): number {
  return Number((a - b).toFixed(10));
}

/**
 * 安全な数値乗算（小数点誤差を防ぐ）
 */
export function safeMultiply(a: number, b: number): number {
  return Number((a * b).toFixed(10));
}

/**
 * 安全な数値除算（小数点誤差を防ぐ）
 */
export function safeDivide(a: number, b: number): number {
  if (b === 0) return 0;
  return Number((a / b).toFixed(10));
}

/**
 * パーセンテージを計算する
 * @param value 値
 * @param total 合計
 * @param decimals 小数点以下の桁数
 * @returns パーセンテージ
 */
export function calculatePercentage(value: number, total: number, decimals = 2): number {
  if (total === 0) return 0;
  return Number(((value / total) * 100).toFixed(decimals));
}
