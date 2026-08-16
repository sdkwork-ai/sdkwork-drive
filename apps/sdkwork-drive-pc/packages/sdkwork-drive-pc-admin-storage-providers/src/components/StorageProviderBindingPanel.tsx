import React, { useState } from 'react';
import type { StorageProviderBindingView, StorageProviderView } from '../types/storageProviderAdminTypes';
import { getProviderKindMeta } from '../utils/providerKindConfig';
import { SELECT_CLASS, INPUT_CLASS, PRIMARY_BUTTON_CLASS, SECONDARY_BUTTON_CLASS, CARD_CLASS, BADGE_BASE_CLASS } from '../utils/uiPrimitives';
import { ConfirmDialog } from './ConfirmDialog';
import { useTranslation } from '../hooks/useTranslation';

interface StorageProviderBindingPanelProps {
  providers: StorageProviderView[];
  binding?: StorageProviderBindingView;
  onSetDefaultBinding: (providerId: string, spaceId?: string) => void;
  onDeleteDefaultBinding: (spaceId?: string) => void;
  pending?: boolean;
}

export function StorageProviderBindingPanel({
  providers,
  binding,
  onSetDefaultBinding,
  onDeleteDefaultBinding,
  pending,
}: StorageProviderBindingPanelProps) {
  const { t } = useTranslation();
  const [providerId, setProviderId] = useState('');
  const [spaceId, setSpaceId] = useState('');
  const [showClearConfirm, setShowClearConfirm] = useState(false);

  const boundProvider = providers.find((p) => p.id === binding?.providerId);

  return (
    <div className={CARD_CLASS}>
      <div className="border-b border-neutral-100 px-5 py-3 dark:border-neutral-800">
        <h3 className="text-sm font-semibold text-neutral-900 dark:text-neutral-100">{t('storageProviders.defaultBinding')}</h3>
        <p className="mt-0.5 text-[11px] text-neutral-500 dark:text-neutral-400">
          {t('storageProviders.bindingDesc')}
        </p>
      </div>

      <div className="px-5 py-4">
        {/* Current binding visualization */}
        <div className="mb-4 rounded-md border border-neutral-100 bg-neutral-50 p-3 dark:border-neutral-800 dark:bg-neutral-900/50">
          <div className="text-[10px] font-semibold uppercase tracking-wider text-neutral-400 dark:text-neutral-500">{t('storageProviders.currentBinding')}</div>
          {binding && boundProvider ? (
            <div className="mt-2 flex items-center gap-3">
              <div className="flex items-center gap-2">
                <div className="flex h-8 items-center rounded border border-neutral-200 bg-white px-2 text-xs font-medium dark:border-neutral-700 dark:bg-neutral-800">
                  <svg className="mr-1.5 h-3.5 w-3.5 text-neutral-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                  </svg>
                  Tenant
                </div>
              </div>

              <svg className="h-4 w-4 text-neutral-300 dark:text-neutral-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>

              <div className="flex items-center gap-2">
                {(() => {
                  const meta = getProviderKindMeta(boundProvider.providerKind);
                  return (
                    <div className="flex items-center gap-1.5 rounded border border-blue-200 bg-blue-50 px-2 py-1 dark:border-blue-800 dark:bg-blue-950/30">
                      <span className={`inline-flex h-5 w-5 items-center justify-center rounded text-[9px] font-bold ${meta.bgClass} ${meta.textClass}`}>
                        {meta.icon}
                      </span>
                      <span className="text-xs font-medium text-blue-700 dark:text-blue-300">
                        {boundProvider.displayName}
                      </span>
                    </div>
                  );
                })()}
              </div>

              {binding.spaceId && (
                <>
                  <svg className="h-4 w-4 text-neutral-300 dark:text-neutral-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
                  </svg>
                  <span className={`${BADGE_BASE_CLASS} bg-purple-50 text-purple-700 dark:bg-purple-950/30 dark:text-purple-300`}>
                    Space: {binding.spaceId}
                  </span>
                </>
              )}
            </div>
          ) : (
            <div className="mt-2 text-xs text-neutral-500 dark:text-neutral-400">
              {t('storageProviders.notConfigured')}
            </div>
          )}
        </div>

        {/* Set binding form */}
        <div className="space-y-2">
          <select
            value={providerId}
            onChange={(e) => setProviderId(e.target.value)}
            className={SELECT_CLASS}
          >
            <option value="">{t('storageProviders.selectProvider')}</option>
            {providers.filter((p) => p.status === 'active').map((provider) => (
              <option key={provider.id} value={provider.id}>
                {provider.displayName || provider.id}
              </option>
            ))}
          </select>
          <div className="flex gap-2">
            <input
              value={spaceId}
              onChange={(e) => setSpaceId(e.target.value)}
              className={`${INPUT_CLASS} flex-1`}
              placeholder={t('storageProviders.spaceIdOptional')}
            />
            <button
              type="button"
              className={PRIMARY_BUTTON_CLASS}
              disabled={pending || !providerId}
              onClick={() => onSetDefaultBinding(providerId, spaceId || undefined)}
            >
              {t('storageProviders.set')}
            </button>
          </div>
        </div>

        {binding && (
          <div className="mt-3 flex justify-end">
            <button
              type="button"
              className="text-xs text-red-600 hover:text-red-700 dark:text-red-400"
              onClick={() => setShowClearConfirm(true)}
            >
              {t('storageProviders.clear')}
            </button>
          </div>
        )}
      </div>

      <ConfirmDialog
        open={showClearConfirm}
        title={t('storageProviders.clearConfirmTitle')}
        message={t('storageProviders.clearConfirmMessage')}
        confirmLabel={t('storageProviders.clearConfirmLabel')}
        variant="danger"
        onConfirm={() => { onDeleteDefaultBinding(spaceId || undefined); setShowClearConfirm(false); }}
        onCancel={() => setShowClearConfirm(false)}
      />
    </div>
  );
}
