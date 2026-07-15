import { Link } from 'react-router-dom';
import { Button } from '../atoms/Button';
import { EmptyState } from '../atoms/EmptyState';
import { ROUTE_PATHS } from '@/routes/routePaths';

interface ErrorPageProps {
  title?: string;
  message?: string;
  showHomeButton?: boolean;
  showRelatedLinks?: boolean;
}

const RELATED_LINKS = [
  { to: ROUTE_PATHS.home, label: 'ホーム' },
  { to: ROUTE_PATHS.search, label: '銘柄検索' },
  { to: ROUTE_PATHS.receipts, label: '取引明細' },
] as const;

export function ErrorPage({
  title = 'エラーが発生しました',
  message = 'アプリケーションで問題が発生しました。もう一度お試しください。',
  showHomeButton = true,
  showRelatedLinks = false
}: ErrorPageProps) {
  return (
    <div className="min-h-[50vh] flex items-center justify-center">
      <EmptyState
        icon={
          <svg className="h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        }
        title={title}
        description={message}
        headingLevel="h1"
        action={
          <div className="flex flex-col items-center gap-4">
            {showHomeButton && (
              <Link to={ROUTE_PATHS.home}>
                <Button variant="primary" size="md">ホームに戻る</Button>
              </Link>
            )}
            {showRelatedLinks && (
              <div className="flex flex-wrap justify-center gap-3">
                {RELATED_LINKS.map(({ to, label }) => (
                  <Link
                    key={to}
                    to={to}
                    className="text-base text-primary hover:underline"
                  >
                    {label}
                  </Link>
                ))}
              </div>
            )}
          </div>
        }
      />
    </div>
  );
}
