import { describe, it, expect } from 'vitest';
import { 
  detectMojibake, 
  calculateEncodingConfidence, 
  removeBOM,
  tryDecodeWithMultipleEncodings 
} from '../encoding';

describe('encoding utilities', () => {
  describe('detectMojibake', () => {
    it('文字化けを検出する', () => {
      expect(detectMojibake('正常なテキスト')).toBe(false);
      expect(detectMojibake('文字化け�が含まれる')).toBe(true);
      expect(detectMojibake('��文字化け')).toBe(true);
      expect(detectMojibake('文字化け\uFFFD')).toBe(true);
    });
  });

  describe('calculateEncodingConfidence', () => {
    it('空文字列の場合は0を返す', () => {
      expect(calculateEncodingConfidence('')).toBe(0);
    });

    it('正常な日本語テキストは高い信頼度を返す', () => {
      const confidence = calculateEncodingConfidence('こんにちは世界');
      expect(confidence).toBeGreaterThan(0.8);
    });

    it('文字化けがある場合は信頼度が下がる', () => {
      const confidence = calculateEncodingConfidence('こんにちは�世界');
      expect(confidence).toBeLessThan(0.8);
    });

    it('制御文字がある場合は信頼度が下がる', () => {
      const confidence = calculateEncodingConfidence('hello\x00world');
      expect(confidence).toBeLessThan(1.0);
    });
  });

  describe('removeBOM', () => {
    it('BOMを削除する', () => {
      expect(removeBOM('\uFEFFhello world')).toBe('hello world');
      expect(removeBOM('hello world')).toBe('hello world');
    });
  });

  describe('tryDecodeWithMultipleEncodings', () => {
    it('UTF-8エンコーディングでデコードする', () => {
      const utf8Text = 'こんにちは世界';
      const encoder = new TextEncoder();
      const uint8Array = encoder.encode(utf8Text);
      
      const result = tryDecodeWithMultipleEncodings(uint8Array);
      
      expect(result.text).toBe(utf8Text);
      expect(result.encoding).toBe('utf-8');
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it('空の配列の場合は適切にハンドリングする', () => {
      const uint8Array = new Uint8Array(0);
      
      const result = tryDecodeWithMultipleEncodings(uint8Array);
      
      expect(result.text).toBe('');
      expect(result.confidence).toBeGreaterThanOrEqual(0);
    });

    it('無効なバイト配列でも結果を返す', () => {
      // 無効なUTF-8シーケンス
      const uint8Array = new Uint8Array([0xFF, 0xFE, 0xFD]);
      
      const result = tryDecodeWithMultipleEncodings(uint8Array);
      
      expect(result).toBeDefined();
      expect(typeof result.text).toBe('string');
      expect(typeof result.encoding).toBe('string');
      expect(typeof result.confidence).toBe('number');
    });
  });
});
