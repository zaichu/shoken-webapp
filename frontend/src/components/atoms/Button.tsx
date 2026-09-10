import { ButtonHTMLAttributes, ReactNode, forwardRef } from 'react';
import { Spinner } from './Spinner';

type ButtonVariant =
  | 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'info' | 'light' | 'dark'
  | 'outline-primary' | 'outline-secondary' | 'outline-success' | 'outline-danger'
  | 'outline-warning' | 'outline-info' | 'outline-light' | 'outline-dark';

type ButtonSize = 'sm' | 'md' | 'lg';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: ReactNode;
  variant?: ButtonVariant;
  size?: ButtonSize;
  fullWidth?: boolean;
  loading?: boolean;
  icon?: ReactNode;
  iconPosition?: 'left' | 'right';
}

const variantStyles: Record<ButtonVariant, string> = {
  primary: 'border border-slate-950 bg-slate-950 text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.12)] hover:bg-slate-800 active:bg-slate-950',
  secondary: 'border border-slate-300 bg-white text-slate-800 hover:border-slate-500 hover:bg-slate-50',
  success: 'border border-teal-700 bg-teal-700 text-white hover:bg-teal-800',
  danger: 'border border-danger bg-danger text-white hover:bg-danger-hover',
  warning: 'border border-amber-500 bg-amber-500 text-slate-950 hover:bg-amber-600',
  info: 'border border-blue-600 bg-blue-600 text-white hover:bg-blue-700',
  light: 'border border-slate-200 bg-white text-slate-900 hover:bg-slate-100',
  dark: 'border border-slate-950 bg-slate-950 text-white hover:bg-slate-800',
  'outline-primary': 'border border-slate-950 text-slate-950 hover:bg-slate-950 hover:text-white',
  'outline-secondary': 'border border-slate-300 text-slate-700 hover:border-slate-500 hover:bg-slate-50',
  'outline-success': 'border border-teal-700 text-teal-700 hover:bg-teal-700 hover:text-white',
  'outline-danger': 'border border-danger text-danger hover:bg-danger hover:text-white',
  'outline-warning': 'border border-amber-600 text-amber-700 hover:bg-amber-500 hover:text-slate-950',
  'outline-info': 'border border-blue-600 text-blue-700 hover:bg-blue-600 hover:text-white',
  'outline-light': 'border border-white/70 text-white hover:bg-white hover:text-slate-950',
  'outline-dark': 'border border-slate-950 text-slate-950 hover:bg-slate-950 hover:text-white',
};

const sizeStyles: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-sm max-sm:min-h-[44px]',
  md: 'px-4 py-2 text-sm max-sm:min-h-[44px]',
  lg: 'px-5 py-2.5 text-base',
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
    const baseClasses = 'inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 no-print';
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

    return (
      <button
        ref={ref}
        className={combinedClasses}
        disabled={isDisabled}
        data-loading={loading ? 'true' : undefined}
        {...rest}
      >
        {loading ? (
          <>
            <Spinner size="sm" className="mr-2" />
            読み込み中...
          </>
        ) : (
          <>
            {icon && iconPosition === 'left' && (
              <span className="inline-flex items-center mr-2">{icon}</span>
            )}
            {children}
            {icon && iconPosition === 'right' && (
              <span className="inline-flex items-center ml-2">{icon}</span>
            )}
          </>
        )}
      </button>
    );
  }
);

Button.displayName = 'Button';

export { Button };
