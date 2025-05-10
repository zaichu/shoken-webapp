// importを追加
import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Jest互換性のための設定
import * as matchers from '@testing-library/jest-dom/matchers';
import { expect } from 'vitest';

// jest-domのマッチャーを有効化
expect.extend(matchers);

// Jest互換性のためのグローバル設定
(globalThis as any).jest = {
  fn: vi.fn,
  spyOn: vi.spyOn,
  mocked: vi.mocked
};

// ResizeObserver mock
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
})) as any;
