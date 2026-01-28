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
  private previousResetKeys: Array<string | number> = [];

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
          <div className="error-boundary-fallback p-4">
            <div className="alert alert-danger">
              <h4 className="alert-heading">エラーが頻発しています</h4>
              <p>アプリケーションで問題が発生しています。ページを再読み込みしてください。</p>
              <button
                className="btn btn-outline-danger"
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
        <div className={`error-boundary-fallback ${isolate ? 'isolated' : ''} p-4`}>
          <div className="alert alert-danger" role="alert">
            <h4 className="alert-heading">エラーが発生しました</h4>
            <p className="mb-0">申し訳ございません。予期しないエラーが発生しました。</p>
            {import.meta.env.DEV && (
              <details className="mt-3">
                <summary>エラー詳細（開発環境のみ）</summary>
                <pre className="mt-2 p-2 bg-light">
                  <code>{error.stack}</code>
                </pre>
              </details>
            )}
            <hr />
            <button
              className="btn btn-primary"
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

// エラーバウンダリーのフック（実験的）
// eslint-disable-next-line react-refresh/only-export-components
export function useErrorHandler() {
  return (error: Error) => {
    throw error;
  };
}
