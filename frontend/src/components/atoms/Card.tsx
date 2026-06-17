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
        'rounded-xl border border-slate-950/10 bg-white/90 shadow-[0_14px_38px_-32px_rgba(15,23,42,0.85)] backdrop-blur-sm print:border-black print:shadow-none',
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
    primary: 'border-b border-slate-950/10 bg-slate-950 text-white',
    secondary: 'border-b border-slate-950/10 bg-slate-900 text-white',
    default: 'border-b border-slate-950/10 bg-white/80 text-dark',
  };

  return (
    <div
      className={cn('px-4 py-3', variantClasses[variant], className)}
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
