import type { components } from '@/generated/api';
import { apiClient } from '@/lib/api/client';
import { withCredentialsConfig } from '@/lib/api/requestHelpers';

// 生成型のエイリアス（API契約の正本は docs/openapi.json）
export type DividendPerShareItem = components['schemas']['DividendPerShareItem'];
export type DividendStatus = DividendPerShareItem['status'];

type BatchResponse = components['schemas']['DividendPerShareBatchResponse'];

export async function fetchDividendPerShareBatch(
  securityCodes: string[]
): Promise<DividendPerShareItem[]> {
  const response = await apiClient.post<BatchResponse>(
    '/api/v1/dividend-per-share-estimates',
    { security_codes: securityCodes },
    withCredentialsConfig
  );
  return response.items;
}
