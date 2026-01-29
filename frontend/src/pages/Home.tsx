import { Layout } from '../components/templates/Layout';
import { Card, CardBody } from '../components/atoms/Card';

export function HomePage() {
  return (
    <Layout>
      <div className="rounded-lg border border-border bg-white p-6 shadow-sm">
        <h1 className="text-3xl font-bold">証券Webへようこそ</h1>
        <p className="mt-2 text-sm text-secondary">このプラットフォームでは、銘柄検索や受取金の管理ができます。</p>
        <hr className="my-4 border-border" />
        <p className="text-sm text-dark">さまざまな機能を使って、投資をより効率的に管理しましょう。</p>

        <div className="mt-6 grid gap-4 md:grid-cols-3">
          <Card className="h-full">
            <CardBody className="space-y-2">
              <h5 className="text-base font-semibold">銘柄検索</h5>
              <p className="text-sm text-secondary">日本の株式銘柄情報を検索できます。コードや銘柄名から簡単に検索できます。</p>
            </CardBody>
          </Card>

          <Card className="h-full">
            <CardBody className="space-y-2">
              <h5 className="text-base font-semibold">受取金</h5>
              <p className="text-sm text-secondary">配当金や分配金の記録を管理できます。CSVファイルのインポートにも対応しています。</p>
            </CardBody>
          </Card>

          <Card className="h-full">
            <CardBody className="space-y-2">
              <h5 className="text-base font-semibold">投資情報</h5>
              <p className="text-sm text-secondary">株式や投資信託に関する情報にアクセスできます。効率的な投資判断をサポートします。</p>
            </CardBody>
          </Card>
        </div>
      </div>
    </Layout>
  );
}
