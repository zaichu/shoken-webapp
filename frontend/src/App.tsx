import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { UserInfoProvider } from './context/UserContext';
import { ErrorBoundary } from './components/molecules/ErrorBoundary';
import { ErrorPage } from './components/templates/ErrorPage';
import { Home } from './pages/Home';
import { Search } from './pages/Search';
import { Receipts } from './pages/Receipts';
import { NotFound } from './pages/NotFound';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5分間はデータを新鮮とみなす
      retry: 1,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <UserInfoProvider>
        <ErrorBoundary fallback={<ErrorPage />}>
          <BrowserRouter>
            <Routes>
              <Route path="/shoken-webapp/" element={<Home />} />
              <Route path="/shoken-webapp/search" element={<Search />} />
              <Route path="/shoken-webapp/receipts" element={<Receipts />} />
              <Route path="/shoken-webapp/404" element={<NotFound />} />
              <Route path="*" element={<Navigate to="/shoken-webapp/404" replace />} />
            </Routes>
          </BrowserRouter>
        </ErrorBoundary>
      </UserInfoProvider>
    </QueryClientProvider>
  );
}

export default App;
