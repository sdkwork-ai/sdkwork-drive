import React, { useEffect, useMemo, useState } from 'react';
import { CircleAlert, LoaderCircle, Plus, RefreshCw, Settings2 } from 'lucide-react';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import { StorageBucketPanel } from '../components/StorageBucketPanel';
import { StorageObjectBrowser } from '../components/StorageObjectBrowser';
import { StorageProviderNavList } from '../components/StorageProviderNavList';
import {
  createStorageProviderAdminService,
  type StorageProviderAdminService,
} from '../services/storageProviderAdminService';
import type { StorageProviderView } from '../types/storageProviderAdminTypes';
import { getProviderKindMeta } from '../utils/providerKindConfig';
import {
  BADGE_BASE_CLASS,
  GHOST_BUTTON_CLASS,
  SECONDARY_BUTTON_CLASS,
} from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';
import { useProviderOptions } from '../hooks/useProviderOptions';

interface StorageBucketsAdminPageProps {
  adminStorageSdkClient: DriveAdminStorageSdkClient;
  getSession: () => SessionSnapshot;
  /**
   * Quick-configure entry points. The bucket page is where operators notice a
   * missing or wrong configuration, so it must be able to hand them off to the
   * provider pages. Both are optional: a host that renders this page without
   * the provider surfaces mounted simply does not get the shortcuts.
   */
  onConfigureProviders?: () => void;
  onCreateProvider?: () => void;
}

