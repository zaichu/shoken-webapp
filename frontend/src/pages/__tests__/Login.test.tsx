import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useAuth } from '@/features/auth/hooks/useAuth';
import { LoginPage } from '../Login';

const { mockNavigate } = vi.hoisted(() => ({
  mockNavigate: vi.fn(),
}));

vi.mock('react-router-dom', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router-dom')>();
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

vi.mock('@/features/auth/hooks/useAuth', () => ({
  useAuth: vi.fn(),
}));

vi.mock('@/components/templates/Layout', () => ({
  Layout: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

type AuthState = ReturnType<typeof useAuth>;

function createAuthState(overrides: Partial<AuthState> = {}): AuthState {
  return {
    user: null,
    setUser: vi.fn(),
    login: vi.fn(),
    logout: vi.fn().mockResolvedValue(undefined),
    deleteAccount: vi.fn().mockResolvedValue(undefined),
    isAuthenticated: false,
    isLoading: false,
    onLogout: vi.fn(() => vi.fn()),
    ...overrides,
  };
}

function renderLoginPage(authOverrides: Partial<AuthState> = {}) {
  const authState = createAuthState(authOverrides);
  vi.mocked(useAuth).mockReturnValue(authState);

  render(
    <MemoryRouter>
      <LoginPage />
    </MemoryRouter>
  );

  return authState;
}

describe('LoginPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockNavigate.mockReset();
    vi.mocked(useAuth).mockReturnValue(createAuthState());
  });

  it('ロード中はSpinnerを表示する', () => {
    renderLoginPage({ isLoading: true });

    expect(screen.getByRole('status', { name: '読み込み中...' })).toBeInTheDocument();
  });

  it('ロード完了後の未ログイン状態ではGoogleでログインボタンを表示する', () => {
    renderLoginPage();

    expect(screen.getByRole('button', { name: /Googleでログイン/ })).toBeInTheDocument();
  });

  it('Googleでログインボタンのクリックでloginを呼ぶ', async () => {
    const login = vi.fn();
    const user = userEvent.setup();
    renderLoginPage({ login });

    await user.click(screen.getByRole('button', { name: /Googleでログイン/ }));

    expect(login).toHaveBeenCalledTimes(1);
  });

  it('認証済みの場合はホームへリダイレクトする', async () => {
    renderLoginPage({
      user: {
        id: 'user-1',
        email: 'test@example.com',
      },
      isAuthenticated: true,
      isLoading: false,
    });

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/');
    });
  });
});
