import '@testing-library/jest-dom';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

// テスト後のクリーンアップ
afterEach(() => {
  cleanup();
});

// グローバルなモックや設定をここに追加できます

// window.matchMedia のモック
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => {},
  }),
});

// IntersectionObserver のモック
global.IntersectionObserver = class IntersectionObserver {
  constructor() {}
  disconnect() {}
  observe() {}
  unobserve() {}
} as any;

// React 19対応: global ISモックの追加
if (typeof global.IS_REACT_ACT_ENVIRONMENT === 'undefined') {
  global.IS_REACT_ACT_ENVIRONMENT = true;
}
