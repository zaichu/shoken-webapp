// 決算情報 API レスポンスの型定義（生成済みスキーマをそのまま使用）
import type { components } from '@/generated/api';

/**
 * 決算サマリーデータ（V2、省略形フィールド名を使用）
 */
export type FinancialStatementData = components['schemas']['FinancialStatementData'];

/**
 * 決算サマリーレスポンス（V2、fins/summary を使用し、ルートフィールドは "data"）
 */
export type FinancialStatementsResponse = components['schemas']['FinancialStatementsResponse'];
