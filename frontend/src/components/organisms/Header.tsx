import { useState, useEffect, useRef } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Button } from '../atoms/Button';
import { ConfirmDeleteModal } from '../molecules/ConfirmDeleteModal/ConfirmDeleteModal';
import { useAuth } from '../../features/auth/hooks/useAuth';
import { cn } from '../../lib/utils/classNames';
import { APP_SHELL_CONTAINER } from '@/lib/layout';

const NAV_LINKS = [
  { to: '/search', label: '銘柄検索' },
  { to: '/assetbalance', label: '資産管理' },
  { to: '/receipts', label: '取引明細' },
] as const;

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
      <header className="sticky top-0 z-40 border-b border-slate-950/10 bg-[#111827]/95 text-white shadow-[0_18px_44px_-34px_rgba(15,23,42,0.95)] backdrop-blur no-print">
        <div className={cn(APP_SHELL_CONTAINER, 'py-3')}>
          <div className="flex flex-col gap-3 lg:flex-row lg:items-center">
            <div className="flex items-center justify-between gap-4">
              <Link className="group inline-flex items-center gap-3 text-white transition-colors hover:text-amber-100" to="/">
                <span className="flex h-10 w-10 items-center justify-center rounded-md border border-white/15 bg-white text-sm font-black text-slate-950 shadow-[inset_0_-3px_0_rgba(192,132,3,0.35)]">
                  証
                </span>
                <span>
                  <span className="block text-[11px] font-bold uppercase tracking-[0.28em] text-amber-300/90">Portfolio Desk</span>
                  <span className="block text-xl font-black leading-tight tracking-normal">証券Web</span>
                </span>
              </Link>
            </div>

            <nav className="flex flex-wrap items-center gap-1 rounded-md border border-white/10 bg-white/5 p-1" aria-label="主要ナビゲーション">
              {NAV_LINKS.map(({ to, label }) => {
                const isActive = location.pathname === to || location.pathname.startsWith(to + '/');
                return (
                  <Link
                    key={to}
                    to={to}
                    className={cn(
                      'rounded px-3.5 py-2 text-sm font-bold transition-[background-color,color,box-shadow]',
                      isActive
                        ? 'bg-white text-slate-950 shadow-[inset_0_-2px_0_#f59e0b]'
                        : 'text-slate-300 hover:bg-white/10 hover:text-white'
                    )}
                    aria-current={isActive ? 'page' : undefined}
                  >
                    {label}
                  </Link>
                );
              })}
            </nav>

            <div className="flex items-center gap-3 lg:ml-auto">
              {isLoading ? (
                <span className="text-sm font-semibold text-white/70">読み込み中...</span>
              ) : isAuthenticated && user ? (
                <div className="flex flex-wrap items-center gap-3">
                  {user.picture_url && !imageError ? (
                    <img
                      src={user.picture_url}
                      alt={user.name || 'ユーザー'}
                      className="h-9 w-9 rounded-md border border-white/20 bg-slate-700 object-cover"
                      onError={() => setImageError(true)}
                    />
                  ) : (
                    <div
                      className="flex h-9 w-9 items-center justify-center rounded-md border border-white/20 bg-slate-700 text-sm font-black text-white"
                      aria-label={user.name || 'ユーザー'}
                    >
                      {getInitials(user.name, user.email)}
                    </div>
                  )}
                  <span className="max-w-[16rem] truncate text-sm font-semibold text-white/85">{user.name || user.email}</span>
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
                        className="absolute right-0 mt-2 w-56 overflow-hidden rounded-lg border border-slate-950/10 bg-white text-dark shadow-2xl"
                        role="menu"
                        aria-label="ユーザーメニュー"
                      >
                        <li role="none">
                          <button
                            className="w-full px-3 py-2 text-left text-sm font-semibold hover:bg-slate-100"
                            onClick={handleLogout}
                            role="menuitem"
                          >
                            ログアウト
                          </button>
                        </li>
                        <li role="none"><hr className="border-border" /></li>
                        <li role="none" className="px-3 py-2 text-xs font-bold uppercase tracking-[0.16em] text-secondary">
                          危険な操作
                        </li>
                        <li role="none">
                          <button
                            className="w-full px-3 py-2 text-left text-sm font-bold text-danger hover:bg-danger/10"
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

      <ConfirmDeleteModal
        isOpen={showDeleteConfirm}
        onConfirm={handleDeleteAccount}
        onCancel={() => setShowDeleteConfirm(false)}
        title="アカウント削除の確認"
        description="アカウントを削除すると、資産管理・配当金・取引履歴などすべてのデータが削除されます。"
        itemCount={1}
        confirmLabel="削除する"
      />
    </>
  );
}
