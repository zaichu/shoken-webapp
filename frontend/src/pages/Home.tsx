import { Layout } from './Layout';

export const Home = () => {
  return (
    <Layout>
      <div className="jumbotron">
        <h1 className="display-4">証券Webへようこそ</h1>
        <p className="lead">このプラットフォームでは、銘柄検索や受取金の管理ができます。</p>
        <hr className="my-4" />
        <p>さまざまな機能を使って、投資をより効率的に管理しましょう。</p>
      </div>
    </Layout>
  );
}
