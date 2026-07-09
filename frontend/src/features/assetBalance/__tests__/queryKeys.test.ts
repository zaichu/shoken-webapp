import { describe, it, expect, vi } from 'vitest';
import type { QueryClient } from '@tanstack/react-query';
import { assetBalanceQueryKeys, clearAssetBalanceCache } from '../queryKeys';

describe('assetBalanceQueryKeys', () => {
  it('prefix が ["assetBalance"] である', () => {
    expect(assetBalanceQueryKeys.prefix).toEqual(['assetBalance']);
  });

  it('all("user-1") が ["assetBalance", "user-1"] を返す', () => {
    expect(assetBalanceQueryKeys.all('user-1')).toEqual(['assetBalance', 'user-1']);
  });

  it('lookup("user-1", "7203") が lookup キーを返す', () => {
    expect(assetBalanceQueryKeys.lookup('user-1', '7203')).toEqual(['assetBalance', 'user-1', 'lookup', '7203']);
  });

  it('list("user-1") が ["assetBalance", "user-1", "list"] を返す', () => {
    expect(assetBalanceQueryKeys.list('user-1')).toEqual(['assetBalance', 'user-1', 'list']);
  });

  it('all("user-2") が ["assetBalance", "user-2"] を返す', () => {
    expect(assetBalanceQueryKeys.all('user-2')).toEqual(['assetBalance', 'user-2']);
  });
});

describe('clearAssetBalanceCache', () => {
  it('cancelQueries と removeQueries がプレフィックスキーで呼ばれる', () => {
    const mockQueryClient = {
      cancelQueries: vi.fn(),
      removeQueries: vi.fn(),
    } as unknown as QueryClient;

    clearAssetBalanceCache(mockQueryClient);

    expect(mockQueryClient.cancelQueries).toHaveBeenCalledWith({
      queryKey: assetBalanceQueryKeys.prefix,
    });
    expect(mockQueryClient.removeQueries).toHaveBeenCalledWith({
      queryKey: assetBalanceQueryKeys.prefix,
    });
  });
});
