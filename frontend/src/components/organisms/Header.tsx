import { useState, useEffect, useRef } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Button } from '../atoms/Button';
import { useAuth } from '../../features/auth/hooks/useAuth';
import { cn } from '../../lib/utils/classNames';
import { APP_SHELL_CONTAINER } from '@/lib/layout';

// ナビゲーションリンクの設定
const NAV_LINKS = [
  { to: '/search', label: '銘柄検索' },
  { to: '/assetbalance', label: '資産管理' },
  { to: '/receipts', label: '取引明細' },
] as const;

// ユーザー名からイニシャルを取得
const getInitials = (name: string | null | undefined, email: string | null | undefined): string => {
  if (name) {
    const parts = name.trim().split(/\s+/);
    if (parts.length >= 2) {
      return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
    }
    return name.slice(0, 2).toUpperCase();
  }
  if (email) {
    return email.slice(0, 2).toUpperCase();
  }
  return 'U';
};

export function Header() {
  const { user, login, logout, deleteAccount, isAuthenticated, isLoading } = useAuth();
  const [showDropdown, setShowDropdown] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [imageError, setImageError] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const location = useLocation();

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
      <header className="bg-slate-800 text-white no-print">
        <div className={`${APP_SHELL_CONTAINER} py-3`}>
          <div className="flex flex-col gap-2 md:flex-row md:items-center">
            <div className="flex items-center justify-between">
              <Link className="text-2xl font-bold tracking-wide text-white hover:text-slate-200 transition-colors" to="/">証券Web</Link>
            </div>

            <nav className="flex flex-wrap items-center gap-1" aria-label="主要ナビゲーション">
              {NAV_LINKS.map(({ to, label }) => {
                const isActive = location.pathname === to || location.pathname.startsWith(to + '/');
                return (
                  <Link
                    key={to}
                    to={to}
                    className={cn(
                      'px-4 py-2 text-base font-medium transition-colors border-b-2',
                      isActive
                        ? 'border-white text-white font-semibold'
                        : 'border-transparent text-slate-300 hover:text-white hover:border-slate-500'
                    )}
                    aria-current={isActive ? 'page' : undefined}
                  >
                    {label}
                  </Link>
                );
              })}
            </nav>

            <div className="flex items-center gap-3 md:ml-auto">
              {isLoading ? (
                <span className="text-white/80 text-base">読み込み中...</span>
              ) : isAuthenticated && user ? (
                <div className="flex items-center gap-3">
                  {/* ユーザーアバター（フォールバック付き） */}
                  {user.picture_url && !imageError ? (
                    <img
                      src={user.picture_url}
                      alt={user.name || 'ユーザー'}
                      className="h-8 w-8 rounded-full object-cover bg-slate-600"
                      onError={() => setImageError(true)}
                    />
                  ) : (
                    <div
                      className="h-8 w-8 rounded-full bg-slate-600 flex items-center justify-center text-white text-sm font-medium"
                      aria-label={user.name || 'ユーザー'}
                    >
                      {getInitials(user.name, user.email)}
                    </div>
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
          onKeyDown={(e) => { if (e.key === 'Escape') setShowDeleteConfirm(false); }}
          role="dialog"
          aria-modal="true"
          aria-labelledby="delete-modal-title"
          aria-describedby="delete-modal-description"
          tabIndex={-1}
        >
          <div
            className="w-full max-w-md"
            role="presentation"
            onClick={(e) => e.stopPropagation()}
            onKeyDown={(e) => { if (e.key === 'Escape') setShowDeleteConfirm(false); else e.stopPropagation(); }}
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
                  資産管理、配当金、取引履歴などすべてのデータが削除されます。
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
