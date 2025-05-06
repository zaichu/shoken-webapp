import { Link } from 'react-router-dom';

export function Footer() {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="bg-light py-3 mt-5">
      <div className="container">
        <div className="row">
          <div className="col-md-4">
            <h5>証券Webapp</h5>
            <p className="text-muted">
              投資情報と受取金管理のためのプラットフォーム
            </p>
          </div>

          <div className="col-md-4">
            <h5>リンク</h5>
            <ul className="list-unstyled">
              <li>
                <Link className="text-decoration-none" to="/">ホーム</Link>
              </li>
              <li>
                <Link className="text-decoration-none" to="/search">銘柄検索</Link>
              </li>
              <li>
                <Link className="text-decoration-none" to="/receipts">受取金管理</Link>
              </li>
            </ul>
          </div>

          <div className="col-md-4">
            <h5>お問い合わせ</h5>
            <p className="text-muted">
              ご質問やご意見がありましたら、お気軽にお問い合わせください。
            </p>
            <a href="mailto:contact@shoken-webapp.example.com" className="text-decoration-none">
              contact@shoken-webapp.example.com
            </a>
          </div>
        </div>

        <hr className="my-3" />

        <div className="text-center text-muted">
          <small>&copy; {currentYear} 証券Webapp All Rights Reserved.</small>
        </div>
      </div>
    </footer>
  );
}
