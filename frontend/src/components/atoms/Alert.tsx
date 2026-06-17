import { HTMLAttributes, ReactNode } from 'react';
import { cn } from '../../lib/utils/classNames';

type AlertVariant = 'info' | 'warning' | 'danger' | 'success';

interface AlertProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
  variant?: AlertVariant;
}

const variantClasses: Record<AlertVariant, string> = {
  info: 'border-blue-200 bg-blue-50 text-blue-800',
  warning: 'border-amber-200 bg-amber-50 text-amber-900',
  danger: 'border-red-200 bg-red-50 text-red-700',
  success: 'border-teal-200 bg-teal-50 text-teal-800',
};

export function Alert({ children, variant = 'info', className, ...rest }: AlertProps) {
  return (
    <div
      className={cn('rounded-lg border px-4 py-3 text-sm font-medium shadow-sm', variantClasses[variant], className)}
      role="alert"
      {...rest}
    >
      {children}
    </div>
  );
}
