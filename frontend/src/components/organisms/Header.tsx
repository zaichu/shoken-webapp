import { Link } from 'react-router-dom';
import { Button } from '../atoms/Button';
import { useAuth } from '../../features/auth/hooks/useAuth';

export function Header() {
  const { user, logout, isAuthenticated, isLoading } = useAuth();

  const handleLogout = async () => {
    try {
      await logout();
    } catch (error) {
      console.error('ログアウトエラー:', error);
    }
  };

  return (
    <header className="navbar navbar-expand-lg navbar-dark bg-primary">
      <div className="container" style={{ maxWidth: '1600px' }}>
        <Link className="navbar-brand" to="/">証券Web</Link>

        <div className="collapse navbar-collapse" id="navbarNav">
          <ul className="navbar-nav">
            <li className="nav-item">
              <Link className="nav-link" to="/search">銘柄検索</Link>
            </li>
            <li className="nav-item">
              <Link className="nav-link" to="/assetbalance">保有銘柄</Link>
            </li>
            <li className="nav-item">
              <Link className="nav-link" to="/receipts">受取金</Link>
            </li>
          </ul>

          <div className="d-flex align-items-center ms-auto">
            {isLoading ? (
              <span className="text-light">読み込み中...</span>
            ) : isAuthenticated && user ? (
              <div className="d-flex align-items-center">
                {user.picture_url && (
                  <img
                    src={user.picture_url}
                    alt={user.name || 'ユーザー'}
                    className="rounded-circle me-2"
                    style={{ width: '32px', height: '32px' }}
                  />
                )}
                <span className="text-light me-3">{user.name || user.email}</span>
                <Button variant="outline-light" size="sm" onClick={handleLogout}>
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
          </div>
        </div>
      </div>
    </header>
  );
}
