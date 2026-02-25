import React from 'react';
import { render, RenderResult } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

/** テスト用 QueryClient（自動再フェッチなし・リトライなし） */
export function makeQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: Infinity },
    },
  });
}

/** QueryClientProvider でラップしてレンダリング */
export function renderWithQuery(
  ui: React.ReactElement,
  qc?: QueryClient
): RenderResult & { queryClient: QueryClient } {
  const client = qc ?? makeQueryClient();
  const result = render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>
  );
  return { ...result, queryClient: client };
}

/** waitFor の共通タイムアウトオプション（全スイート実行時のCPU競合対策） */
export const waitOpts = { timeout: 5000 } as const;
