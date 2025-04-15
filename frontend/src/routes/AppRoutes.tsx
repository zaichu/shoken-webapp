import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { ErrorBoundary } from '../components/molecules/ErrorBoundary';
import { ErrorPage } from '../components/templates/ErrorPage';
import { HomePage } from '../pages/Home';
import { SearchPage } from '../pages/Search';
import { ReceiptsPage } from '../pages/Receipts';
import { NotFoundPage } from '../pages/NotFound';

export function AppRoutes() {
  return (
    <ErrorBoundary fallback={<ErrorPage />}>
      <BrowserRouter>
        <Routes>
          <Route path="/shoken-webapp/" element={<HomePage />} />
          <Route path="/shoken-webapp/search" element={<SearchPage />} />
          <Route path="/shoken-webapp/receipts" element={<ReceiptsPage />} />
          <Route path="/shoken-webapp/404" element={<NotFoundPage />} />
          <Route path="*" element={<Navigate to="/shoken-webapp/404" replace />} />
        </Routes>
      </BrowserRouter>
    </ErrorBoundary>
  );
}
