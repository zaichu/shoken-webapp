import { createRef } from 'react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { ErrorBoundary, WithErrorBoundary } from '../ErrorBoundary';

// エラーを発生させるコンポーネント
const ThrowError = ({
  shouldThrow,
  message = 'Test error',
}: {
  shouldThrow: boolean;
  message?: string;
}) => {
  if (shouldThrow) {
    throw new Error(message);
  }
  return <div>No error</div>;
};

describe('ErrorBoundary', () => {
  // コンソールエラーをモック化
  const originalError = console.error;
  beforeEach(() => {
    console.error = vi.fn();
  });

  afterEach(() => {
    console.error = originalError;
    vi.clearAllMocks();
  });

  describe('基本的な動作', () => {
    it('エラーがない場合は子コンポーネントをレンダリングする', () => {
      render(
        <ErrorBoundary>
          <div>Child component</div>
        </ErrorBoundary>
      );

      expect(screen.getByText('Child component')).toBeInTheDocument();
    });

    it('エラーが発生した場合はフォールバックを表示する', () => {
      render(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(screen.getByText('エラーが発生しました')).toBeInTheDocument();
      expect(screen.getByText('再試行')).toBeInTheDocument();
    });

    it('カスタムフォールバックを表示できる', () => {
      const customFallback = <div>Custom error message</div>;

      render(
        <ErrorBoundary fallback={customFallback}>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(screen.getByText('Custom error message')).toBeInTheDocument();
    });

    it('関数型フォールバックでエラー情報を受け取れる', () => {
      const fallbackFn = vi.fn((error) => (
        <div>Error: {error.message}</div>
      ));

      render(
        <ErrorBoundary fallback={fallbackFn}>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(screen.getByText('Error: Test error')).toBeInTheDocument();
      expect(fallbackFn).toHaveBeenCalledWith(
        expect.objectContaining({ message: 'Test error' }),
        expect.any(Object)
      );
    });
  });

  describe('エラーハンドリング', () => {
    it('onErrorコールバックが呼ばれる', () => {
      const onError = vi.fn();

      render(
        <ErrorBoundary onError={onError}>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(onError).toHaveBeenCalledWith(
        expect.objectContaining({ message: 'Test error' }),
        expect.any(Object)
      );
    });

    it('コンソールにエラーがログ出力される', () => {
      render(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(console.error).toHaveBeenCalledWith(
        'ErrorBoundary caught an error:',
        'Test error'
      );
    });
  });

  describe('エラーリセット', () => {
    it('再試行ボタンでエラー状態をリセットできる', () => {
      let shouldThrow = true;

      const TestComponent = () => {
        if (shouldThrow) {
          throw new Error('Test error');
        }
        return <div>No error</div>;
      };

      render(
        <ErrorBoundary>
          <TestComponent />
        </ErrorBoundary>
      );

      expect(screen.getByText('エラーが発生しました')).toBeInTheDocument();

      // エラーを発生させないように設定
      shouldThrow = false;

      // 再試行ボタンをクリック
      fireEvent.click(screen.getByText('再試行'));

      expect(screen.getByText('No error')).toBeInTheDocument();
    });

    it('resetKeysが変更されるとエラー状態がリセットされる', () => {
      const { rerender } = render(
        <ErrorBoundary resetKeys={['key1']}>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(screen.getByText('エラーが発生しました')).toBeInTheDocument();

      // resetKeysを変更
      rerender(
        <ErrorBoundary resetKeys={['key2']}>
          <ThrowError shouldThrow={false} />
        </ErrorBoundary>
      );

      expect(screen.getByText('No error')).toBeInTheDocument();
    });

    it('resetOnPropsChangeが有効な場合、子要素の変更でresetErrorBoundaryが呼ばれる', () => {
      const boundaryRef = createRef<ErrorBoundary>();
      const { rerender } = render(
        <ErrorBoundary ref={boundaryRef} resetOnPropsChange={true}>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(screen.getByText('エラーが発生しました')).toBeInTheDocument();
      const resetSpy = vi.spyOn(boundaryRef.current!, 'resetErrorBoundary');

      // 子要素を変更
      rerender(
        <ErrorBoundary ref={boundaryRef} resetOnPropsChange={true}>
          <div>New child</div>
        </ErrorBoundary>
      );

      expect(resetSpy).toHaveBeenCalledTimes(1);
      expect(screen.getByText('New child')).toBeInTheDocument();
    });

    it('resetOnPropsChangeが無効な場合、子要素が変わってもリセットされない', () => {
      const boundaryRef = createRef<ErrorBoundary>();
      const { rerender } = render(
        <ErrorBoundary ref={boundaryRef} resetOnPropsChange={false}>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(screen.getByText('エラーが発生しました')).toBeInTheDocument();
      const resetSpy = vi.spyOn(boundaryRef.current!, 'resetErrorBoundary');

      rerender(
        <ErrorBoundary ref={boundaryRef} resetOnPropsChange={false}>
          <div>New child</div>
        </ErrorBoundary>
      );

      expect(resetSpy).not.toHaveBeenCalled();
      expect(screen.getByText('エラーが発生しました')).toBeInTheDocument();
      expect(screen.queryByText('New child')).not.toBeInTheDocument();
    });

    it('resetKeysが不足している場合、hasResetKeyChangedはfalseを返す', () => {
      const boundaryRef = createRef<ErrorBoundary>();

      render(
        <ErrorBoundary ref={boundaryRef}>
          <div>Child component</div>
        </ErrorBoundary>
      );

      expect(boundaryRef.current?.hasResetKeyChanged(['key1'])).toBe(false);
    });

    it('resetTimeoutIdが設定されている状態でunmountするとclearTimeoutが呼ばれる', () => {
      const clearTimeoutSpy = vi.spyOn(globalThis, 'clearTimeout');
      const boundaryRef = createRef<ErrorBoundary>();
      const { unmount } = render(
        <ErrorBoundary ref={boundaryRef}>
          <div>Child component</div>
        </ErrorBoundary>
      );
      const timeoutId = window.setTimeout(() => undefined, 1000);

      (boundaryRef.current as unknown as { resetTimeoutId: number | null }).resetTimeoutId = timeoutId;

      unmount();

      expect(clearTimeoutSpy).toHaveBeenCalledWith(timeoutId);
    });
  });

  describe('エラー頻発時の処理', () => {
    it('エラーが4回以上発生すると特別なメッセージが表示される', () => {
      const ErrorComponent = ({ count }: { count: number }) => {
        if (count < 4) {
          throw new Error(`Error ${count}`);
        }
        return <div>Success</div>;
      };

      const { rerender } = render(
        <ErrorBoundary>
          <ErrorComponent count={0} />
        </ErrorBoundary>
      );

      // エラーを3回発生させる
      for (let i = 1; i <= 3; i++) {
        fireEvent.click(screen.getByText('再試行'));
        rerender(
          <ErrorBoundary>
            <ErrorComponent count={i} />
          </ErrorBoundary>
        );
      }

      // ErrorBoundaryのerrorCountが3を超えているので、
      // 次のレンダリングで特別なメッセージが表示される
      rerender(
        <ErrorBoundary>
          <ErrorComponent count={3} />
        </ErrorBoundary>
      );

      expect(screen.getByText('エラーが頻発しています')).toBeInTheDocument();
      expect(screen.getByText('ページを再読み込み')).toBeInTheDocument();
    });

    it('ページ再読み込みボタンを押せる', () => {
      const boundaryRef = createRef<ErrorBoundary>();

      render(
        <ErrorBoundary ref={boundaryRef}>
          <div>Child component</div>
        </ErrorBoundary>
      );

      act(() => {
        boundaryRef.current?.setState({
          hasError: true,
          error: new Error('Too many errors'),
          errorCount: 4,
        });
      });

      expect(() => {
        fireEvent.click(screen.getByText('ページを再読み込み'));
      }).not.toThrow();
    });
  });

  describe('追加のレンダリング分岐', () => {
    it('isolateが有効な場合、デフォルトフォールバックにisolatedクラスが付く', () => {
      render(
        <ErrorBoundary isolate={true}>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(screen.getByRole('alert').parentElement).toHaveClass('isolated');
    });

    it('エラーメッセージが空文字の場合はUnknown errorをログ出力する', () => {
      render(
        <ErrorBoundary>
          <ThrowError shouldThrow={true} message="" />
        </ErrorBoundary>
      );

      expect(console.error).toHaveBeenCalledWith(
        'ErrorBoundary caught an error:',
        'Unknown error'
      );
    });
  });

  describe('WithErrorBoundary', () => {
    it('ラッパーコンポーネントが正しく動作する', () => {
      render(
        <WithErrorBoundary>
          <div>Wrapped content</div>
        </WithErrorBoundary>
      );

      expect(screen.getByText('Wrapped content')).toBeInTheDocument();
    });

    it('エラー時にフォールバックを表示する', () => {
      const fallback = <div>Error fallback</div>;

      render(
        <WithErrorBoundary fallback={fallback}>
          <ThrowError shouldThrow={true} />
        </WithErrorBoundary>
      );

      expect(screen.getByText('Error fallback')).toBeInTheDocument();
    });

    it('onErrorコールバックが動作する', () => {
      const onError = vi.fn();

      render(
        <WithErrorBoundary onError={onError}>
          <ThrowError shouldThrow={true} />
        </WithErrorBoundary>
      );

      expect(onError).toHaveBeenCalled();
    });
  });
});
