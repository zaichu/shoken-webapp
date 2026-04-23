import { describe, it, expect, vi } from 'vitest';
import {
  processHeaders,
  validateCSVData,
  validateFileSize,
  validateParseErrors,
  validateParseResult
} from '../validation';

describe('validation utilities', () => {
  describe('processHeaders', () => {
    it('ヘッダーを正しく処理する', () => {
      const headers = [' Name ', '"Age"', "'Email'", '\uFEFFCode'];
      const processed = processHeaders(headers);

      expect(processed).toEqual(['Name', 'Age', 'Email', 'Code']);
    });

    it('空配列を処理する', () => {
      expect(processHeaders([])).toEqual([]);
    });
  });

  describe('validateCSVData', () => {
    it('有効なデータは検証を通過する', () => {
      const data = [
        { name: 'John', age: 30 },
        { name: 'Jane', age: 25 }
      ];
      const requiredFields = ['name', 'age'];

      const result = validateCSVData(data, requiredFields);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('空のデータは無効とする', () => {
      const result = validateCSVData([], ['name']);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('データが空です');
    });

    it('必須フィールドが不足している場合はエラーとする', () => {
      const data = [{ name: 'John' }];
      const requiredFields = ['name', 'age'];

      const result = validateCSVData(data, requiredFields);

      expect(result.valid).toBe(false);
      expect(result.errors.some(error => error.includes('必須フィールドが不足'))).toBe(true);
    });

    it('空の値がある場合はエラーとする', () => {
      const data = [
        { name: 'John', age: 30 },
        { name: '', age: 25 }
      ];
      const requiredFields = ['name', 'age'];

      const result = validateCSVData(data, requiredFields);

      expect(result.valid).toBe(false);
      expect(result.errors.some(error => error.includes('行 2'))).toBe(true);
    });

    it('null値がある場合はエラーとする', () => {
      const data = [
        { name: 'John', age: null }
      ];
      const requiredFields = ['name', 'age'];

      const result = validateCSVData(data, requiredFields);

      expect(result.valid).toBe(false);
      expect(result.errors.some(error => error.includes('"age" が空です'))).toBe(true);
    });
  });

  describe('validateFileSize', () => {
    it('適切なサイズのファイルは通過する', () => {
      const file = new File(['test content'], 'test.csv', { type: 'text/csv' });

      expect(() => validateFileSize(file)).not.toThrow();
    });

    it('大きすぎるファイルはエラーとする', () => {
      // 10MBのファイルを5MB制限で検証
      const largeContent = new Uint8Array(10 * 1024 * 1024);
      const file = new File([largeContent], 'test.csv', { type: 'text/csv' });

      expect(() => validateFileSize(file, 5)).toThrow('ファイルサイズが大きすぎます');
    });

    it('カスタム最大サイズが適用される', () => {
      const content = new Uint8Array(2 * 1024 * 1024); // 2MB
      const file = new File([content], 'test.csv', { type: 'text/csv' });

      expect(() => validateFileSize(file, 1)).toThrow(); // 1MB制限
      expect(() => validateFileSize(file, 3)).not.toThrow(); // 3MB制限
    });
  });

  describe('validateParseErrors', () => {
    it('エラーがない場合は何もしない', () => {
      expect(() => validateParseErrors([])).not.toThrow();
    });

    it('軽微なエラーは警告のみ', () => {
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      const errors = [{ type: 'Delimiter', message: 'Minor error' }];

      expect(() => validateParseErrors(errors)).not.toThrow();
      expect(consoleSpy).toHaveBeenCalled();

      consoleSpy.mockRestore();
    });

    it('重要なエラーは例外を投げる', () => {
      const errors = [
        { type: 'Quotes', message: 'Quote error', row: 1 },
        { type: 'FieldMismatch', message: 'Field mismatch', row: 2 }
      ];

      expect(() => validateParseErrors(errors)).toThrow('CSV解析エラー');
    });
  });

  describe('validateParseResult', () => {
    it('有効なデータとフィールドは通過する', () => {
      const data = [{ name: 'John' }];
      const fields = ['name'];

      expect(() => validateParseResult(data, fields)).not.toThrow();
    });

    it('空のデータは例外を投げる', () => {
      expect(() => validateParseResult([], ['name'])).toThrow('CSVデータが見つかりませんでした');
    });

    it('空のフィールドは例外を投げる', () => {
      const data = [{ name: 'John' }];

      expect(() => validateParseResult(data, [])).toThrow('CSVヘッダーが見つかりませんでした');
      expect(() => validateParseResult(data, undefined)).toThrow('CSVヘッダーが見つかりませんでした');
    });
  });
});
