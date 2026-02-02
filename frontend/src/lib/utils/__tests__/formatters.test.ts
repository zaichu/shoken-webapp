import {
  parseDate,
  formatJPDate,
  formatDateString,
  createYearMonthKey,
  createISODateKey,
  parseNumberString,
  parseNumber,
  formatNumber,
  formatCurrency,
  formatPercentage,
  formatPercentageValue,
  safeAdd,
  safeSubtract,
  safeMultiply,
  safeDivide,
  calculatePercentage,
  TAX_RATE,
  normalizeSecurityCode
} from '../formatters';

describe('日付関連のフォーマット関数', () => {
  describe('parseDate', () => {
    it('正しい日付文字列を解析する', () => {
      const result = parseDate('2023/12/25');
      expect(result).toBeInstanceOf(Date);
      expect(result?.getFullYear()).toBe(2023);
      expect(result?.getMonth()).toBe(11); // 0ベース
      expect(result?.getDate()).toBe(25);
    });

    it('無効な文字列でnullを返す', () => {
      expect(parseDate('')).toBeNull();
      expect(parseDate('invalid')).toBeNull();
      expect(parseDate('2023-12-25')).toBeNull(); // 違う形式
    });

    it('単桁の月日も正しく解析する', () => {
      const result = parseDate('2023/1/5');
      expect(result?.getMonth()).toBe(0);
      expect(result?.getDate()).toBe(5);
    });
  });

  describe('formatJPDate', () => {
    it('正常な日付を日本形式でフォーマットする', () => {
      const date = new Date(2023, 11, 25);
      expect(formatJPDate(date)).toBe('2023/12/25');
    });

    it('無効な値でハイフンを返す', () => {
      expect(formatJPDate(null)).toBe('-');
      expect(formatJPDate(undefined)).toBe('-');
      expect(formatJPDate('string')).toBe('-');
      expect(formatJPDate(new Date('invalid'))).toBe('-');
    });
  });

  describe('formatDateString', () => {
    const date = new Date(2023, 11, 25);

    it('短い形式でフォーマットする', () => {
      expect(formatDateString(date, 'short')).toBe('2023/12/25');
      expect(formatDateString(date)).toBe('2023/12/25'); // デフォルト
    });

    it('長い形式でフォーマットする', () => {
      expect(formatDateString(date, 'long')).toBe('2023年12月25日');
    });

    it('無効な日付で空文字を返す', () => {
      expect(formatDateString(new Date('invalid'))).toBe('');
    });
  });

  describe('createYearMonthKey', () => {
    it('年月キーを生成する', () => {
      const date = new Date(2023, 11, 25);
      expect(createYearMonthKey(date)).toBe('2023-12');
    });

    it('単桁月もゼロ埋めする', () => {
      const date = new Date(2023, 0, 15);
      expect(createYearMonthKey(date)).toBe('2023-01');
    });

    it('無効な日付で空文字を返す', () => {
      expect(createYearMonthKey(new Date('invalid'))).toBe('');
    });
  });

  describe('createISODateKey', () => {
    it('ISO日付キーを生成する', () => {
      const date = new Date(2023, 11, 25);
      expect(createISODateKey(date)).toBe('2023-12-25');
    });

    it('単桁日もゼロ埋めする', () => {
      const date = new Date(2023, 0, 5);
      expect(createISODateKey(date)).toBe('2023-01-05');
    });
  });
});

