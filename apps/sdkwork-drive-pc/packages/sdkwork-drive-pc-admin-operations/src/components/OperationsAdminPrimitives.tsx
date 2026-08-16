import React, { type ComponentType, type ReactNode } from 'react';
import {
  ChevronLeft,
  ChevronRight,
  CircleAlert,
  CircleCheck,
  Inbox,
  LoaderCircle,
  X,
} from 'lucide-react';
import { ICON_BUTTON_CLASS, SECONDARY_BUTTON_CLASS } from '../utils/uiPrimitives';

type IconComponent = ComponentType<{
  'aria-hidden'?: boolean | 'true' | 'false';
  className?: string;
  size?: number | string;
  strokeWidth?: number | string;
}>;

interface OperationsPageHeaderProps {
  actions?: ReactNode;
  description: string;
  icon: IconComponent;
  meta?: ReactNode;
  title: string;
  toneClassName?: string;
}

export function OperationsPageHeader({
  actions,
  description,
  meta,
  title,
}: OperationsPageHeaderProps) {
  return (
    <div aria-label={title} className={actions || meta ? 'flex shrink-0 flex-wrap items-center justify-between gap-3 px-4 pt-4 sm:px-6 sm:pt-6' : 'sr-only'}>
      <span className="sr-only">{description}</span>
      {meta ? <div className="flex items-center gap-2">{meta}</div> : <span />}
      {actions ? <div className="flex shrink-0 flex-wrap items-center gap-2">{actions}</div> : null}
    </div>
  );
}

interface NoticeBannerProps {
  dismissLabel?: string;
  message: string;
  onDismiss?: () => void;
  type: 'error' | 'success';
}

export function NoticeBanner({ dismissLabel = 'Dismiss', message, onDismiss, type }: NoticeBannerProps) {
  const Icon = type === 'success' ? CircleCheck : CircleAlert;
  return (
    <div
      className={`flex items-start gap-3 rounded-lg border px-4 py-3 text-sm ${
        type === 'success'
          ? 'border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900/60 dark:bg-emerald-950/30 dark:text-emerald-200'
          : 'border-red-200 bg-red-50 text-red-700 dark:border-red-900/60 dark:bg-red-950/30 dark:text-red-200'
      }`}
      role={type === 'error' ? 'alert' : 'status'}
    >
      <Icon aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
      <span className="min-w-0 flex-1">{message}</span>
      {onDismiss ? (
        <button type="button" className={ICON_BUTTON_CLASS} aria-label={dismissLabel} title={dismissLabel} onClick={onDismiss}>
          <X aria-hidden="true" size={15} />
        </button>
      ) : null}
    </div>
  );
}

export function LoadingState({ label }: { label: string }) {
  return (
    <div className="flex min-h-56 items-center justify-center text-sm text-neutral-500 dark:text-neutral-400" role="status">
      <LoaderCircle aria-hidden="true" className="mr-2 animate-spin" size={18} />
      {label}
    </div>
  );
}

interface EmptyStateProps {
  description?: string;
  icon?: IconComponent;
  title: string;
}

export function EmptyState({ description, icon: Icon = Inbox, title }: EmptyStateProps) {
  return (
    <div className="flex min-h-56 flex-col items-center justify-center px-6 py-10 text-center">
      <div className="flex h-11 w-11 items-center justify-center rounded-full bg-neutral-100 text-neutral-400 dark:bg-neutral-800 dark:text-neutral-500">
        <Icon aria-hidden="true" size={20} strokeWidth={1.7} />
      </div>
      <p className="mt-3 text-sm font-medium text-neutral-700 dark:text-neutral-200">{title}</p>
      {description ? <p className="mt-1 max-w-sm text-xs leading-5 text-neutral-500 dark:text-neutral-400">{description}</p> : null}
    </div>
  );
}

interface PaginationBarProps {
  loading: boolean;
  nextDisabled: boolean;
  nextLabel: string;
  onNext: () => void;
  onPrevious: () => void;
  pageLabel: string;
  previousDisabled: boolean;
  previousLabel: string;
}

export function PaginationBar({
  loading,
  nextDisabled,
  nextLabel,
  onNext,
  onPrevious,
  pageLabel,
  previousDisabled,
  previousLabel,
}: PaginationBarProps) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-3 border-t border-neutral-200 px-4 py-3 dark:border-neutral-800 sm:px-5">
      <span className="text-xs text-neutral-500 dark:text-neutral-400">{pageLabel}</span>
      <div className="flex items-center gap-2">
        <button
          type="button"
          className={`${SECONDARY_BUTTON_CLASS} px-2.5`}
          aria-label={previousLabel}
          disabled={previousDisabled || loading}
          onClick={onPrevious}
        >
          <ChevronLeft aria-hidden="true" size={16} />
          <span className="hidden sm:inline">{previousLabel}</span>
        </button>
        <button
          type="button"
          className={`${SECONDARY_BUTTON_CLASS} px-2.5`}
          aria-label={nextLabel}
          disabled={nextDisabled || loading}
          onClick={onNext}
        >
          <span className="hidden sm:inline">{nextLabel}</span>
          <ChevronRight aria-hidden="true" size={16} />
        </button>
      </div>
    </div>
  );
}
