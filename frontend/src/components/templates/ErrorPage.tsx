import { Link } from 'react-router-dom';
import { Button } from '../atoms/Button';
import { EmptyState } from '../atoms/EmptyState';

interface ErrorPageProps {
  title?: string;
  message?: string;
  showHomeButton?: boolean;
}

export function ErrorPage({
  title = 'エラーが発生しました',
  message = 'アプリケーションで問題が発生しました。もう一度お試しください。',
  showHomeButton = true
}: ErrorPageProps) {
  return (
    <div className="min-h-[60vh] flex items-center justify-center">
      <EmptyState
        icon={
          <svg className="h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        }
        title={title}
        description={message}
        action={
          showHomeButton && (
            <Link to="/">
              <Button variant="primary">ホームに戻る</Button>
            </Link>
          )
        }
      />
    </div>
  );
}
