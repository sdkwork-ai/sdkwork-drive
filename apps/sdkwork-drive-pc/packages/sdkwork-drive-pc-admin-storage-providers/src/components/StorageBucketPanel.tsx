import React, { useCallback, useState } from 'react';
import type { StorageProviderAdminService } from '../services/storageProviderAdminService';
import type { StorageProviderView } from '../types/storageProviderAdminTypes';
import { formatMutationError } from '../utils/mutationError';
import { useTranslation } from '../hooks/useTranslation';
import { ConfirmDialog } from './ConfirmDialog';

interface BucketInfo {
  bucket: string;
  exists: boolean;
  configured: boolean;
  creationDate?: string;
}

interface StorageBucketPanelProps {
  provider: StorageProviderView;
  service: StorageProviderAdminService;
}

export function StorageBucketPanel({ provider, service }: StorageBucketPanelProps) {
  const { t } = useTranslation();
  const [buckets, setBuckets] = useState<BucketInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [bucketExists, setBucketExists] = useState<boolean | null>(null);
  const [pendingAction, setPendingAction] = useState<'create' | 'delete' | null>(null);

  const loadBuckets = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const items = await service.listBuckets(provider.id);
      setBuckets(items.map((item) => ({
        bucket: item.bucket,
        exists: true,
        configured: item.configured,
        creationDate: item.creationDate,
      })));
    } catch (err) {
      setError(formatMutationError(err, t('errorLoadBuckets')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, service, t]);

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

  const createBucket = useCallback(async () => {
    setPendingAction(null);
    setLoading(true);
    setError(null);
    try {
      await service.createBucket(provider.id);
      setBucketExists(true);
      await loadBuckets();
    } catch (err) {
      setError(formatMutationError(err, t('errorCreateBucket')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, service, loadBuckets, t]);

  const deleteBucket = useCallback(async () => {
    setPendingAction(null);
    setLoading(true);
    setError(null);
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
    <div className="border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-[#171717]">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-semibold">{t('buckets')}</h3>
        <button type="button" onClick={loadBuckets} disabled={loading} className="text-xs text-blue-600 hover:underline">
          {t('listAll')}
        </button>
      </div>

      {error && <div className="mb-3 rounded bg-red-50 px-3 py-2 text-xs text-red-700 dark:bg-red-950/20 dark:text-red-300">{error}</div>}

      <div className="mb-3 rounded-md bg-neutral-50 p-3 dark:bg-neutral-800">
        <div className="text-xs text-neutral-500">{t('configuredBucket')}</div>
        <div className="font-mono text-sm">{provider.bucket}</div>
        {bucketExists !== null && (
          <div className={`mt-1 text-xs ${bucketExists ? 'text-emerald-600' : 'text-red-600'}`}>
            {bucketExists ? t('bucketReachable') : t('bucketUnreachable')}
          </div>
        )}
      </div>

      <div className="flex flex-wrap gap-2">
        <button type="button" onClick={checkBucket} disabled={loading} className="rounded border px-3 py-1.5 text-xs">
          {t('checkExists')}
        </button>
        <button type="button" onClick={() => setPendingAction('create')} disabled={loading} className="rounded bg-emerald-600 px-3 py-1.5 text-xs text-white">
          {t('createBucket')}
        </button>
        <button type="button" onClick={() => setPendingAction('delete')} disabled={loading} className="rounded bg-red-600 px-3 py-1.5 text-xs text-white">
          {t('deleteBucket')}
        </button>
      </div>

      {buckets.length > 0 && (
        <div className="mt-4 rounded border dark:border-neutral-700">
          {buckets.map((bucket) => (
            <div key={bucket.bucket} className="flex items-center justify-between border-b px-3 py-2 text-xs dark:border-neutral-800">
              <span>{bucket.bucket}</span>
              {bucket.creationDate && <span className="text-neutral-400">{bucket.creationDate}</span>}
            </div>
          ))}
        </div>
      )}
    </div>
    <ConfirmDialog
      busy={loading}
      confirmLabel={pendingAction === 'create' ? t('createBucket') : t('deleteBucket')}
      message={pendingAction === 'create' ? t('confirmCreateBucket') : t('deleteBucketConfirm', { bucket: provider.bucket })}
      onCancel={() => setPendingAction(null)}
      onConfirm={() => {
        if (pendingAction === 'create') void createBucket();
        if (pendingAction === 'delete') void deleteBucket();
      }}
      open={pendingAction !== null}
      title={pendingAction === 'create' ? t('createBucket') : t('deleteBucket')}
      variant={pendingAction === 'delete' ? 'danger' : 'default'}
    />
    </>
  );
}
