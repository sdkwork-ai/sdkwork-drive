import React, { useCallback, useEffect, useState } from 'react';
import {
  CheckCircle2,
  CircleAlert,
  Database,
  LoaderCircle,
  RefreshCw,
  Search,
  ShieldCheck,
  Trash2,
} from 'lucide-react';
import type { StorageProviderAdminService } from '../services/storageProviderAdminService';
import type { StorageProviderView } from '../types/storageProviderAdminTypes';
import { formatMutationError } from '../utils/mutationError';
import { formatDriveDate } from '../utils/formatDriveTimestamp';
import { useTranslation } from '../hooks/useTranslation';
import { ConfirmDialog } from './ConfirmDialog';
import {
  BADGE_BASE_CLASS,
  CARD_CLASS,
  DANGER_BUTTON_CLASS,
  GHOST_BUTTON_CLASS,
  PRIMARY_BUTTON_CLASS,
  SECONDARY_BUTTON_CLASS,
} from '../utils/uiPrimitives';

interface BucketInfo {
  bucket: string;
  exists: boolean;
  configured: boolean;
  creationDateIso?: string;
}

interface StorageBucketPanelProps {
  provider: StorageProviderView;
  service: StorageProviderAdminService;
}

export function StorageBucketPanel({ provider, service }: StorageBucketPanelProps) {
  const { t, language } = useTranslation();
  const [buckets, setBuckets] = useState<BucketInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [bucketExists, setBucketExists] = useState<boolean | null>(null);
  const [pendingDelete, setPendingDelete] = useState(false);
  const [initializeOutcome, setInitializeOutcome] = useState<'created' | 'consistent' | null>(null);

  const loadBuckets = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const items = await service.listBuckets(provider.id);
      setBuckets(items.map((item) => ({
        bucket: item.bucket,
        exists: true,
        configured: item.configured,
        creationDateIso: item.creationDateIso,
      })));
      // The vendor list is authoritative for the configured bucket: when it
      // shows up the existence badge can light up without a separate HEAD.
      setBucketExists((current) =>
        items.some((item) => item.bucket === provider.bucket) ? true : current,
      );
    } catch (err) {
      setError(formatMutationError(err, t('errorLoadBuckets')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, provider.bucket, service, t]);

  // Buckets sync automatically for the selected provider, so the panel mirrors
  // what the vendor holds the moment a configuration is picked on the left.
  useEffect(() => {
    void loadBuckets();
  }, [loadBuckets]);

  const checkBucket = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await service.headBucket(provider.id);
      setBucketExists(result.exists);
    } catch (err) {
      setError(formatMutationError(err, t('errorCheckBucket')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, service, t]);

  const initializeBucket = useCallback(async () => {
    setLoading(true);
    setError(null);
    setInitializeOutcome(null);
    try {
      const result = await service.initializeBucket(provider.id);
      setBucketExists(true);
      setInitializeOutcome(result.changed ? 'created' : 'consistent');
      await loadBuckets();
    } catch (err) {
      setError(formatMutationError(err, t('errorInitializeBucket')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, service, loadBuckets, t]);

  const deleteBucket = useCallback(async () => {
    setPendingDelete(false);
    setLoading(true);
    setError(null);
    setInitializeOutcome(null);
    try {
      await service.deleteBucket(provider.id);
      setBucketExists(false);
      setBuckets([]);
    } catch (err) {
      setError(formatMutationError(err, t('errorDeleteBucket')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, provider.bucket, service, t]);

  return (
    <>
    <section className={CARD_CLASS}>
      <div className="flex items-center justify-between gap-3 border-b border-neutral-100 px-5 py-3 dark:border-neutral-800">
        <div className="flex min-w-0 items-center gap-2">
          <Database aria-hidden="true" className="shrink-0 text-neutral-400" size={15} />
          <h3 className="text-sm font-semibold">{t('buckets')}</h3>
          <span className={`${BADGE_BASE_CLASS} truncate bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300`}>
            {provider.bucket}
          </span>
        </div>
        <button type="button" onClick={loadBuckets} disabled={loading} className={`${GHOST_BUTTON_CLASS} shrink-0`}>
          <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={14} />
          {t('refresh')}
        </button>
      </div>

      <div className="px-5 py-4">
        {error && (
          <div className="mb-3 flex items-start gap-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700 dark:border-red-900 dark:bg-red-950/20 dark:text-red-300">
            <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={14} />
            <span className="flex-1">{error}</span>
          </div>
        )}

        <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
          <div className="flex min-w-0 items-center gap-2">
            <span className="text-xs text-neutral-500">{t('configuredBucket')}</span>
            <span className="truncate font-mono text-sm font-medium">{provider.bucket}</span>
            {bucketExists !== null && (
              <span className={`${BADGE_BASE_CLASS} shrink-0 ${
                bucketExists
                  ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300'
                  : 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300'
              }`}>
                <span className={`h-1.5 w-1.5 rounded-full ${bucketExists ? 'bg-emerald-500' : 'bg-red-500'}`} />
                {bucketExists ? t('bucketReachable') : t('bucketUnreachable')}
              </span>
            )}
          </div>

          <span className="flex-1" />

          <div className="flex flex-wrap items-center gap-2">
            <button type="button" onClick={checkBucket} disabled={loading} className={SECONDARY_BUTTON_CLASS}>
              {loading ? (
                <LoaderCircle aria-hidden="true" className="animate-spin" size={14} />
              ) : (
                <Search aria-hidden="true" size={14} />
              )}
              {t('checkExists')}
            </button>
            <button
              type="button"
              onClick={initializeBucket}
              disabled={loading}
              title={t('initializeBucketHint')}
              className={PRIMARY_BUTTON_CLASS}
            >
              <ShieldCheck aria-hidden="true" size={14} />
              {t('initializeBucket')}
            </button>
            {bucketExists ? (
              <button type="button" onClick={() => setPendingDelete(true)} disabled={loading} className={DANGER_BUTTON_CLASS}>
                <Trash2 aria-hidden="true" size={14} />
                {t('deleteBucket')}
              </button>
            ) : null}
          </div>
        </div>

        {initializeOutcome ? (
          <p
            className={`mt-3 flex items-center gap-1.5 text-xs ${
              initializeOutcome === 'created'
                ? 'text-emerald-600 dark:text-emerald-400'
                : 'text-blue-600 dark:text-blue-400'
            }`}
          >
            <CheckCircle2 aria-hidden="true" size={13} />
            {initializeOutcome === 'created'
              ? t('bucketInitializedCreated')
              : t('bucketInitializedConsistent')}
          </p>
        ) : null}

        {buckets.length > 0 ? (
          <div className="mt-4 divide-y divide-neutral-100 rounded-md border border-neutral-200 dark:divide-neutral-800 dark:border-neutral-700">
            {buckets.map((bucket) => (
              <div
                key={bucket.bucket}
                className={`flex items-center justify-between gap-3 px-3 py-2 text-xs ${
                  bucket.configured ? 'bg-blue-50/60 dark:bg-blue-950/20' : ''
                }`}
              >
                <span className="flex min-w-0 items-center gap-2">
                  <Database aria-hidden="true" className="shrink-0 text-neutral-400" size={13} />
                  <span className="truncate font-mono">{bucket.bucket}</span>
                  {bucket.configured ? (
                    <span className={`${BADGE_BASE_CLASS} shrink-0 bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300`}>
                      {t('configuredBucketShort')}
                    </span>
                  ) : null}
                </span>
                {bucket.creationDateIso ? (
                  <span className="shrink-0 text-neutral-400">
                    {formatDriveDate(bucket.creationDateIso, language)}
                  </span>
                ) : null}
              </div>
            ))}
          </div>
        ) : !loading && !error ? (
          <p className="mt-3 flex items-center gap-1.5 text-xs text-neutral-400">
            <CheckCircle2 aria-hidden="true" size={13} />
            {t('bucketsEmptyList')}
          </p>
        ) : null}
      </div>
    </section>
    <ConfirmDialog
      busy={loading}
      confirmLabel={t('deleteBucket')}
      message={t('deleteBucketConfirm', { bucket: provider.bucket })}
      onCancel={() => setPendingDelete(false)}
      onConfirm={() => void deleteBucket()}
      open={pendingDelete}
      title={t('deleteBucket')}
      variant="danger"
    />
    </>
  );
}
