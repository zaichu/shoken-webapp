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

export function formatCurrencyString(amount: number | null | undefined, currency: string = '¥'): string {
  if (amount === null || amount === undefined) return '';
  return `${currency} ${amount.toLocaleString()}`;
}
