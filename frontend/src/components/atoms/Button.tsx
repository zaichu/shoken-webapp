import React, { ButtonHTMLAttributes, ReactNode, forwardRef } from 'react';

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
    const baseClasses = 'btn';
    const variantClass = `btn-${variant}`;
    const sizeClass = size !== 'md' ? `btn-${size}` : '';
    const widthClass = fullWidth ? 'w-100' : '';
    const loadingClass = loading ? 'btn-loading' : '';

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
            <span className="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true" />
            読み込み中...
          </>
        );
      }

      const iconElement = icon && (
        <span className={`btn-icon ${iconPosition === 'right' ? 'ms-2' : 'me-2'}`}>
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
        {...rest}
      >
        {renderContent()}
      </button>
    );
  }
);

Button.displayName = 'Button';

export { Button };
