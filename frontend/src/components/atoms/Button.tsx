import { ButtonHTMLAttributes, ReactNode, forwardRef } from 'react';
import { Spinner } from './Spinner';

export type ButtonVariant =
  | 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'info' | 'light' | 'dark'
  | 'outline-primary' | 'outline-secondary' | 'outline-success' | 'outline-danger'
  | 'outline-warning' | 'outline-info' | 'outline-light' | 'outline-dark';

export type ButtonSize = 'sm' | 'md' | 'lg';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: ReactNode;
  variant?: ButtonVariant;
  size?: ButtonSize;
  fullWidth?: boolean;
  loading?: boolean;
  icon?: ReactNode;
  iconPosition?: 'left' | 'right';
}

// バリアント別のスタイル定義
const variantStyles: Record<ButtonVariant, string> = {
  primary: 'bg-primary text-white hover:bg-primary-hover active:bg-primary-dark',
  secondary: 'bg-secondary text-white hover:bg-secondary-hover',
  success: 'bg-success text-white hover:bg-success-hover',
  danger: 'bg-danger text-white hover:bg-danger-hover',
  warning: 'bg-warning text-dark hover:bg-warning-hover',
  info: 'bg-info text-dark hover:bg-info-hover',
  light: 'bg-light text-dark hover:bg-gray-200',
  dark: 'bg-dark text-white hover:bg-gray-800',
  'outline-primary': 'border border-primary text-primary hover:bg-primary hover:text-white',
  'outline-secondary': 'border border-secondary text-secondary hover:bg-secondary hover:text-white',
  'outline-success': 'border border-success text-success hover:bg-success hover:text-white',
  'outline-danger': 'border border-danger text-danger hover:bg-danger hover:text-white',
  'outline-warning': 'border border-amber-600 text-amber-700 hover:bg-warning hover:text-dark',
  'outline-info': 'border border-info text-info hover:bg-info hover:text-dark',
  'outline-light': 'border border-light text-light hover:bg-light hover:text-dark',
  'outline-dark': 'border border-dark text-dark hover:bg-dark hover:text-white',
};

// サイズ別のスタイル定義
const sizeStyles: Record<ButtonSize, string> = {
  sm: 'px-2 py-1 text-sm',
  md: 'px-3 py-1.5 text-base',
  lg: 'px-4 py-2 text-lg',
};

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      children,
      variant = 'primary',
      size = 'md',
      fullWidth = false,
      loading = false,
      icon,
      iconPosition = 'left',
      disabled,
      className = '',
      ...rest
    },
    ref
  ) => {
    const baseClasses = 'inline-flex items-center justify-center font-medium rounded transition-colors focus:outline-none focus:ring-2 focus:ring-offset-1 focus:ring-primary/50 disabled:opacity-65 disabled:cursor-not-allowed no-print';
    const variantClass = variantStyles[variant];
    const sizeClass = sizeStyles[size];
    const widthClass = fullWidth ? 'w-full' : '';
    const loadingClass = loading ? 'relative' : '';

    const combinedClasses = [
      baseClasses,
      variantClass,
      sizeClass,
      widthClass,
      loadingClass,
      className
    ].filter(Boolean).join(' ');

    const isDisabled = disabled || loading;

    const renderContent = () => {
      if (loading) {
        return (
          <>
            <Spinner size="sm" className="mr-2" />
            読み込み中...
          </>
        );
      }

      const iconElement = icon && (
        <span className={`inline-flex items-center ${iconPosition === 'right' ? 'ml-2' : 'mr-2'}`}>
          {icon}
        </span>
      );

      return (
        <>
          {icon && iconPosition === 'left' && iconElement}
          {children}
          {icon && iconPosition === 'right' && iconElement}
        </>
      );
    };

    return (
      <button
        ref={ref}
        className={combinedClasses}
        disabled={isDisabled}
        data-loading={loading ? 'true' : undefined}
        {...rest}
      >
        {renderContent()}
      </button>
    );
  }
);

Button.displayName = 'Button';

export { Button };
