/// <reference types="vitest/globals" />

import type { TestingLibraryMatchers } from "@testing-library/jest-dom/matchers";

declare module "vitest" {
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  interface Assertion<T = unknown>
    extends TestingLibraryMatchers<T, void> {}
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  interface AsymmetricMatchersContaining
    extends TestingLibraryMatchers<unknown, void> {}
}

// ResizeObserver API の型定義を追加
declare global {
  interface ResizeObserverSize {
    readonly blockSize: number;
    readonly inlineSize: number;
  }

  interface ResizeObserverEntry {
    readonly target: Element;
    readonly contentRect: DOMRectReadOnly;
    readonly borderBoxSize: ReadonlyArray<ResizeObserverSize>;
    readonly contentBoxSize: ReadonlyArray<ResizeObserverSize>;
    readonly devicePixelContentBoxSize: ReadonlyArray<ResizeObserverSize>;
  }

  interface ResizeObserver {
    observe(target: Element): void;
    unobserve(target: Element): void;
    disconnect(): void;
  }

  interface ResizeObserverCallback {
    (entries: ResizeObserverEntry[], observer: ResizeObserver): void;
  }

  const ResizeObserver: {
    prototype: ResizeObserver;
    new(callback: ResizeObserverCallback): ResizeObserver;
  };
}

