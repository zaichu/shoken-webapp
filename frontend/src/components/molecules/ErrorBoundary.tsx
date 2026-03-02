import { Component, ReactNode, ErrorInfo } from 'react';

interface ErrorBoundaryProps {
  fallback?: ReactNode | ((error: Error, errorInfo: ErrorInfo) => ReactNode);
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
  resetKeys?: Array<string | number>;
  resetOnPropsChange?: boolean;
  isolate?: boolean;
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
  errorCount: number;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  private resetTimeoutId: number | null = null;

  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      errorCount: 0
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return {
      hasError: true,
      error
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    const { onError } = this.props;

    // エラー情報をstateに保存
    this.setState(prevState => ({
      errorInfo,
      errorCount: prevState.errorCount + 1
    }));

    // エラーログを出力（機微情報の露出を避ける）
    const errorMessage = error.message || 'Unknown error';
    console.error('ErrorBoundary caught an error:', errorMessage);

    // カスタムエラーハンドラーを実行
    if (onError) {
      onError(error, errorInfo);
    }

    // 開発環境でのみ詳細なエラー情報を表示
    if (import.meta.env.DEV) {
      console.group('Error Details');
      console.error('Error:', errorMessage);
      console.error('Component Stack:', errorInfo.componentStack);
      console.groupEnd();
    }
  }

  componentDidUpdate(prevProps: ErrorBoundaryProps): void {
    const { resetKeys, resetOnPropsChange } = this.props;
    const { hasError } = this.state;

    // resetKeysが変更された場合、エラー状態をリセット
    if (hasError && resetKeys && this.hasResetKeyChanged(prevProps.resetKeys)) {
      this.resetErrorBoundary();
    }

    // resetOnPropsChangeが有効で、propsが変更された場合リセット
    if (hasError && resetOnPropsChange && prevProps.children !== this.props.children) {
      this.resetErrorBoundary();
    }
  }

  componentWillUnmount(): void {
    if (this.resetTimeoutId) {
      clearTimeout(this.resetTimeoutId);
    }
  }

  hasResetKeyChanged(prevResetKeys?: Array<string | number>): boolean {
    if (!prevResetKeys || !this.props.resetKeys) {
      return false;
    }

    return this.props.resetKeys.some(
      (key, index) => key !== prevResetKeys[index]
    );
  }

  resetErrorBoundary = (): void => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null
    });
  };

  render() {
    const { hasError, error, errorInfo, errorCount } = this.state;
    const { fallback, children, isolate } = this.props;

    if (hasError && error) {
      // エラーが頻発している場合は自動リトライを停止
      if (errorCount > 3) {
        return (
          <div className="p-4">
            <div className="bg-danger/10 border-l-4 border-danger text-danger rounded-lg p-4" role="alert">
              <h4 className="font-bold text-lg mb-2">エラーが頻発しています</h4>
              <p className="mb-4">アプリケーションで問題が発生しています。ページを再読み込みしてください。</p>
              <button
                className="px-4 py-2 border border-danger text-danger rounded hover:bg-danger hover:text-white transition-colors"
                onClick={() => window.location.reload()}
              >
                ページを再読み込み
              </button>
            </div>
          </div>
        );
      }

      // カスタムフォールバックコンポーネント
      if (typeof fallback === 'function') {
        return <>{fallback(error, errorInfo!)}</>;
      }

      // 静的フォールバックコンポーネント
      if (fallback) {
        return <>{fallback}</>;
      }

      // デフォルトフォールバック
      return (
        <div className={`p-4 ${isolate ? 'isolated' : ''}`}>
          <div className="bg-danger/10 border-l-4 border-danger text-danger rounded-lg p-4" role="alert">
            <h4 className="font-bold text-lg mb-2">エラーが発生しました</h4>
            <p>申し訳ございません。予期しないエラーが発生しました。</p>
            {import.meta.env.DEV && (
              <details className="mt-3">
                <summary className="cursor-pointer">エラー詳細（開発環境のみ）</summary>
                <pre className="mt-2 p-2 bg-gray-100 rounded text-sm overflow-x-auto">
                  <code>{error.stack}</code>
                </pre>
              </details>
            )}
            <hr className="my-4 border-danger/30" />
            <button
              className="px-4 py-2 bg-primary text-white rounded hover:bg-primary-hover transition-colors"
              onClick={this.resetErrorBoundary}
            >
              再試行
            </button>
          </div>
        </div>
      );
    }

    return children;
  }
}

// 便利なラッパーコンポーネント
interface WithErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

export function WithErrorBoundary({
  children,
  fallback,
  onError
}: WithErrorBoundaryProps) {
  return (
    <ErrorBoundary fallback={fallback} onError={onError}>
      {children}
    </ErrorBoundary>
  );
}

