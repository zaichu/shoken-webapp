import { describe, it, expect, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import React from 'react';
import { ResizeContext } from '@/contexts/ResizeContextDefinition';
import { useForceResize, useTriggerResize } from '../useResize';

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <ResizeContext.Provider value={{ forceResize: 1, triggerResize: vi.fn() }}>
    {children}
  </ResizeContext.Provider>
);

describe('useForceResize', () => {
  it('Provider外ではundefinedを返す', () => {
    const { result } = renderHook(() => useForceResize());
    expect(result.current).toBeUndefined();
  });

  it('Provider内ではcontextのforceResize値を返す', () => {
    const { result } = renderHook(() => useForceResize(), { wrapper });
    expect(result.current).toBe(1);
  });
});

describe('useTriggerResize', () => {
  it('Provider外ではundefinedを返す', () => {
    const { result } = renderHook(() => useTriggerResize());
    expect(result.current).toBeUndefined();
  });

  it('Provider内ではcontextのtriggerResize関数を返す', () => {
    const { result } = renderHook(() => useTriggerResize(), { wrapper });
    expect(typeof result.current).toBe('function');
  });
});