export function StorageBucketsAdminPage({
  adminStorageSdkClient,
  getSession,
  onConfigureProviders,
  onCreateProvider,
}: StorageBucketsAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo<StorageProviderAdminService>(
    () => createStorageProviderAdminService({ adminStorageSdkClient, getSession }),
    [adminStorageSdkClient, getSession],
  );
  /**
   * The left rail *is* the provider switch, so it reads the option set rather
   * than one cursor page of the provider table.
   *
   * One page of 20 is what made a freshly initialized plane (25 providers) hide
   * its last five — including the default-bound COS row — behind a page control
   * this rail does not even have. The hook asks for the maximum page the
   * contract allows and exposes a continuation for anything past it.
   */
  const providerOptions = useProviderOptions(service, { status: 'active' });
  const providers = providerOptions.items;
  const [selectedProviderId, setSelectedProviderId] = useState('');
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);

  const load = (signal?: AbortSignal) => {
    setLoading(true);
    setLoadError(false);
    providerOptions
      .reload(signal)
      .then((items) => {
        setSelectedProviderId((current) =>
          items.some((provider) => provider.id === current) ? current : (items[0]?.id ?? ''),
        );
      })
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setLoadError(true);
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    const c = new AbortController();
    load(c.signal);
    return () => c.abort();
  }, [service]);

  const selectedProvider = providers.find((provider) => provider.id === selectedProviderId);
  const selectedMeta = selectedProvider ? getProviderKindMeta(selectedProvider.providerKind) : undefined;

  return (
    <main className="flex h-full flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      {/* Header keeps only identity and hand-off actions: provider switching
          moved into the left rail, so the row stays readable at any width. */}
      <header
        aria-label={t('bucketsPageTitle')}
        className="flex shrink-0 flex-wrap items-center gap-2 border-b border-neutral-200 bg-white px-4 py-2.5 dark:border-neutral-800 dark:bg-neutral-900 sm:px-6"
      >
        <h1 className="shrink-0 text-sm font-semibold text-neutral-900 dark:text-neutral-100">
          {t('bucketsPageTitle')}
        </h1>
        <span
          className={`${BADGE_BASE_CLASS} shrink-0 bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300`}
        >
          {t('bucketsSummary', { count: providers.length })}
        </span>

        <span className="flex-1" />

        <div className="flex shrink-0 items-center gap-2">
          {onConfigureProviders ? (
            <button
              type="button"
              className={GHOST_BUTTON_CLASS}
              title={t('bucketsQuickConfigureHint')}
              onClick={onConfigureProviders}
            >
              <Settings2 aria-hidden="true" size={15} />
              <span className="hidden sm:inline">{t('bucketsQuickConfigure')}</span>
            </button>
          ) : null}
          {onCreateProvider ? (
            <button
              type="button"
              className={SECONDARY_BUTTON_CLASS}
              title={t('bucketsQuickCreateHint')}
              onClick={onCreateProvider}
            >
              <Plus aria-hidden="true" size={15} />
              <span className="hidden sm:inline">{t('bucketsQuickCreate')}</span>
            </button>
          ) : null}
          <span className="mx-0.5 hidden h-5 w-px shrink-0 bg-neutral-200 dark:bg-neutral-700 sm:block" />
          <button
            type="button"
            className={GHOST_BUTTON_CLASS}
            aria-label={t('refresh')}
            title={t('refresh')}
            disabled={loading}
            onClick={() => load()}
          >
            <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
          </button>
        </div>
      </header>

      {loadError || loading || providers.length === 0 ? (
        <div className="flex-1 overflow-auto p-4 sm:p-6">
          {loadError ? (
            <div className="flex items-start gap-3 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/30 dark:text-red-200">
              <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
              <span className="flex-1">{t('bucketsNoticeLoadFailed')}</span>
              <button type="button" className={SECONDARY_BUTTON_CLASS} onClick={() => load()}>
                {t('overviewRetry')}
              </button>
            </div>
          ) : loading ? (
            <div className="flex min-h-[280px] items-center justify-center rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900">
              <div className="flex items-center gap-3 text-sm text-neutral-500">
                <LoaderCircle aria-hidden="true" className="animate-spin" size={19} />
                {t('bucketsLoading')}
              </div>
            </div>
          ) : (
            <div className="flex flex-col items-start gap-4 rounded-lg border border-amber-200 bg-amber-50 px-5 py-6 text-sm text-amber-800 dark:border-amber-900 dark:bg-amber-950/30 dark:text-amber-200">
              <div className="flex items-start gap-3">
                <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
                <div>
                  <p className="font-medium">{t('bucketsNoActiveProvidersTitle')}</p>
                  <p className="mt-1 text-xs text-amber-700 dark:text-amber-300">{t('bucketsNoActiveProviders')}</p>
                </div>
              </div>
              <div className="flex flex-wrap items-center gap-2 pl-7">
                {onCreateProvider ? (
                  <button type="button" className={SECONDARY_BUTTON_CLASS} onClick={onCreateProvider}>
                    <Plus aria-hidden="true" size={15} />
                    {t('bucketsQuickCreate')}
                  </button>
                ) : null}
                {onConfigureProviders ? (
                  <button type="button" className={GHOST_BUTTON_CLASS} onClick={onConfigureProviders}>
                    <Settings2 aria-hidden="true" size={15} />
                    {t('bucketsQuickConfigure')}
                  </button>
                ) : null}
                <button type="button" className={GHOST_BUTTON_CLASS} onClick={() => load()}>
                  <RefreshCw aria-hidden="true" size={15} />
                  {t('refresh')}
                </button>
              </div>
            </div>
          )}
        </div>
      ) : (
        <div className="flex min-h-0 flex-1 flex-col lg:flex-row">
          <StorageProviderNavList
            providers={providers}
            selectedProviderId={selectedProviderId}
            onSelect={setSelectedProviderId}
            hasMore={providerOptions.hasMore}
            loadingMore={providerOptions.loading}
            onLoadMore={() => void providerOptions.loadMore()}
          />

          <section className="flex min-h-0 flex-1 flex-col lg:overflow-y-auto">
            {selectedProvider ? (
              <>
                {/* Selection identity strip: which configuration owns the
                    buckets and files below, always visible while scrolling. */}
                <div className="sticky top-0 z-10 shrink-0 border-b border-neutral-200 bg-white/95 px-4 py-2 backdrop-blur dark:border-neutral-800 dark:bg-neutral-900/95 sm:px-6">
                  <div className="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1">
                    <span
                      title={selectedProvider.providerKind}
                      className={`inline-flex shrink-0 items-center rounded-md px-1.5 py-0.5 text-[10px] font-bold ${selectedMeta?.bgClass ?? ''} ${selectedMeta?.textClass ?? ''}`}
                    >
                      {selectedMeta?.icon}
                    </span>
                    <h2 className="shrink-0 text-sm font-semibold">{selectedProvider.displayName}</h2>
                    <span
                      className={`${BADGE_BASE_CLASS} shrink-0 ${
                        selectedProvider.credentialConfigured
                          ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300'
                          : 'bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-300'
                      }`}
                    >
                      <span
                        className={`h-1.5 w-1.5 rounded-full ${
                          selectedProvider.credentialConfigured ? 'bg-emerald-500' : 'bg-amber-500'
                        }`}
                      />
                      {selectedProvider.credentialConfigured ? t('credentialSet') : t('credentialMissing')}
                    </span>
                    <span className="hidden h-4 w-px shrink-0 bg-neutral-200 dark:bg-neutral-700 sm:block" />
                    <span className="min-w-0 truncate font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
                      {selectedProvider.endpointUrl}
                    </span>
                    {selectedProvider.region ? (
                      <span className="shrink-0 text-[11px] text-neutral-500 dark:text-neutral-400">
                        · {selectedProvider.region}
                      </span>
                    ) : null}
                    <span className="shrink-0 text-[11px] text-neutral-500 dark:text-neutral-400">
                      · {t('configuredBucket')}{' '}
                      <span className="font-mono text-neutral-600 dark:text-neutral-300">
                        {selectedProvider.bucket}
                      </span>
                    </span>
                  </div>
                </div>

                <div className="grid content-start gap-4 p-4 sm:p-6">
                  <StorageBucketPanel key={`bucket-${selectedProvider.id}`} provider={selectedProvider} service={service} />
                  <StorageObjectBrowser key={`objects-${selectedProvider.id}`} provider={selectedProvider} service={service} />
                </div>
              </>
            ) : null}
          </section>
        </div>
      )}
    </main>
  );
}
