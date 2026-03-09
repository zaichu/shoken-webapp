/**
 * CSVファイルのエンコーディング検出と変換を担当するユーティリティ
 */

// サポートされているエンコーディング
const SUPPORTED_ENCODINGS = ['shift-jis', 'utf-8', 'iso-8859-1', 'euc-jp'] as const;

export interface DecodeResult {
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
  const results: DecodeResult[] = [];

  for (const encoding of SUPPORTED_ENCODINGS) {
    try {
      let decoder: TextDecoder;

      // ブラウザ環境でサポートされていないエンコーディングはスキップ
      try {
        decoder = new TextDecoder(encoding, { fatal: false });
      } catch (e) {
        console.warn(`エンコーディング ${encoding} はサポートされていません:`, e);
        continue;
      }

      // デコード実行
      let text: string;
      try {
        text = decoder.decode(uint8Array);
      } catch (error) {
        console.warn(`${encoding} でのデコードに失敗しました:`, error);
        continue;
      }

      const confidence = calculateEncodingConfidence(text);
      results.push({ text, encoding, confidence });
    } catch (error) {
      console.warn(`${encoding} でのデコードに失敗しました:`, error);
    }
  }

  // UTF-8 優先条件:
  // バイト列が有効な UTF-8（U+FFFD なしでデコード可能）であれば UTF-8 を優先して返す。
  //
  // 根拠:
  //   - 同じバイト列が UTF-8 でも Shift-JIS でも有効に見えるケースが存在する
  //     （例: 3バイト UTF-8 日本語文字のバイト列は Shift-JIS でも有効な文字列に見える）。
  //   - 実際の Shift-JIS / EUC-JP CSV（証券会社等）は必ず漢字を含む。
  //     Shift-JIS: 漢字の先頭バイト（0x81-0x9F）は UTF-8 の継続バイト → U+FFFD 生成
  //     EUC-JP: 漢字の先頭バイト（0xA1-0xFE）も UTF-8 の継続バイト → U+FFFD 生成
  //     そのため実ファイルでは cleanUtf8 は null となり正しいエンコーディングが選ばれる。
  //   - 回帰テスト: [0x82, 0xA0] (Shift-JIS かな) や [0xCC, 0xF3] (EUC-JP 漢字) 相当の
  //     バイト列は UTF-8 で U+FFFD を生成するため Shift-JIS / EUC-JP が正しく選ばれる。
  const cleanUtf8 = results.find(r => r.encoding === 'utf-8' && !r.text.includes('\uFFFD'));
  if (cleanUtf8 && cleanUtf8.confidence > 0.5) {
    return cleanUtf8;
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
