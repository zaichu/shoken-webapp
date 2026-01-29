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
    } catch {
      // ログアウト失敗時は何もしない（useAuthで処理）
    }
  };

  const handleDeleteAccount = async () => {
    try {
      await deleteAccount();
      setShowDeleteConfirm(false);
      setShowDropdown(false);
    } catch {
      // 削除失敗時はモーダルを閉じない
    }
  };

  return (
    <>
      <header className="bg-primary text-white no-print">
        <div className="mx-auto w-full max-w-[1600px] px-4 py-3">
          <div className="flex flex-col gap-3 md:flex-row md:items-center">
            <div className="flex items-center justify-between">
              <Link className="text-lg font-semibold tracking-wide text-white" to="/">証券Web</Link>
            </div>

            <nav className="flex flex-wrap items-center gap-4 text-base md:text-lg" aria-label="主要ナビゲーション">
              <Link className="text-white/90 hover:text-white transition-colors" to="/search">銘柄検索</Link>
              <Link className="text-white/90 hover:text-white transition-colors" to="/assetbalance">保有銘柄</Link>
              <Link className="text-white/90 hover:text-white transition-colors" to="/receipts">受取金</Link>
            </nav>

            <div className="flex items-center gap-3 md:ml-auto">
              {isLoading ? (
                <span className="text-white/80 text-base">読み込み中...</span>
              ) : isAuthenticated && user ? (
                <div className="flex items-center gap-3">
                  {user.picture_url && (
                    <img
                      src={user.picture_url}
                      alt={user.name || 'ユーザー'}
                      className="h-8 w-8 rounded-full object-cover"
                    />
                  )}
                  <span className="text-base text-white/90">{user.name || user.email}</span>
                  <div className="relative" ref={dropdownRef}>
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
                        className="absolute right-0 mt-2 w-56 rounded-md border border-border bg-white text-dark shadow-lg"
                        role="menu"
                        aria-label="ユーザーメニュー"
                      >
                        <li role="none">
                          <button
                            className="w-full px-3 py-2 text-left text-base hover:bg-gray-100"
                            onClick={handleLogout}
                            role="menuitem"
                          >
                            ログアウト
                          </button>
                        </li>
                        <li role="none"><hr className="my-1 border-border" /></li>
                        <li role="none" className="px-3 py-1 text-xs text-secondary">
                          危険な操作
                        </li>
                        <li role="none">
                          <button
                            className="w-full px-3 py-2 text-left text-base text-danger hover:bg-danger/10"
                            onClick={() => setShowDeleteConfirm(true)}
                            role="menuitem"
                            aria-describedby="delete-warning"
                          >
                            <span id="delete-warning" className="sr-only">
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
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
          onClick={() => setShowDeleteConfirm(false)}
          role="dialog"
          aria-modal="true"
          aria-labelledby="delete-modal-title"
          aria-describedby="delete-modal-description"
        >
          <div
            className="w-full max-w-md"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="rounded-lg bg-white shadow-lg">
              <div className="flex items-center justify-between border-b border-border px-4 py-3">
                <h5 id="delete-modal-title" className="text-danger font-semibold">
                  アカウント削除の確認
                </h5>
                <button
                  type="button"
                  className="text-gray-500 hover:text-gray-700"
                  onClick={() => setShowDeleteConfirm(false)}
                  aria-label="閉じる"
                >
                  <span aria-hidden="true">✕</span>
                </button>
              </div>
              <div id="delete-modal-description" className="px-4 py-4 text-base text-dark">
                <p>本当にアカウントを削除しますか？</p>
                <p className="mt-2 text-danger">
                  <strong>警告:</strong> この操作は取り消せません。
                  保有銘柄、配当金、取引履歴などすべてのデータが削除されます。
                </p>
              </div>
              <div className="flex justify-end gap-2 border-t border-border px-4 py-3">
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
