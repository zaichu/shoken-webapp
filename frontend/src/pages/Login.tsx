import { Layout } from '../components/templates/Layout';
import { useAuth } from '../features/auth/hooks/useAuth';
import { useNavigate } from 'react-router-dom';
import { useEffect } from 'react';
import { Card, CardBody } from '../components/atoms/Card';
import { Spinner } from '../components/atoms/Spinner';

export function LoginPage() {
  const { login, isAuthenticated, isLoading } = useAuth();
  const navigate = useNavigate();

  // すでにログイン済みの場合はホームにリダイレクト
  useEffect(() => {
    if (isAuthenticated && !isLoading) {
      navigate('/');
    }
  }, [isAuthenticated, isLoading, navigate]);

  const handleGoogleLogin = async () => {
    await login();
  };

  if (isLoading) {
    return (
      <Layout>
        <div className="flex items-center justify-center py-12">
          <Spinner size="lg" className="text-primary" />
        </div>
      </Layout>
    );
  }

  return (
    <Layout>
      <div className="mx-auto flex max-w-md flex-col items-center py-12">
        <Card className="w-full shadow-md">
          <CardBody className="space-y-6 p-6 text-center">
            <h2 className="text-xl font-semibold">ログイン</h2>

            <p className="text-sm text-secondary">
              Googleアカウントでログインしてください
            </p>

            <button
              type="button"
              className="inline-flex w-full items-center justify-center gap-2 rounded-md border border-dark px-4 py-2 text-sm font-semibold text-dark transition-colors hover:bg-dark hover:text-white"
              onClick={handleGoogleLogin}
            >
              <img
                src="https://www.gstatic.com/firebasejs/ui/2.0.0/images/auth/google.svg"
                alt="Google"
                width="20"
                height="20"
              />
              Googleでログイン
            </button>
          </CardBody>
        </Card>
      </div>
    </Layout>
  );
}
