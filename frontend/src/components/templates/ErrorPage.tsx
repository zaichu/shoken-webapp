import { Button } from '../atoms/Button';

interface ErrorPageProps {
  title?: string;
  message?: string;
  onRetry?: () => void;
}

export function ErrorPage({
  title = 'エラーが発生しました',
  message = '申し訳ありませんが、問題が発生しました。もう一度お試しください。',
  onRetry
}: ErrorPageProps) {
  return (
    <div className="text-center py-5">
      <div className="mb-4">
        <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" fill="currentColor" className="bi bi-exclamation-triangle text-warning" viewBox="0 0 16 16">
          <path d="M8 1.5a6.5 6.5 0 1 1 0 13 6.5 6.5 0 0 1 0-13zm0 1a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11z"/>
          <path d="M7.25 7.5v3a.75.75 0 0 0 1.5 0v-3a.75.75 0 0 0-1.5 0zm.75-3a1 1 0 1 0 0 2 1 1 0 0 0 0-2z"/>
        </svg>
      </div>
      <h2>{title}</h2>
      <p className="lead mb-4">{message}</p>
      {onRetry && (
        <Button
          variant="primary"
          onClick={onRetry}
        >
          再試行
        </Button>
      )}
    </div>
  );
}
