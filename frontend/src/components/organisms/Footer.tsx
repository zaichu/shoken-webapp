import { Link } from 'react-router-dom';

export function Footer() {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="mt-10 bg-light py-6 text-dark">
      <div className="mx-auto w-full max-w-[1600px] px-4">
        <div className="grid gap-6 md:grid-cols-3">
          <div>
            <h5 className="text-base font-semibold">証券Webapp</h5>
            <p className="mt-2 text-sm text-secondary">
              投資情報と取引明細管理のためのプラットフォーム
            </p>
          </div>

          <div>
            <h5 className="text-base font-semibold">リンク</h5>
            <ul className="mt-2 space-y-2 text-sm">
              <li>
                <Link className="text-primary hover:underline" to="/">ホーム</Link>
              </li>
              <li>
                <Link className="text-primary hover:underline" to="/search">銘柄検索</Link>
              </li>
              <li>
                <Link className="text-primary hover:underline" to="/receipts">取引明細管理</Link>
              </li>
            </ul>
          </div>

          <div>
            <h5 className="text-base font-semibold">お問い合わせ</h5>
            <p className="mt-2 text-sm text-secondary">
              ご質問やご意見がありましたら、お気軽にお問い合わせください。
            </p>
            <a href="mailto:contact@shoken-webapp.example.com" className="text-sm text-primary hover:underline">
              contact@shoken-webapp.example.com
            </a>
          </div>
        </div>

        <hr className="my-6 border-border" />

        <div className="text-center text-xs text-secondary">
          &copy; {currentYear} 証券Webapp All Rights Reserved.
        </div>
      </div>
    </footer>
  );
}
