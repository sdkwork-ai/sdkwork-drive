import React from 'react';
import { useTranslation } from 'sdkwork-drive-pc-commons';
import { DriveErrorBoundary } from './DriveErrorBoundary';

export function DriveErrorBoundaryShell({ children }: { children: React.ReactNode }) {
  const { t } = useTranslation();
  return (
    <DriveErrorBoundary
      fallbackTitle={t('settings.errorBoundaryTitle')}
      fallbackDescription={t('settings.errorBoundaryDesc')}
      retryLabel={t('settings.errorBoundaryRetry')}
    >
      {children}
    </DriveErrorBoundary>
  );
}
