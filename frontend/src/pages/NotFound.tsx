import { ErrorPage } from '../components/templates/ErrorPage';

export function NotFoundPage() {
  return (
    <ErrorPage
      title="404 - ページが見つかりません"
      message="お探しのページは存在しないか、移動した可能性があります。"
      showHomeButton={true}
      showRelatedLinks={true}
    />
  );
}
