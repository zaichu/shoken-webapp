import { Button } from '../atoms/Button';
import { Link } from 'react-router-dom';

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
    <div className="mx-auto flex min-h-[60vh] max-w-xl flex-col items-center justify-center px-4 py-12 text-center">
      <div className="text-6xl font-bold text-danger">⚠️</div>
      <h2 className="mt-4 text-2xl font-semibold">{title}</h2>
      <p className="mt-4 text-base text-secondary">{message}</p>
        {showHomeButton && (
        <Link to="/" className="mt-6">
          <Button variant="primary">ホームに戻る</Button>
        </Link>
        )}
    </div>
  );
}
