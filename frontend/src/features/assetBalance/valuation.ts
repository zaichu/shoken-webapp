/**
 * 資産管理の評価損益計算。
 *
 * Issue #851 改訂1・改訂2 の確定事項に従う:
 * - 損益率には DB値 (`profit_loss_rate`) を使わず、金額から再計算する
 * - 損益額 = 評価額 - 取得総額、損益率 = 損益額 / 取得総額 * 100
 * - 取得総額が 0 のとき率は算出不可 (null)
 * - 欠損 (null / undefined / NaN / Infinity / 空文字 / '-') は
 *   0円として計算に含めず、金額・率ともに null を返す
 * - 欠損を含む集計は incomplete: true とし、金額・率は表示しない (null)
 */

export type ValuationInput = unknown;

export interface ValuationResult {
  amount: number | null;
  rate: number | null;
}

export interface ValuationSummary extends ValuationResult {
  marketValue: number | null;
  incomplete: boolean;
}

/** 欠損マーカーとして扱う文字列 (前後空白除去後に比較) */
const MISSING_MARKERS = new Set(['', '-', '—', 'ー', '--', 'n/a', 'null', 'undefined']);

/**
 * 任意の値を有限数に正規化する。欠損は null を返す。
 * 数値文字列 (カンマ区切り可) は数値として扱う。
 */
export function toFiniteAmount(value: ValuationInput): number | null {
  if (value === null || value === undefined) return null;
  if (typeof value === 'number') {
    return Number.isFinite(value) ? value : null;
  }
  if (typeof value === 'string') {
    const trimmed = value.replace(/,/g, '').trim();
    if (MISSING_MARKERS.has(trimmed.toLowerCase())) return null;
    if (trimmed === '') return null;
    const num = Number(trimmed);
    return Number.isFinite(num) ? num : null;
  }
  return null;
}

/**
 * 1銘柄の評価損益を計算する。DBの損益率は使わない。
 * いずれかが欠損なら { amount: null, rate: null } を返す。
 */
export function calculateValuation(
  marketValue: ValuationInput,
  purchaseAmount: ValuationInput,
): ValuationResult {
  const market = toFiniteAmount(marketValue);
  const purchase = toFiniteAmount(purchaseAmount);
  if (market === null || purchase === null) {
    return { amount: null, rate: null };
  }
  const amount = market - purchase;
  if (purchase === 0) {
    return { amount, rate: null };
  }
  return { amount, rate: (amount / purchase) * 100 };
}

export interface ValuationItem {
  market_value: ValuationInput;
  total_purchase_amount: ValuationInput;
}

/**
 * 複数銘柄の合計を集計する。率は合計金額から計算し、単純平均しない。
 * 1件でも欠損があれば incomplete: true で金額・率は null。
 */
export function summarizeValuation(items: ValuationItem[]): ValuationSummary {
  let totalMarket = 0;
  let totalPurchase = 0;
  for (const item of items) {
    const market = toFiniteAmount(item.market_value);
    const purchase = toFiniteAmount(item.total_purchase_amount);
    if (market === null || purchase === null) {
      return { marketValue: null, amount: null, rate: null, incomplete: true };
    }
    totalMarket += market;
    totalPurchase += purchase;
  }
  const amount = totalMarket - totalPurchase;
  if (totalPurchase === 0) {
    return { marketValue: totalMarket, amount, rate: null, incomplete: false };
  }
  return {
    marketValue: totalMarket,
    amount,
    rate: (amount / totalPurchase) * 100,
    incomplete: false,
  };
}

/** 欠損表示 */
export const VALUATION_MISSING_LABEL = '—';

/**
 * 評価金額の表示文字列。非負には + を付けて色以外の手がかりにする。
 * 欠損は「—」。
 */
export function formatValuationAmount(
  amount: number | null,
  format: (value: number) => string,
): string {
  if (amount === null || Number.isNaN(amount)) return VALUATION_MISSING_LABEL;
  if (amount >= 0) return `+${format(amount)}`;
  return format(amount);
}

/**
 * 評価損益率の表示文字列。非負には + を付ける。欠損は「—」。
 */
export function formatValuationRate(rate: number | null, decimals = 1): string {
  if (rate === null || Number.isNaN(rate)) return VALUATION_MISSING_LABEL;
  const sign = rate >= 0 ? '+' : '';
  return `${sign}${rate.toFixed(decimals)}%`;
}
