/**
 * CSVファイルのエンコーディング検出と変換を担当するユーティリティ
 */

// サポートされているエンコーディング（utf-8 を先頭に置くことで信頼度同点時に utf-8 を優先する）
const SUPPORTED_ENCODINGS = ['utf-8', 'shift-jis', 'iso-8859-1', 'euc-jp'] as const;

interface DecodeResult {
  text: string;
  encoding: string;
  confidence: number;
}

/**
 * 文字化けを検出する
 */
export const detectMojibake = (text: string): boolean => {
  return /[\uFFFD]/.test(text);
};

/**
 * テキストのエンコーディング信頼度を計算
 */
export const calculateEncodingConfidence = (text: string): number => {
  if (!text || text.length === 0) return 0;

  let score = 1.0;

  // 文字化けチェック
  if (detectMojibake(text)) {
    score -= 0.5;
  }

  // 日本語文字の存在をチェック
  const hasJapanese = /[\u3000-\u303f\u3040-\u309f\u30a0-\u30ff\u4e00-\u9faf]/.test(text);
  if (hasJapanese) {
    score += 0.2;
  }

  // 制御文字の存在をチェック（改行とタブ以外）
  // eslint-disable-next-line no-control-regex
  const hasControlChars = /[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/.test(text);
  if (hasControlChars) {
    score -= 0.3;
  }

  return Math.max(0, Math.min(1, score));
};

/**
 * 複数のエンコーディングを試して正常に読み込めるものを使用
 */
export const tryDecodeWithMultipleEncodings = (uint8Array: Uint8Array): DecodeResult => {
  // UTF-8 BOM (0xEF 0xBB 0xBF) が存在する場合は UTF-8 と確定できる
  const hasBOM = uint8Array.length >= 3 &&
    uint8Array[0] === 0xEF && uint8Array[1] === 0xBB && uint8Array[2] === 0xBF;
  if (hasBOM) {
    const decoder = new TextDecoder('utf-8', { fatal: false });
    return { text: decoder.decode(uint8Array), encoding: 'utf-8', confidence: 1.0 };
  }

  const results: DecodeResult[] = [];

  for (const encoding of SUPPORTED_ENCODINGS) {
    try {
      let decoder: TextDecoder;

      // ブラウザ環境でサポートされていないエンコーディングはスキップ
      try {
        decoder = new TextDecoder(encoding, { fatal: false });
      } catch (e) {
        console.warn(`エンコーディング ${encoding} はサポートされていません:`, e instanceof Error ? e.message : String(e));
        continue;
      }

      // デコード実行
      let text: string;
      try {
        text = decoder.decode(uint8Array);
      } catch (error) {
        console.warn(`${encoding} でのデコードに失敗しました:`, error instanceof Error ? error.message : String(error));
        continue;
      }

      const confidence = calculateEncodingConfidence(text);
      results.push({ text, encoding, confidence });
    } catch (error) {
      console.warn(`${encoding} でのデコードに失敗しました:`, error instanceof Error ? error.message : String(error));
    }
  }

  // 信頼度が最も高いものを選択
  results.sort((a, b) => b.confidence - a.confidence);

  if (results.length > 0 && results[0].confidence > 0.5) {
    return results[0];
  }

  // 全て失敗した場合はutf-8で強制的にデコード
  try {
    const decoder = new TextDecoder('utf-8', { fatal: false });
    return {
      text: decoder.decode(uint8Array),
      encoding: 'utf-8',
      confidence: 0.3
    };
  } catch {
    // utf-8もダメな場合は文字列変換
    const text = Array.from(uint8Array).map(byte => String.fromCharCode(byte)).join('');
    return {
      text,
      encoding: 'unknown',
      confidence: 0.1
    };
  }
};

/**
 * BOMを削除する
 */
export const removeBOM = (text: string): string => {
  return text.replace(/^\uFEFF/, '');
};
