/**
 * 数値関連のユーティリティ関数
 */

/**
 * 文字列をパースして数値に変換する
 * カンマ区切りも対応
 * @param value パースする値
 * @returns 数値
 */
export const parseNumber = (value: unknown): number => {
  return Number(String(value || '0').replace(/,/g, ''));
};

/**
 * 数値を安全に計算する（小数点の誤差を防ぐ）
 */
export const safeAdd = (a: number, b: number): number => {
  return Number((a + b).toFixed(10));
};

export const safeSubtract = (a: number, b: number): number => {
  return Number((a - b).toFixed(10));
};

export const safeMultiply = (a: number, b: number): number => {
  return Number((a * b).toFixed(10));
};

export const safeDivide = (a: number, b: number): number => {
  if (b === 0) return 0;
  return Number((a / b).toFixed(10));
};

/**
 * パーセンテージを計算する
 * @param value 値
 * @param total 合計
 * @param decimals 小数点以下の桁数
 * @returns パーセンテージ
 */
export const calculatePercentage = (value: number, total: number, decimals = 2): number => {
  if (total === 0) return 0;
  return Number(((value / total) * 100).toFixed(decimals));
};
