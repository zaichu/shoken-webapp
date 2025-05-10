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
 * 日付をJP形式でフォーマットする関数
 */
export const formatJPDate = (value: unknown): string => {
  if (!value || !(value instanceof Date) || isNaN(value.getTime())) {
    return '-';
  }
  return value.toLocaleDateString('ja-JP', JP_DATE_FORMAT_OPTIONS);
};

/**
 * 日付から年月のグループキーを生成する関数
 * 受取金データの月次集計などに利用可能
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
  // タイムゾーンに関係なく日付部分を正しく取得
  const year = date.getFullYear();
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const day = date.getDate().toString().padStart(2, '0');
  return `${year}-${month}-${day}`;
};

/**
 * 数値を「¥ XXX」形式の通貨文字列にフォーマットする
 * マイナスの場合は「¥ -XXX」形式になる（マイナス記号が数字の前に）
 * @param value フォーマットする数値または数値文字列
 * @returns フォーマットされた通貨文字列
 */
export const formatCurrency = (value: unknown): string => {
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
    
    // マイナスの場合は絶対値を使ってフォーマットし、自分でマイナス記号を配置
    const isNegative = num < 0;
    const absNum = Math.abs(num);
    
    // 数値のみのフォーマット（マイナス記号と通貨記号なし）
    const formattedNumber = new Intl.NumberFormat('ja-JP', {
      style: 'decimal',
      useGrouping: true
    }).format(absNum);
    
    // マイナスの場合は赤文字にするためのCSSクラスをデータ属性として追加
    if (isNegative) {
      return `<span data-negative="true">¥ -${formattedNumber}</span>`;
    } else {
      return `¥ ${formattedNumber}`;
    }
  } catch (error) {
    console.error('数値のフォーマットに失敗しました:', error);
    return '-';
  }
};

/**
 * 数値をフォーマットする（通貨記号なし）
 * マイナスの場合は「-XXX」形式になる
 * @param value フォーマットする数値または数値文字列
 * @returns フォーマットされた数値文字列
 */
export const formatNumber = (value: unknown): string => {
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
    
    // マイナスの場合は絶対値を使ってフォーマットし、自分でマイナス記号を配置
    const isNegative = num < 0;
    const absNum = Math.abs(num);
    
    const formattedNumber = new Intl.NumberFormat('ja-JP', {
      style: 'decimal',
      useGrouping: true
    }).format(absNum);
    
    // マイナスの場合は赤文字にするためのCSSクラスをデータ属性として追加
    if (isNegative) {
      return `<span data-negative="true">-${formattedNumber}</span>`;
    } else {
      return formattedNumber;
    }
  } catch (error) {
    console.error('数値のフォーマットに失敗しました:', error);
    return '-';
  }
};
