import { ErrorPage } from '../components/templates/ErrorPage';
import { Layout } from '../components/templates/Layout';

export function NotFoundPage() {
  return (
    <Layout>
      <ErrorPage
        title="404 - ページが見つかりません"
        message="お探しのページは存在しないか、移動した可能性があります。"
        showHomeButton={true}
        showRelatedLinks={true}
      />
    </Layout>
  );
}
