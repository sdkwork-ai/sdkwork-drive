import React, { useCallback, useEffect, useState } from 'react';
import { CircleAlert, CheckCircle2, LoaderCircle, RefreshCw } from 'lucide-react';
import type { StorageProviderAdminService } from '../services/storageProviderAdminService';
import type { StorageProviderView } from '../types/storageProviderAdminTypes';
import { SECONDARY_BUTTON_CLASS } from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

type ProbeStatus = 'idle' | 'checking' | 'exists' | 'missing' | 'unknown';

interface StorageBucketReadinessProps {
  /** Provider configuration whose bucket the binding will write into. */
  provider: StorageProviderView | undefined;
  service: StorageProviderAdminService;
  /**
   * Called after the bucket was created, so the surface can refresh the rows it
   * derives from the provider set.
   */
  onInitialized?: () => void;
}

/**
 * Whether the selected provider configuration's bucket actually exists.
 *
 * Binding a space type to a bucket that was never created is the failure this
 * control exists for: the binding saves, uploads fail, and nothing on the page
 * said so. The check is a vendor `HeadBucket` through the provider's own
 * credentials, so it also answers "are these credentials usable at all" — the
 * second silent precondition of every binding.
 *
 * It is deliberately on demand plus automatic on provider change, never a probe
 * per table row: an operator console must not issue one vendor call per row.
 */
export function StorageBucketReadiness({
  provider,
  service,
  onInitialized,
}: StorageBucketReadinessProps) {
  const { t } = useTranslation();
  const [status, setStatus] = useState<ProbeStatus>('idle');
  const [bucket, setBucket] = useState('');
  const [pending, setPending] = useState(false);

  const providerId = provider?.id;

  const probe = useCallback(async () => {
    if (!providerId) {
      setStatus('idle');
      return;
    }
    setStatus('checking');
    try {
      const result = await service.headBucket(providerId);
      setBucket(result.bucket || provider?.bucket || '');
      setStatus(result.exists ? 'exists' : 'missing');
    } catch {
      // An unreachable vendor, an unconfigured credential pair, or a bucket the
      // credentials may not head all land here. Naming them apart is not
      // possible from one error, so the control says "cannot verify" and keeps
      // the operator's own retry in reach.
      setBucket(provider?.bucket ?? '');
      setStatus('unknown');
    }
  }, [providerId, provider?.bucket, service]);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      if (cancelled) return;
      await probe();
    })();
    return () => {
      cancelled = true;
    };
  }, [probe]);

  const initializeBucket = async () => {
    if (!providerId) return;
    setPending(true);
    try {
      await service.initializeBucket(providerId);
      await probe();
      onInitialized?.();
    } catch {
      setStatus('unknown');
    } finally {
      setPending(false);
    }
  };

  if (!provider) {
    return null;
  }

  return (
    <div
      data-testid="binding-bucket-readiness"
      className="grid gap-2 rounded-md border border-neutral-200 bg-neutral-50 p-3 text-xs dark:border-neutral-700 dark:bg-neutral-800"
    >
      <div className="flex flex-wrap items-center gap-2">
        <span className="font-medium text-neutral-600 dark:text-neutral-300">
          {t('bindingsBucketTarget')}
        </span>
        <span className="font-mono text-neutral-800 dark:text-neutral-100">
          {bucket || provider.bucket || '--'}
        </span>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        {status === 'checking' ? (
          <span className="inline-flex items-center gap-1.5 text-neutral-500">
            <LoaderCircle aria-hidden="true" className="animate-spin" size={13} />
            {t('bindingsBucketChecking')}
          </span>
        ) : status === 'exists' ? (
          <span className="inline-flex items-center gap-1.5 text-emerald-700 dark:text-emerald-300">
            <CheckCircle2 aria-hidden="true" size={13} />
            {t('bindingsBucketExists')}
          </span>
        ) : status === 'missing' ? (
          <span className="inline-flex items-center gap-1.5 text-amber-700 dark:text-amber-300">
            <CircleAlert aria-hidden="true" size={13} />
            {t('bindingsBucketMissing')}
          </span>
        ) : status === 'unknown' ? (
          <span className="inline-flex items-center gap-1.5 text-amber-700 dark:text-amber-300">
            <CircleAlert aria-hidden="true" size={13} />
            {t('bindingsBucketUnknown')}
          </span>
        ) : (
          <span className="text-neutral-500">{t('bindingsBucketUnchecked')}</span>
        )}
        <span className="flex-1" />
        <button
          type="button"
          className={SECONDARY_BUTTON_CLASS}
          disabled={pending || status === 'checking'}
          onClick={() => void probe()}
        >
          <RefreshCw aria-hidden="true" size={13} />
          {t('bindingsBucketCheck')}
        </button>
        {status === 'missing' && (
          <button
            type="button"
            className={SECONDARY_BUTTON_CLASS}
            disabled={pending}
            onClick={() => void initializeBucket()}
          >
            {t('bindingsBucketInitialize')}
          </button>
        )}
      </div>
      {!provider.credentialConfigured && (
        <p className="text-amber-700 dark:text-amber-300">{t('bindingsCredentialWarning')}</p>
      )}
    </div>
  );
}
