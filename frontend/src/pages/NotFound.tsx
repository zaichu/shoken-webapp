import { Layout } from './Layout';

export const NotFound = () => {
  return (
    <Layout>
      <div className="text-center mt-5">
        <h1>404 - ページが見つかりません</h1>
        <p>お探しのページは存在しないか、移動した可能性があります。</p>
      </div>
    </Layout>
  );
}
