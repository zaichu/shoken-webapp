import '@testing-library/jest-dom';
import { vi, beforeEach } from 'vitest';

import * as matchers from '@testing-library/jest-dom/matchers';
import { expect } from 'vitest';

expect.extend(matchers);

// ResizeObserver のモック実装
class MockResizeObserver {
  private callback: ResizeObserverCallback;
  private elements: Set<Element> = new Set();

  constructor(callback: ResizeObserverCallback) {
    this.callback = callback;
  }

  observe(target: Element): void {
    this.elements.add(target);
  }

  unobserve(target: Element): void {
    this.elements.delete(target);
  }

  disconnect(): void {
    this.elements.clear();
  }

  trigger(): void {
    const entries: ResizeObserverEntry[] = Array.from(this.elements).map(
      (element) => ({
        target: element,
        contentRect: element.getBoundingClientRect(),
        borderBoxSize: [] as ResizeObserverSize[],
        contentBoxSize: [] as ResizeObserverSize[],
        devicePixelContentBoxSize: [] as ResizeObserverSize[],
      } as ResizeObserverEntry)
    );
    this.callback(entries, this);
  }
}

// グローバルにResizeObserverのモックを設定
global.ResizeObserver = MockResizeObserver as typeof ResizeObserver;

// 各テストの前にクリーンアップ
beforeEach(() => {
  vi.clearAllMocks();
});

