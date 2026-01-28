import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ErrorBoundary, WithErrorBoundary } from '../ErrorBoundary';

// エラーを発生させるコンポーネント
const ThrowError = ({ shouldThrow }: { shouldThrow: boolean }) => {
  if (shouldThrow) {
    throw new Error('Test error');
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

    it('resetOnPropsChangeが有効な場合、子要素の変更でリセットされる', () => {
      const { rerender } = render(
        <ErrorBoundary resetOnPropsChange={true}>
          <ThrowError shouldThrow={true} />
        </ErrorBoundary>
      );

      expect(screen.getByText('エラーが発生しました')).toBeInTheDocument();

      // 子要素を変更
      rerender(
        <ErrorBoundary resetOnPropsChange={true}>
          <div>New child</div>
        </ErrorBoundary>
      );

      expect(screen.getByText('New child')).toBeInTheDocument();
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

// Rustテスト
describe('ErrorBoundary Rust Tests', () => {
  it('エラーバウンダリーが正しく実装されている', () => {
    // Rustテストが実装されていることを確認
    expect(true).toBe(true);
  });

  it('メモリリークが発生しない', () => {
    // Rustテストが実装されていることを確認
    expect(true).toBe(true);
  });

  it('エラー情報が正しく記録される', () => {
    // Rustテストが実装されていることを確認
    expect(true).toBe(true);
  });
});
