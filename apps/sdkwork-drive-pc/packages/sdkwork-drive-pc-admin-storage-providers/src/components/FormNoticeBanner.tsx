import React from 'react';

interface FormNoticeBannerProps {
  type: 'success' | 'error';
  message: string;
  onDismiss?: () => void;
}

export function FormNoticeBanner({ type, message, onDismiss }: FormNoticeBannerProps) {
  const styles =
    type === 'success'
      ? 'border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950/30 dark:text-emerald-200'
      : 'border-red-200 bg-red-50 text-red-800 dark:border-red-900 dark:bg-red-950/30 dark:text-red-200';

  return (
    <div className={`mb-4 flex items-start gap-3 rounded-lg border px-4 py-3 text-sm ${styles}`}>
      {type === 'success' ? (
        <svg className="mt-0.5 h-4 w-4 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      ) : (
        <svg className="mt-0.5 h-4 w-4 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      )}
      <span className="flex-1 whitespace-pre-wrap break-words">{message}</span>
      {onDismiss && (
        <button type="button" className="text-current opacity-50 hover:opacity-100" onClick={onDismiss}>
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      )}
    </div>
  );
}
