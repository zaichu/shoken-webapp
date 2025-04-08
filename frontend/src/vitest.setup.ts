import '@testing-library/jest-dom';
import { vi } from 'vitest';

// グローバルモック
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    // useNavigateのモック
    useNavigate: () => vi.fn(),
    // useLocationのモック
    useLocation: () => ({
      pathname: '/test-path',
      search: '',
      hash: '',
      state: null,
      key: 'default',
    }),
  };
});

// 環境変数のモック
vi.stubGlobal('import.meta', {
  env: {
    VITE_SHOKEN_WEBAPI_API_URL: 'http://localhost:3000/api',
  },
});

// マッチャーの型拡張
declare global {
  namespace Vi {
    interface JestAssertion {
      toBeInTheDocument(): void;
      toHaveTextContent(text: string): void;
      toBeDisabled(): void;
      toBeEnabled(): void;
    }
  }
}
