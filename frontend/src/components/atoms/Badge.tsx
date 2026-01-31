import { HTMLAttributes, ReactNode } from 'react';
import { cn } from '../../lib/utils/classNames';

export type BadgeVariant = 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info' | 'light' | 'dark';
export type BadgeTone = 'solid' | 'soft' | 'outline';

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  children: ReactNode;
  variant?: BadgeVariant;
  tone?: BadgeTone;
}

const solidStyles: Record<BadgeVariant, string> = {
  primary: 'bg-primary text-white',
  secondary: 'bg-secondary text-white',
  success: 'bg-success text-white',
  warning: 'bg-warning text-dark',
  danger: 'bg-danger text-white',
  info: 'bg-info text-dark',
  light: 'bg-light text-dark',
  dark: 'bg-dark text-white',
};

const softStyles: Record<BadgeVariant, string> = {
  primary: 'bg-primary/15 text-primary',
  secondary: 'bg-secondary/15 text-secondary',
  success: 'bg-success/15 text-success',
  warning: 'bg-warning/20 text-warning',
  danger: 'bg-danger/15 text-danger',
  info: 'bg-info/15 text-info',
  light: 'bg-light text-dark',
  dark: 'bg-dark/15 text-dark',
};

const outlineStyles: Record<BadgeVariant, string> = {
  primary: 'border border-primary text-primary',
  secondary: 'border border-secondary text-secondary',
  success: 'border border-success text-success',
  warning: 'border border-warning text-warning',
  danger: 'border border-danger text-danger',
  info: 'border border-info text-info',
  light: 'border border-light text-light',
  dark: 'border border-dark text-dark',
};

export function Badge({ children, variant = 'primary', tone = 'soft', className, ...rest }: BadgeProps) {
  const toneClass = tone === 'solid'
    ? solidStyles[variant]
    : tone === 'outline'
      ? outlineStyles[variant]
      : softStyles[variant];

  return (
    <span
      className={cn('inline-flex items-center rounded-md px-2.5 py-0.5 text-xs font-semibold', toneClass, className)}
      {...rest}
    >
      {children}
    </span>
  );
}