describe('数値関連のフォーマット関数', () => {
  describe('normalizeSecurityCode', () => {
    it('銘柄コードの空白を除去して正規化する', () => {
      expect(normalizeSecurityCode(' 7974 ')).toBe('7974');
    });

    it('ラベル形式から銘柄コードを抽出する', () => {
      expect(normalizeSecurityCode('7974: 任天堂')).toBe('7974');
    });

    it('英字コードを大文字化する', () => {
      expect(normalizeSecurityCode('brk.b')).toBe('BRK.B');
    });
  });
  describe('parseNumberString', () => {
    it('数値文字列を解析する', () => {
      expect(parseNumberString('123')).toBe(123);
      expect(parseNumberString('123.45')).toBe(123.45);
      expect(parseNumberString('1,234')).toBe(1234);
      expect(parseNumberString('1,234.56')).toBe(1234.56);
    });

    it('無効な値でnullを返す', () => {
      expect(parseNumberString(null)).toBeNull();
      expect(parseNumberString(undefined)).toBeNull();
      expect(parseNumberString('')).toBeNull();
      expect(parseNumberString('abc')).toBeNull();
    });
  });

  describe('parseNumber', () => {
    it('様々な値を数値に変換する', () => {
      expect(parseNumber('123')).toBe(123);
      expect(parseNumber('1,234')).toBe(1234);
      expect(parseNumber(456)).toBe(456);
      expect(parseNumber(null)).toBe(0);
      expect(parseNumber(undefined)).toBe(0);
    });
  });

  describe('formatNumber', () => {
    it('正の数値をフォーマットする', () => {
      expect(formatNumber(12345)).toBe('12,345');
      expect(formatNumber('12345')).toBe('12,345');
    });

    it('負の数値をフォーマットする', () => {
      expect(formatNumber(-12345)).toBe('-12,345');
    });

    it('小数点オプションが動作する', () => {
      expect(formatNumber(123.456, { maximumFractionDigits: 2 })).toBe('123.46');
      expect(formatNumber(123, { minimumFractionDigits: 2 })).toBe('123.00');
    });

    it('グループ化オプションが動作する', () => {
      expect(formatNumber(12345, { useGrouping: false })).toBe('12345');
    });

    it('無効な値でハイフンを返す', () => {
      expect(formatNumber('abc')).toBe('-');
      expect(formatNumber(null)).toBe('-');
    });
  });

  describe('formatCurrency', () => {
    it('正の金額を通貨形式でフォーマットする', () => {
      expect(formatCurrency(12345)).toBe('¥ 12,345');
    });

    it('負の金額をフォーマットする', () => {
      expect(formatCurrency(-12345)).toBe('¥ -12,345');
    });

    it('カスタム通貨記号が動作する', () => {
      expect(formatCurrency(12345, { currency: '$' })).toBe('$ 12,345');
    });

    it('小数点オプションが動作する', () => {
      expect(formatCurrency(123.45, { maximumFractionDigits: 2 })).toBe('¥ 123.45');
    });

    it('無効な値でハイフンを返す', () => {
      expect(formatCurrency('abc')).toBe('-');
    });
  });

  describe('formatPercentage', () => {
    it('パーセンテージを計算してフォーマットする', () => {
      expect(formatPercentage(25, 100)).toBe('25.00%');
      expect(formatPercentage(1, 3, 1)).toBe('33.3%');
    });

    it('合計が0の場合は0%を返す', () => {
      expect(formatPercentage(10, 0)).toBe('0%');
    });
  });

  describe('formatPercentageValue', () => {
    it('パーセンテージ値をフォーマットする', () => {
      expect(formatPercentageValue(25.5)).toBe('25.50%');
      expect(formatPercentageValue(25.5, 1)).toBe('25.5%');
    });

    it('無効な値でハイフンを返す', () => {
      expect(formatPercentageValue(NaN)).toBe('-');
    });
  });
});

describe('数値計算関数', () => {
  describe('safeAdd', () => {
    it('正確な加算を行う', () => {
      expect(safeAdd(0.1, 0.2)).toBe(0.3);
      expect(safeAdd(10, 20)).toBe(30);
    });
  });

  describe('safeSubtract', () => {
    it('正確な減算を行う', () => {
      expect(safeSubtract(0.3, 0.1)).toBe(0.2);
      expect(safeSubtract(20, 10)).toBe(10);
    });
  });

  describe('safeMultiply', () => {
    it('正確な乗算を行う', () => {
      expect(safeMultiply(0.1, 3)).toBe(0.3);
      expect(safeMultiply(10, 20)).toBe(200);
    });
  });

  describe('safeDivide', () => {
    it('正確な除算を行う', () => {
      expect(safeDivide(0.3, 3)).toBe(0.1);
      expect(safeDivide(20, 10)).toBe(2);
    });

    it('0で割ったときは0を返す', () => {
      expect(safeDivide(10, 0)).toBe(0);
    });
  });

  describe('calculatePercentage', () => {
    it('パーセンテージを計算する', () => {
      expect(calculatePercentage(25, 100)).toBe(25);
      expect(calculatePercentage(1, 3, 1)).toBe(33.3);
    });

    it('合計が0の場合は0を返す', () => {
      expect(calculatePercentage(10, 0)).toBe(0);
    });
  });
});

describe('定数', () => {
  it('税率が正しい値を持つ', () => {
    expect(TAX_RATE).toBe(0.20315);
  });
});
