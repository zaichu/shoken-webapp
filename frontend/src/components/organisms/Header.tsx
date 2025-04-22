import { Link } from 'react-router-dom';
// import { Button } from '../atoms/Button';
// import { useAuth } from '../../features/auth/context/AuthContext';

interface HeaderProps {
  onToggleTheme?: () => void;
}

export function Header({ onToggleTheme }: HeaderProps) {
  // const { user, logout } = useAuth();

  return (
    <header className="navbar navbar-expand-lg navbar-dark bg-primary">
      <div className="container" style={{ maxWidth: '1600px' }}>
        <Link className="navbar-brand" to="/shoken-webapp/">証券Web</Link>

        <div className="collapse navbar-collapse" id="navbarNav">
          <ul className="navbar-nav">
            <li className="nav-item">
              <Link className="nav-link" to="/shoken-webapp/search">銘柄検索</Link>
            </li>
            <li className="nav-item">
              <Link className="nav-link" to="/shoken-webapp/receipts">受取金</Link>
            </li>
          </ul>

          {/* <div className="d-flex align-items-center">
            {onToggleTheme && (
              <Button
                variant="outline-light"
                size="sm"
                onClick={onToggleTheme}
                className="me-3"
              >
                <span role="img" aria-label="テーマ切替">🌓</span>
              </Button>
            )}

            {user && user.authCode ? (
              <div className="d-flex align-items-center">
                <span className="text-light me-3">{user.name || 'ユーザー'}</span>
                <Button variant="outline-light" size="sm" onClick={logout}>
                  ログアウト
                </Button>
              </div>
            ) : (
              <Link to="/login">
                <Button variant="outline-light" size="sm">
                  ログイン
                </Button>
              </Link>
            )}
          </div> */}
        </div>
      </div>
    </header>
  );
}
