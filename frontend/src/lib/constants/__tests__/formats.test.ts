import { 
  formatJPDate, 
  createYearMonthKey, 
  createISODateKey, 
  formatCurrency, 
  formatNumber, 
  TAX_RATE 
} from '../formats';

describe('日付フォーマット関数', () => {
  test('正常な日付に対してJP形式でフォーマットする', () => {
    const date = new Date(2023, 11, 25); // 2023年12月25日 (JavaScriptの月は0から始まる)
    const result = formatJPDate(date);
    expect(result).toBe('2023/12/25');
  });

  test('無効な日付に対してハイフンを返す', () => {
    // @ts-expect-error: テスト目的で無効な値を渡す
    const result = formatJPDate(null);
    expect(result).toBe('-');

    const invalidDate = new Date('invalid date');
    const result2 = formatJPDate(invalidDate);
    expect(result2).toBe('-');
  });
});

describe('日付キー生成関数', () => {
  test('正常な日付から年月キーを生成する', () => {
    const date = new Date(2023, 11, 25); // 2023年12月25日
    const result = createYearMonthKey(date);
    expect(result).toBe('2023-12');
  });

  test('単数月の場合もゼロ詰めの形式で生成する', () => {
    const date = new Date(2023, 0, 15); // 2023年1月15日
    const result = createYearMonthKey(date);
    expect(result).toBe('2023-01');
  });

  test('無効な日付に対して空文字を返す', () => {
    // @ts-expect-error: テスト目的で無効な値を渡す
    const result = createYearMonthKey(null);
    expect(result).toBe('');

    const invalidDate = new Date('invalid date');
    const result2 = createYearMonthKey(invalidDate);
    expect(result2).toBe('');
  });

  test('正常な日付からISO日付キーを生成する', () => {
    const date = new Date(2023, 11, 25); // 2023年12月25日
    const result = createISODateKey(date);
    expect(result).toBe('2023-12-25');
  });

  test('無効な日付に対して空文字を返す', () => {
    // @ts-expect-error: テスト目的で無効な値を渡す
    const result = createISODateKey(null);
    expect(result).toBe('');

    const invalidDate = new Date('invalid date');
    const result2 = createISODateKey(invalidDate);
    expect(result2).toBe('');
  });
});

describe('金額フォーマット関数', () => {
  test('正の金額を通貨形式でフォーマットする', () => {
    const result = formatCurrency(12345);
    expect(result).toBe('¥ 12,345');
  });

  test('負の金額を通貨形式でフォーマットし、マイナス記号を追加する', () => {
    const result = formatCurrency(-12345);
    expect(result).toBe('<span data-negative="true">¥ -12,345</span>');
  });

  test('文字列として渡された金額も正しくフォーマットする', () => {
    const result = formatCurrency('12345');
    expect(result).toBe('¥ 12,345');
  });

  test('カンマ付きの文字列も正しくフォーマットする', () => {
    const result = formatCurrency('12,345');
    expect(result).toBe('¥ 12,345');
  });

  test('非数値文字列に対してはハイフンを返す', () => {
    const result = formatCurrency('abc');
    expect(result).toBe('-');
  });
});

describe('数値フォーマット関数', () => {
  test('正の数値をフォーマットする', () => {
    const result = formatNumber(12345);
    expect(result).toBe('12,345');
  });

  test('負の数値をフォーマットし、マイナス記号を追加する', () => {
    const result = formatNumber(-12345);
    expect(result).toBe('<span data-negative="true">-12,345</span>');
  });

  test('文字列として渡された数値も正しくフォーマットする', () => {
    const result = formatNumber('12345');
    expect(result).toBe('12,345');
  });

  test('カンマ付きの文字列も正しくフォーマットする', () => {
    const result = formatNumber('12,345');
    expect(result).toBe('12,345');
  });

  test('非数値文字列に対してはハイフンを返す', () => {
    const result = formatNumber('abc');
    expect(result).toBe('-');
  });
});

describe('税率定数', () => {
  test('税率は正しい値である', () => {
    // 配当金や分配金の源泉徴収税率は20.315%（所得税15%、復興特別所得税0.315%、住民税5%）
    expect(TAX_RATE).toBe(0.20315);
  });

  test('税率を使った計算が正しい', () => {
    const amount = 100000;
    const expectedTax = amount * TAX_RATE;
    expect(expectedTax).toBe(20315);
  });
});
