import React, { useEffect, useMemo, useState } from 'react';
import { CircleAlert, LoaderCircle, RefreshCw } from 'lucide-react';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import { StorageBucketPanel } from '../components/StorageBucketPanel';
import { StorageObjectBrowser } from '../components/StorageObjectBrowser';
import {
  createStorageProviderAdminService,
  type StorageProviderAdminService,
} from '../services/storageProviderAdminService';
import type { StorageProviderView } from '../types/storageProviderAdminTypes';
import { getAllProviderKindMeta, getProviderKindMeta } from '../utils/providerKindConfig';
import { SELECT_CLASS, BADGE_BASE_CLASS, CARD_CLASS, SECONDARY_BUTTON_CLASS } from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

interface StorageBucketsAdminPageProps {
  adminStorageSdkClient: DriveAdminStorageSdkClient;
  getSession: () => SessionSnapshot;
}

export function StorageBucketsAdminPage({
  adminStorageSdkClient,
  getSession,
}: StorageBucketsAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo<StorageProviderAdminService>(
    () => createStorageProviderAdminService({ adminStorageSdkClient, getSession }),
    [adminStorageSdkClient, getSession],
  );
  const [providers, setProviders] = useState<StorageProviderView[]>([]);
  const [selectedProviderId, setSelectedProviderId] = useState('');
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);

  const load = (signal?: AbortSignal) => {
    setLoading(true);
    setLoadError(false);
    service
      .listProviders({ signal })
      .then((items) => {
        const active = items.filter((provider) => provider.status === 'active');
        setProviders(active);
        setSelectedProviderId((current) =>
          active.some((provider) => provider.id === current) ? current : (active[0]?.id ?? ''),
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

  // Group options by provider kind so operators can pick the right
  // configuration under the desired 服务商 (one kind owns many configs).
  const kindGroups = useMemo(() => {
    const groups = new Map<string, StorageProviderView[]>();
    for (const provider of providers) {
      const key = provider.providerKind.startsWith('custom:') ? 'custom' : provider.providerKind;
      const list = groups.get(key) ?? [];
      list.push(provider);
      groups.set(key, list);
    }
    return getAllProviderKindMeta()
      .map((meta) => ({
        meta,
        providers: groups.get(meta.value) ?? [],
      }))
      .filter((group) => group.providers.length > 0);
  }, [providers]);

  return (
    <main className="flex h-full flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <div aria-label={t('bucketsPageTitle')} className="flex shrink-0 flex-wrap items-center justify-between gap-3 px-4 pt-4 sm:px-6 sm:pt-6">
        <span className={`${BADGE_BASE_CLASS} bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300`}>
          {t('bucketsSummary', { count: providers.length })}
        </span>
        <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading} onClick={() => load()}>
          <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
          {t('refresh')}
        </button>
      </div>

      <div className="flex-1 overflow-auto p-4 sm:p-6">
        {loadError ? (
          <div className="flex items-start gap-3 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/30 dark:text-red-200">
            <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
            <span>{t('bucketsNoticeLoadFailed')}</span>
          </div>
        ) : loading ? (
          <div className="flex min-h-[360px] items-center justify-center rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900">
            <div className="flex items-center gap-3 text-sm text-neutral-500">
              <LoaderCircle aria-hidden="true" className="animate-spin" size={19} />
              {t('bucketsLoading')}
            </div>
          </div>
        ) : providers.length === 0 ? (
          <div className="flex items-start gap-3 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800 dark:border-amber-900 dark:bg-amber-950/30 dark:text-amber-200">
            <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
            <span>{t('bucketsNoActiveProviders')}</span>
          </div>
        ) : (
          <div className="grid content-start gap-4">
            <div className={`${CARD_CLASS} px-5 py-4`}>
              <label className="grid gap-2 text-xs font-medium text-neutral-600 dark:text-neutral-300">
                {t('bucketsSelectProvider')}
                <select
                  value={selectedProviderId}
                  onChange={(event) => setSelectedProviderId(event.target.value)}
                  className={SELECT_CLASS}
                >
                  {kindGroups.map((group) => (
                    <optgroup key={group.meta.value} label={group.meta.label}>
                      {group.providers.map((provider) => (
                        <option key={provider.id} value={provider.id}>
                          [{group.meta.shortLabel}] {provider.displayName} ({provider.bucket})
                        </option>
                      ))}
                    </optgroup>
                  ))}
                </select>
              </label>
              {selectedProvider ? (
                <div className="mt-3 flex flex-wrap items-center gap-2 text-xs text-neutral-500">
                  <span className={`inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-bold ${getProviderKindMeta(selectedProvider.providerKind).bgClass} ${getProviderKindMeta(selectedProvider.providerKind).textClass}`}>
                    {getProviderKindMeta(selectedProvider.providerKind).icon}
                  </span>
                  <span className="font-mono break-all">{selectedProvider.endpointUrl}</span>
                  {selectedProvider.region ? <span>· {selectedProvider.region}</span> : null}
                </div>
              ) : null}
            </div>

            {selectedProvider ? (
              <>
                <StorageBucketPanel key={`bucket-${selectedProvider.id}`} provider={selectedProvider} service={service} />
                <StorageObjectBrowser key={`objects-${selectedProvider.id}`} provider={selectedProvider} service={service} />
              </>
            ) : null}
          </div>
        )}
      </div>
    </main>
  );
}
