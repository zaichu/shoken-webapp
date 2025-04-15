import { ButtonHTMLAttributes, ReactNode } from 'react';

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
  className?: string;
}

export function Button({
  children,
  variant = 'primary',
  size = 'md',
  fullWidth = false,
  className = '',
  ...rest
}: ButtonProps) {
  const baseClasses = 'btn';
  const variantClass = `btn-${variant}`;
  const sizeClass = size !== 'md' ? `btn-${size}` : '';
  const widthClass = fullWidth ? 'w-100' : '';

  const combinedClasses = [
    baseClasses,
    variantClass,
    sizeClass,
    widthClass,
    className
  ].filter(Boolean).join(' ');

  return (
    <button className={combinedClasses} {...rest}>
      {children}
    </button>
  );
}
