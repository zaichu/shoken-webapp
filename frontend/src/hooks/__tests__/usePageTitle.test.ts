import { renderHook } from '@testing-library/react';
import { describe, it, expect, afterEach } from 'vitest';
import { usePageTitle } from '../usePageTitle';

describe('usePageTitle', () => {
  afterEach(() => {
    document.title = '';
  });

  it('pageTitleを指定するとタイトルが設定される', () => {
    renderHook(() => usePageTitle('銘柄検索'));

    expect(document.title).toBe('銘柄検索 - 証券Web');
  });

  it('pageTitleを省略するとベースタイトルが設定される', () => {
    renderHook(() => usePageTitle());

    expect(document.title).toBe('証券Web');
  });

  it('アンマウント時にベースタイトルに戻る', () => {
    const { unmount } = renderHook(() => usePageTitle('資産管理'));
    expect(document.title).toBe('資産管理 - 証券Web');

    unmount();

    expect(document.title).toBe('証券Web');
  });
});
