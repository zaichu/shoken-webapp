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
    <div className="container py-5">
      <div className="text-center">
        <h1 className="display-1 fw-bold text-danger">⚠️</h1>
        <h2 className="mt-4">{title}</h2>
        <p className="lead my-4">{message}</p>
        {showHomeButton && (
          <Link to="/">
            <Button variant="primary">ホームに戻る</Button>
          </Link>
        )}
      </div>
    </div>
  );
}
