import { renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { useAuth } from '../useAuth';

describe('useAuth', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('AuthProvider の外で useAuth を呼ぶと Error を throw する', () => {
    vi.spyOn(console, 'error').mockImplementation(() => {});

    expect(() => renderHook(() => useAuth())).toThrow(
      'useAuth must be used within an AuthProvider'
    );
  });
});
