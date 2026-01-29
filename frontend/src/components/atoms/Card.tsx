import { HTMLAttributes, ReactNode } from 'react';

export type CardVariant = 'default' | 'primary';

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

export interface CardHeaderProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
  variant?: CardVariant;
}

export interface CardBodyProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

export interface CardFooterProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

export function Card({ children, className = '', ...rest }: CardProps) {
  return (
    <div
      className={`rounded-lg border border-border bg-white shadow-sm print:border-black print:shadow-none ${className}`.trim()}
      {...rest}
    >
      {children}
    </div>
  );
}

export function CardHeader({ children, variant = 'default', className = '', ...rest }: CardHeaderProps) {
  const variantClass = variant === 'primary'
    ? 'bg-primary text-white'
    : 'border-b border-border bg-white text-dark';

  return (
    <div
      className={`px-4 py-2 ${variantClass} ${className}`.trim()}
      {...rest}
    >
      {children}
    </div>
  );
}

export function CardBody({ children, className = '', ...rest }: CardBodyProps) {
  return (
    <div className={`${className}`.trim()} {...rest}>
      {children}
    </div>
  );
}

export function CardFooter({ children, className = '', ...rest }: CardFooterProps) {
  return (
    <div
      className={`border-t border-border px-4 py-3 ${className}`.trim()}
      {...rest}
    >
      {children}
    </div>
  );
}
