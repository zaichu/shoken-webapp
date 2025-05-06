import { Layout } from '../components/templates/Layout';

export function HomePage() {
  return (
    <Layout>
      <div className="jumbotron">
        <h1 className="display-4">証券Webへようこそ</h1>
        <p className="lead">このプラットフォームでは、銘柄検索や受取金の管理ができます。</p>
        <hr className="my-4" />
        <p>さまざまな機能を使って、投資をより効率的に管理しましょう。</p>

        <div className="row mt-5">
          <div className="col-md-4 mb-4">
            <div className="card h-100">
              <div className="card-body">
                <h5 className="card-title">銘柄検索</h5>
                <p className="card-text">日本の株式銘柄情報を検索できます。コードや銘柄名から簡単に検索できます。</p>
              </div>
            </div>
          </div>

          <div className="col-md-4 mb-4">
            <div className="card h-100">
              <div className="card-body">
                <h5 className="card-title">受取金</h5>
                <p className="card-text">配当金や分配金の記録を管理できます。CSVファイルのインポートにも対応しています。</p>
              </div>
            </div>
          </div>

          <div className="col-md-4 mb-4">
            <div className="card h-100">
              <div className="card-body">
                <h5 className="card-title">投資情報</h5>
                <p className="card-text">株式や投資信託に関する情報にアクセスできます。効率的な投資判断をサポートします。</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}
