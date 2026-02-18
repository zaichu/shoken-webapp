import { apiClient } from '@/lib/api/client';

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
    '/dividends/per-share/batch',
    { security_codes: securityCodes }
  );
  return response.items;
}
