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
    it('UTF-8 BOM付きファイルは UTF-8 として正しくデコードされる', () => {
      // UTF-8 BOM (0xEF 0xBB 0xBF) が存在する場合は UTF-8 と確定できる。
      // Windows の Excel 等が出力する UTF-8 CSV はこの BOM を持つ。
      const utf8Text = 'こんにちは世界';
      const encoder = new TextEncoder();
      const utf8Bytes = encoder.encode(utf8Text);
      const bom = new Uint8Array([0xEF, 0xBB, 0xBF]);
      const uint8Array = new Uint8Array(bom.length + utf8Bytes.length);
      uint8Array.set(bom, 0);
      uint8Array.set(utf8Bytes, bom.length);

      const result = tryDecodeWithMultipleEncodings(uint8Array);

      expect(result.encoding).toBe('utf-8');
      expect(result.confidence).toBe(1.0);
      expect(result.text).toContain(utf8Text);
    });

    it('UTF-8 BOMなし日本語ファイルは UTF-8 として正しくデコードされる', () => {
      // SUPPORTED_ENCODINGS の先頭が utf-8 であるため、UTF-8 と Shift-JIS の信頼度が
      // 同点になった場合でも安定ソートにより utf-8 が優先して選ばれる。
      const utf8Text = 'こんにちは世界';
      const encoder = new TextEncoder();
      const uint8Array = encoder.encode(utf8Text);

      const result = tryDecodeWithMultipleEncodings(uint8Array);

      expect(result.encoding).toBe('utf-8');
      expect(result.text).toBe(utf8Text);
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

    it('Shift-JIS 単バイト半角カナは Shift-JIS として正しくデコードされる', () => {
      // 半角カタカナ「ｱｲｳ」の Shift-JIS バイト (0xB1, 0xB2, 0xB3)
      // UTF-8 では無効バイト（継続バイトが単独で出現）→ U+FFFD が生成され confidence が下がる
      const shiftJisBytes = new Uint8Array([0xB1, 0xB2, 0xB3]);

      const result = tryDecodeWithMultipleEncodings(shiftJisBytes);

      expect(result.encoding).toBe('shift-jis');
      expect(result.text).toBe('ｱｲｳ');
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it('Shift-JIS 2バイト文字（漢字等、先頭バイト 0x81-0x9F）は Shift-JIS として正しくデコードされる', () => {
      // 実際の証券 CSV は列ヘッダーに漢字を含む。
      // Shift-JIS 2バイト文字の先頭バイト 0x82 は UTF-8 の継続バイト → U+FFFD を生成し
      // cleanUtf8 が null となるため、信頼度ソートで Shift-JIS が選ばれる。
      const shiftJisKanji = new Uint8Array([0x82, 0xA0, 0x82, 0xA2]); // Shift-JIS 2バイト文字列

      const result = tryDecodeWithMultipleEncodings(shiftJisKanji);

      expect(result.encoding).toBe('shift-jis');
      expect(result.text).not.toContain('\uFFFD');
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it('EUC-JP 2バイト文字（先頭バイト 0xA1-0xFE）は正しくデコードされる', () => {
      // EUC-JP 漢字の先頭バイト（0xA4, 0xCC 等 0xA1-0xFE 範囲）は
      // UTF-8 の継続バイトのため先頭に出現すると U+FFFD → cleanUtf8 が null となり
      // 信頼度ソートに委ねられる。このテストは cleanUtf8 が EUC-JP 判定を壊さないことを確認。
      // EUC-JP 約定日 相当のバイト列（先頭が継続バイト 0xCC → UTF-8 で U+FFFD 生成）
      const eucJpBytes = new Uint8Array([0xCC, 0xF3, 0xC4, 0xEA]); // EUC-JP 2バイト文字列

      const result = tryDecodeWithMultipleEncodings(eucJpBytes);

      // UTF-8 には U+FFFD が生成されるため cleanUtf8 は null → euc-jp または shift-jis が選ばれる
      // 重要: UTF-8 が返らないこと（cleanUtf8 優先が発動しないこと）を確認
      expect(result.encoding).not.toBe('utf-8');
      expect(result.text).toBeDefined();
    });

  });
});
