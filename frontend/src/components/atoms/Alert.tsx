import { HTMLAttributes, ReactNode } from 'react';
import { cn } from '../../lib/utils/classNames';

export type AlertVariant = 'info' | 'warning' | 'danger' | 'success';

export interface AlertProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
  variant?: AlertVariant;
}

const variantClasses: Record<AlertVariant, string> = {
  info: 'bg-info/10 text-info border-info/30',
  warning: 'bg-warning/15 text-warning border-warning/40',
  danger: 'bg-danger/10 text-danger border-danger/30',
  success: 'bg-success/10 text-success border-success/30',
};

export function Alert({ children, variant = 'info', className, ...rest }: AlertProps) {
  return (
    <div
      className={cn('rounded-md border px-4 py-3 text-sm', variantClasses[variant], className)}
      role="alert"
      {...rest}
    >
      {children}
    </div>
  );
}
