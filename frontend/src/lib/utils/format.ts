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

export function parseNumberString(numStr: string | null | undefined): number | null {
  if (!numStr) return null;
  const cleanStr = numStr.replace(/,/g, '');
  const num = parseFloat(cleanStr);
  return isNaN(num) ? null : num;
}

export function formatDateString(date: Date, format: 'short' | 'long' = 'short'): string {
  if (!date) return '';

  const year = date.getFullYear();
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const day = date.getDate().toString().padStart(2, '0');

  if (format === 'short') {
    return `${year}/${month}/${day}`;
  }

  return `${year}年${month}月${day}日`;
}

/**
 * 数値を「¥ XXX」形式の通貨文字列にフォーマットする
 * マイナスの場合は「¥ -XXX」形式になる（マイナス記号が数字の前に）
 * @param value フォーマットする数値または数値文字列
 * @returns フォーマットされた通貨文字列
 */
export function formatCurrencyString(value: string | number): string {
  try {
    const num = typeof value === 'string' ? Number(value.replace(/,/g, '')) : value;
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
}

export function formatNumber(value: string | number): string {
  try {
    const num = typeof value === 'string' ? Number(value.replace(/,/g, '')) : value;
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
}
