import { HTMLAttributes, ReactNode } from 'react';
import { cn } from '../../lib/utils/classNames';

type CardVariant = 'default' | 'primary' | 'secondary';

interface CardProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

interface CardHeaderProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
  variant?: CardVariant;
}

interface CardBodyProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}


export function Card({ children, className, ...rest }: CardProps) {
  return (
    <div
      className={cn(
        'rounded-lg border border-border bg-white shadow-sm print:border-black print:shadow-none',
        className
      )}
      {...rest}
    >
      {children}
    </div>
  );
}

export function CardHeader({ children, variant = 'default', className, ...rest }: CardHeaderProps) {
  const variantClasses: Record<CardVariant, string> = {
    primary: 'bg-slate-700 text-white',
    secondary: 'bg-slate-500 text-white',
    default: 'border-b border-border bg-white text-dark',
  };

  return (
    <div
      className={cn('px-4 py-2', variantClasses[variant], className)}
      {...rest}
    >
      {children}
    </div>
  );
}

export function CardBody({ children, className, ...rest }: CardBodyProps) {
  return (
    <div className={cn(className)} {...rest}>
      {children}
    </div>
  );
}

