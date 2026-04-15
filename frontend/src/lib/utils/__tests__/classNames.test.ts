import { describe, it, expect } from 'vitest';
import { cn } from '../classNames';

describe('cn', () => {
  it('1つのクラス名をそのまま返す', () => {
    expect(cn('base')).toBe('base');
  });

  it('複数のクラス名をスペースで結合する', () => {
    expect(cn('foo', 'bar', 'baz')).toBe('foo bar baz');
  });

  it('false を除外する', () => {
    expect(cn('base', false)).toBe('base');
  });

  it('null を除外する', () => {
    expect(cn('base', null)).toBe('base');
  });

  it('undefined を除外する', () => {
    expect(cn('base', undefined)).toBe('base');
  });

  it('条件分岐パターン: isActive=true のとき active が付く', () => {
    const isActive = true;
    expect(cn('base', isActive && 'active')).toBe('base active');
  });

  it('条件分岐パターン: isActive=false のとき active が付かない', () => {
    const isActive = false;
    expect(cn('base', isActive && 'active')).toBe('base');
  });
});
