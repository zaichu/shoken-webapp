import { describe, it, expect, vi } from 'vitest';
import type { QueryClient } from '@tanstack/react-query';
import { receiptQueryKeys, clearReceiptsCache } from '../queryKeys';

describe('receiptQueryKeys', () => {
  it('prefix が ["receipts"] である', () => {
    expect(receiptQueryKeys.prefix).toEqual(['receipts']);
  });

  it('dividend("user-1") が ["receipts", "user-1", "dividend"] を返す', () => {
    expect(receiptQueryKeys.dividend('user-1')).toEqual(['receipts', 'user-1', 'dividend']);
  });

  it('domesticstock("user-1") が ["receipts", "user-1", "domesticstock"] を返す', () => {
    expect(receiptQueryKeys.domesticstock('user-1')).toEqual([
      'receipts',
      'user-1',
      'domesticstock',
    ]);
  });

  it('mutualfund("user-1") が ["receipts", "user-1", "mutualfund"] を返す', () => {
    expect(receiptQueryKeys.mutualfund('user-1')).toEqual(['receipts', 'user-1', 'mutualfund']);
  });
});

describe('clearReceiptsCache', () => {
  it('cancelQueries と removeQueries がプレフィックスキーで呼ばれる', () => {
    const mockQueryClient = {
      cancelQueries: vi.fn(),
      removeQueries: vi.fn(),
    } as unknown as QueryClient;

    clearReceiptsCache(mockQueryClient);

    expect(mockQueryClient.cancelQueries).toHaveBeenCalledWith({
      queryKey: receiptQueryKeys.prefix,
    });
    expect(mockQueryClient.removeQueries).toHaveBeenCalledWith({
      queryKey: receiptQueryKeys.prefix,
    });
  });
});
