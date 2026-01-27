import { useState, useEffect, useRef } from 'react';
import { Link } from 'react-router-dom';
import { Button } from '../atoms/Button';
import { useAuth } from '../../features/auth/hooks/useAuth';

export function Header() {
  const { user, login, logout, deleteAccount, isAuthenticated, isLoading } = useAuth();
  const [showDropdown, setShowDropdown] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // ドロップダウン外クリックで閉じる
  useEffect(() => {
    if (!showDropdown) return;

    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setShowDropdown(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [showDropdown]);

  // キーボード操作対応（Escapeで閉じる）
  useEffect(() => {
    if (!showDropdown) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setShowDropdown(false);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [showDropdown]);

  const handleLogout = async () => {
    setShowDropdown(false);
    try {
      await logout();
    } catch (error) {
      console.error('ログアウトエラー:', error);
    }
  };

  const handleDeleteAccount = async () => {
    try {
      await deleteAccount();
      setShowDeleteConfirm(false);
      setShowDropdown(false);
    } catch (error) {
      console.error('アカウント削除エラー:', error);
    }
  };

  return (
    <>
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
                <div className="d-flex align-items-center position-relative">
                  {user.picture_url && (
                    <img
                      src={user.picture_url}
                      alt={user.name || 'ユーザー'}
                      className="rounded-circle me-2"
                      style={{ width: '32px', height: '32px' }}
                    />
                  )}
                  <span className="text-light me-3">{user.name || user.email}</span>
                  <div className="dropdown" ref={dropdownRef}>
                    <Button
                      variant="outline-light"
                      size="sm"
                      onClick={() => setShowDropdown(!showDropdown)}
                      aria-haspopup="menu"
                      aria-expanded={showDropdown}
                      aria-controls="user-menu"
                    >
                      メニュー
                    </Button>
                    {showDropdown && (
                      <ul
                        id="user-menu"
                        className="dropdown-menu dropdown-menu-end show"
                        style={{ position: 'absolute', right: 0, top: '100%' }}
                        role="menu"
                        aria-label="ユーザーメニュー"
                      >
                        <li role="none">
                          <button
                            className="dropdown-item"
                            onClick={handleLogout}
                            role="menuitem"
                          >
                            ログアウト
                          </button>
                        </li>
                        <li role="none"><hr className="dropdown-divider" /></li>
                        <li role="none" className="dropdown-header small text-muted">
                          危険な操作
                        </li>
                        <li role="none">
                          <button
                            className="dropdown-item text-danger"
                            onClick={() => setShowDeleteConfirm(true)}
                            role="menuitem"
                            aria-describedby="delete-warning"
                          >
                            <span id="delete-warning" className="visually-hidden">
                              警告: この操作は取り消せません
                            </span>
                            アカウント削除
                          </button>
                        </li>
                      </ul>
                    )}
                  </div>
                </div>
              ) : (
                <Button variant="outline-light" size="sm" onClick={() => login()}>
                  ログイン
                </Button>
              )}
            </div>
          </div>
        </div>
      </header>

      {/* アカウント削除確認モーダル */}
      {showDeleteConfirm && (
        <div
          className="modal show d-block"
          style={{ backgroundColor: 'rgba(0,0,0,0.5)' }}
          onClick={() => setShowDeleteConfirm(false)}
          role="dialog"
          aria-modal="true"
          aria-labelledby="delete-modal-title"
          aria-describedby="delete-modal-description"
        >
          <div
            className="modal-dialog modal-dialog-centered"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-content">
              <div className="modal-header">
                <h5 id="delete-modal-title" className="modal-title text-danger">
                  アカウント削除の確認
                </h5>
                <button
                  type="button"
                  className="btn-close"
                  onClick={() => setShowDeleteConfirm(false)}
                  aria-label="閉じる"
                />
              </div>
              <div id="delete-modal-description" className="modal-body">
                <p>本当にアカウントを削除しますか？</p>
                <p className="text-danger mb-0">
                  <strong>警告:</strong> この操作は取り消せません。
                  保有銘柄、配当金、取引履歴などすべてのデータが削除されます。
                </p>
              </div>
              <div className="modal-footer">
                <Button
                  variant="secondary"
                  onClick={() => setShowDeleteConfirm(false)}
                >
                  キャンセル
                </Button>
                <Button variant="outline-danger" onClick={handleDeleteAccount}>
                  削除する
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
