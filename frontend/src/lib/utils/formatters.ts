/**
 * 統合されたフォーマットライブラリ
 * 日付、数値、通貨の表示を統一的に処理
 */

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
 * 配当金や分配金の源泉徴収税率（20.315%）
 */
export const TAX_RATE = 0.20315;

/**
 * HTMLエスケープ処理
 * XSS攻撃を防ぐため、特殊文字をエスケープ
 */
const escapeHtml = (str: string): string => {
  const htmlEscapes: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;',
  };
  return str.replace(/[&<>"']/g, char => htmlEscapes[char]);
};

/**
 * 銘柄コードのリンクHTML生成
 * 銘柄検索ページへ遷移するリンクを生成
 */
export function createSecurityCodeLink(value: unknown): string {
  const code = typeof value === 'string' ? value.trim() : '';
  if (!code) return '';

  // 想定外の文字列はリンク化せず表示のみ（URLパラメータの安全性確保）
  if (!/^[0-9A-Za-z]+$/.test(code)) {
    return escapeHtml(code);
  }

  const escapedCode = escapeHtml(code);
  return `<a href="/search?code=${encodeURIComponent(code)}" class="security-code-link" style="color: #0d6efd; font-weight: 600;">${escapedCode}</a>`;
}

// ==================== 日付関連 ====================

/**
 * 文字列から日付オブジェクトを解析する
 * @param dateStr 日付文字列 (YYYY/MM/DD形式)
 * @returns 日付オブジェクト、解析できない場合はnull
 */
export function parseDate(dateStr: string): Date | null {
  if (!dateStr) return null;

  const match = dateStr.match(/^(\d{4})\/(\d{1,2})\/(\d{1,2})$/);
  if (match) {
    const year = parseInt(match[1], 10);
    const month = parseInt(match[2], 10) - 1;
    const day = parseInt(match[3], 10);
    return new Date(year, month, day);
  }

  return null;
}

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
 * 日付文字列をフォーマットする
 * @param date 日付オブジェクト
 * @param format フォーマット形式
 * @returns フォーマットされた日付文字列
 */
export function formatDateString(date: Date, format: 'short' | 'long' = 'short'): string {
  if (!date || !(date instanceof Date) || isNaN(date.getTime())) {
    return '';
  }

  const year = date.getFullYear();
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const day = date.getDate().toString().padStart(2, '0');

  if (format === 'short') {
    return `${year}/${month}/${day}`;
  }

  return `${year}年${month}月${day}日`;
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
 * 数値文字列を解析して数値に変換する
 * @param value 解析する値
 * @returns 数値、解析できない場合はnull
 */
export function parseNumberString(value: string | null | undefined): number | null {
  if (!value) return null;
  const cleanStr = value.replace(/,/g, '');
  const num = parseFloat(cleanStr);
  return isNaN(num) ? null : num;
}

/**
 * 任意の値を数値に変換する（カンマ区切り対応）
 * @param value 変換する値
 * @returns 数値
 */
export function parseNumber(value: unknown): number {
  return Number(String(value || '0').replace(/,/g, ''));
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
    showNegativeSpan?: boolean;
  } = {}
): string {
  const {
    minimumFractionDigits = 0,
    maximumFractionDigits = 2,
    useGrouping = true,
    showNegativeSpan = true
  } = options;

  try {
    let num: number;

    if (typeof value === 'string') {
      num = Number(value.replace(/,/g, ''));
    } else if (typeof value === 'number') {
      num = value;
    } else {
      return '-';
    }

    if (isNaN(num)) {
      return '-';
    }

    const isNegative = num < 0;
    const absNum = Math.abs(num);

    const formattedNumber = new Intl.NumberFormat('ja-JP', {
      style: 'decimal',
      useGrouping,
      minimumFractionDigits,
      maximumFractionDigits
    }).format(absNum);

    if (isNegative && showNegativeSpan) {
      return `<span data-negative="true">-${formattedNumber}</span>`;
    } else if (isNegative) {
      return `-${formattedNumber}`;
    } else {
      return formattedNumber;
    }
  } catch (error) {
    console.error('数値のフォーマットに失敗しました:', error);
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
    showNegativeSpan?: boolean;
  } = {}
): string {
  const {
    currency = '¥',
    minimumFractionDigits = 0,
    maximumFractionDigits = 15,
    showNegativeSpan = true
  } = options;

  try {
    let num: number;

    if (typeof value === 'string') {
      num = Number(value.replace(/,/g, ''));
    } else if (typeof value === 'number') {
      num = value;
    } else {
      return '-';
    }

    if (isNaN(num)) {
      return '-';
    }

    const isNegative = num < 0;
    const absNum = Math.abs(num);

    const formattedNumber = new Intl.NumberFormat('ja-JP', {
      style: 'decimal',
      useGrouping: true,
      minimumFractionDigits,
      maximumFractionDigits
    }).format(absNum);

    if (isNegative && showNegativeSpan) {
      return `<span data-negative="true">${currency} -${formattedNumber}</span>`;
    } else if (isNegative) {
      return `${currency} -${formattedNumber}`;
    } else {
      return `${currency} ${formattedNumber}`;
    }
  } catch (error) {
    console.error('通貨のフォーマットに失敗しました:', error);
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

// ==================== 後方互換性のためのエイリアス ====================

/**
 * @deprecated formatCurrency()を使用してください
 */
export const formatCurrencyString = formatCurrency;

/**
 * @deprecated formatJPDate()を使用してください
 */
export const formatJPDateString = formatJPDate;
