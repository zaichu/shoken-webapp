import { apiClient } from '@/lib/api/client';
import { withCredentialsConfig } from '@/lib/api/requestHelpers';

export type DividendStatus = 'ok' | 'zero' | 'pending' | 'error';

export interface DividendPerShareItem {
  security_code: string;
  dividend_per_share: number | null;
  status: DividendStatus;
  fetched_at: string | null;
  is_stale: boolean;
}

interface BatchResponse {
  items: DividendPerShareItem[];
}

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
