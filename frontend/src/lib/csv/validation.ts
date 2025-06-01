/**
 * CSVデータの検証を担当するユーティリティ
 */

export interface ValidationResult {
  valid: boolean;
  errors: string[];
}

/**
 * CSVのヘッダー行を処理する
 */
export const processHeaders = (headers: string[]): string[] => {
  return headers.map(header => {
    // 前後の空白を削除
    let processed = header.trim();

    // BOMを削除（念のため）
    processed = processed.replace(/^\uFEFF/, '');

    // 引用符を削除
    processed = processed.replace(/^["']|["']$/g, '');

    return processed;
  });
};

/**
 * CSVデータを検証する
 */
export const validateCSVData = (
  data: Record<string, unknown>[],
  requiredFields: string[]
): ValidationResult => {
  const errors: string[] = [];

  if (!data || data.length === 0) {
    errors.push('データが空です');
    return { valid: false, errors };
  }

  // ヘッダーの検証
  const headers = Object.keys(data[0]);
  const missingFields = requiredFields.filter(field => !headers.includes(field));

  if (missingFields.length > 0) {
    errors.push(`必須フィールドが不足しています: ${missingFields.join(', ')}`);
  }

  // 各行のデータ検証
  data.forEach((row, index) => {
    requiredFields.forEach(field => {
      if (row[field] === null || row[field] === undefined || row[field] === '') {
        errors.push(`行 ${index + 1}: "${field}" が空です`);
      }
    });
  });

  return {
    valid: errors.length === 0,
    errors,
  };
};

/**
 * CSVファイルのサイズ検証
 */
export const validateFileSize = (file: File, maxSizeInMB: number = 50): void => {
  const maxSizeInBytes = maxSizeInMB * 1024 * 1024;
  if (file.size > maxSizeInBytes) {
    throw new Error(`ファイルサイズが大きすぎます。最大${maxSizeInMB}MBまでです。`);
  }
};

/**
 * パース結果のエラーを検証
 */
export const validateParseErrors = (errors: any[]): void => {
  if (errors.length === 0) return;

  const criticalErrors = errors.filter(e =>
    e.type === 'Quotes' || e.type === 'FieldMismatch'
  );

  if (criticalErrors.length > 0) {
    const errorMessage = criticalErrors
      .map(e => `行 ${e.row}: ${e.message}`)
      .join('\n');
    throw new Error(`CSV解析エラー:\n${errorMessage}`);
  }

  // 軽微なエラーは警告として表示
  console.warn('CSV解析警告:', errors);
};

/**
 * パース結果のデータとメタデータを検証
 */
export const validateParseResult = (data: Record<string, unknown>[], fields: string[] | undefined): void => {
  if (!data || data.length === 0) {
    throw new Error('CSVデータが見つかりませんでした');
  }

  if (!fields || fields.length === 0) {
    throw new Error('CSVヘッダーが見つかりませんでした');
  }
};
