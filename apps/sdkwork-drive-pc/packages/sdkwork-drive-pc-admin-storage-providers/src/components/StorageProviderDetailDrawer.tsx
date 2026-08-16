import React, { useCallback, useEffect, useState } from 'react';
import { Check, File, Folder, X } from 'lucide-react';
import { ConfirmDialog, OperationDrawer } from '@sdkwork/ui-pc-react';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import type { StorageProviderAdminService } from '../services/storageProviderAdminService';
import type { StorageProviderBindingView, StorageProviderBucketListItemView, StorageProviderBucketView, StorageProviderCapabilitiesView, StorageProviderView } from '../types/storageProviderAdminTypes';
import { formatDriveBytes } from 'sdkwork-drive-pc-commons';
import { getProviderKindMeta, HEALTH_STATUS_CONFIG } from '../utils/providerKindConfig';
import { formatMutationError } from '../utils/mutationError';
import { SECONDARY_BUTTON_CLASS, PRIMARY_BUTTON_CLASS, BADGE_BASE_CLASS } from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

type DrawerTab = 'overview' | 'buckets' | 'files';

interface Props {
  provider: StorageProviderView; providers: StorageProviderView[]; adminStorageSdkClient: DriveAdminStorageSdkClient; service: StorageProviderAdminService; pending: boolean;
  onClose: () => void; onTestProvider: (id: string) => void; onActivateProvider: (id: string) => void; onDeactivateProvider: (id: string) => void;
  onSetDefaultBinding: (providerId: string, spaceId?: string) => void; onDeleteDefaultBinding: (spaceId?: string) => void; onRotateCredential: (providerId: string, credentialRef: string) => void;
}

export function StorageProviderDetailDrawer({ provider, providers, service, pending, onClose, onTestProvider, onActivateProvider, onDeactivateProvider, onSetDefaultBinding, onDeleteDefaultBinding }: Props) {
  const { t } = useTranslation();
  const [tab, setTab] = useState<DrawerTab>('overview');
  const meta = getProviderKindMeta(provider.providerKind);
  const health = HEALTH_STATUS_CONFIG[provider.healthStatus ?? 'unknown'];
  const [capabilities, setCapabilities] = useState<StorageProviderCapabilitiesView | undefined>();
  const [bucket, setBucket] = useState<StorageProviderBucketView | undefined>();
  const [binding, setBinding] = useState<StorageProviderBindingView | undefined>();
  const [bindingProviderId, setBindingProviderId] = useState('');
  const [bindingSpaceId, setBindingSpaceId] = useState('');
  const [loading, setLoading] = useState(false);
  const [diagnosticsLoaded, setDiagnosticsLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [bucketExists, setBucketExists] = useState<boolean | null>(null);
  const [buckets, setBuckets] = useState<StorageProviderBucketListItemView[]>([]);
  const [objects, setObjects] = useState<Array<{ key: string; sizeBytes: number; lastModified?: string; isFolder: boolean }>>([]);
  const [currentPrefix, setCurrentPrefix] = useState('');
  const [pageToken, setPageToken] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<{ kind: 'bucket' } | { key: string; kind: 'object' } | null>(null);

  useEffect(() => { service.getDefaultBinding(undefined).then(setBinding).catch(() => {}); }, [service]);

  const loadCapabilities = useCallback(async () => {
    setLoading(true);
    try {
      setCapabilities(await service.getCapabilities(provider.id));
    } catch (e) {
      setError(formatMutationError(e, t('noticeCapabilitiesFailed')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, service]);

  const headBucket = useCallback(async () => {
    setLoading(true);
    try {
      const r = await service.headBucket(provider.id);
      setBucket(r);
      setBucketExists(r.exists);
    } catch (e) {
      setError(formatMutationError(e, t('noticeBucketFailed')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, service, t]);

  const runDiagnostics = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [caps, bucketResult] = await Promise.all([
        service.getCapabilities(provider.id),
        service.headBucket(provider.id),
      ]);
      setCapabilities(caps);
      setBucket(bucketResult);
      setBucketExists(bucketResult.exists);
      setDiagnosticsLoaded(true);
    } catch (e) {
      setError(formatMutationError(e, t('noticeCapabilitiesFailed')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, service]);

  useEffect(() => {
    setDiagnosticsLoaded(false);
    setCapabilities(undefined);
    setBucket(undefined);
    setBucketExists(null);
    setError(null);
    if (tab === 'overview') {
      void runDiagnostics();
    }
  }, [provider.id, tab, runDiagnostics]);

  const loadBuckets = useCallback(async () => { setLoading(true); try { setBuckets(await service.listBuckets(provider.id)); } catch (e) { setError(formatMutationError(e, t('errorLoadBuckets'))); } finally { setLoading(false); } }, [provider.id, service, t]);
  const createBucket = useCallback(async () => { setLoading(true); try { await service.createBucket(provider.id); setBucketExists(true); await loadBuckets(); } catch (e) { setError(formatMutationError(e, t('errorCreateBucket'))); } finally { setLoading(false); } }, [provider.id, service, loadBuckets, t]);
  const deleteBucket = useCallback(async () => { setDeleteTarget(null); setLoading(true); try { await service.deleteBucket(provider.id); setBucketExists(false); setBuckets([]); } catch (e) { setError(formatMutationError(e, t('errorDeleteBucket'))); } finally { setLoading(false); } }, [provider.id, service, t]);
  const loadObjects = useCallback(async (prefix: string, token?: string) => { setLoading(true); try { const result = await service.listObjects(provider.id, { prefix, pageToken: token }); if (token) setObjects((prev) => [...prev, ...result.items]); else setObjects(result.items); setPageToken(result.nextPageToken || null); setHasMore(result.hasMore); setCurrentPrefix(prefix); } catch (e) { setError(formatMutationError(e, t('errorLoadObjects'))); } finally { setLoading(false); } }, [provider.id, service, t]);
  const deleteObject = useCallback(async (key: string) => { setDeleteTarget(null); setLoading(true); try { await service.deleteObject(provider.id, key); await loadObjects(currentPrefix); } catch (e) { setError(formatMutationError(e, t('errorDeleteObject'))); } finally { setLoading(false); } }, [provider.id, currentPrefix, service, loadObjects, t]);

  const tabClass = (d: DrawerTab) => `px-3 py-2 text-xs font-medium border-b-2 transition-colors ${tab === d ? 'border-blue-600 text-blue-600' : 'border-transparent text-neutral-500 hover:text-neutral-700'}`;

  const readinessChecks = [
    {
      label: t('checkCredential'),
      ok: provider.credentialConfigured,
      detail: provider.credentialConfigured ? t('configured') : t('credentialMissing'),
    },
    {
      label: t('checkBucket'),
      ok: bucketExists === true,
      detail: bucketExists === null ? t('checkPending') : bucketExists ? t('exists') : t('doesNotExist'),
    },
    {
      label: t('checkConnectivity'),
      ok: bucket?.exists === true && provider.status === 'active',
      detail: provider.status === 'active' ? t('active') : t('inactive'),
    },
  ];

  return (
    <>
      <OperationDrawer
        actions={(
          <div className="flex flex-wrap items-center justify-end gap-2">
            <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={pending} onClick={() => onTestProvider(provider.id)}>{t('test')}</button>
            {provider.status === 'active' ? <button type="button" className="rounded-md border border-amber-300 px-3 py-1.5 text-xs font-medium text-amber-700 hover:bg-amber-50" disabled={pending} onClick={() => onDeactivateProvider(provider.id)}>{t('disable')}</button>
              : <button type="button" className="rounded-md border border-emerald-300 px-3 py-1.5 text-xs font-medium text-emerald-700 hover:bg-emerald-50" disabled={pending} onClick={() => onActivateProvider(provider.id)}>{t('enable')}</button>}
          </div>
        )}
        badge={<span className={`${BADGE_BASE_CLASS} ${health.bgClass} ${health.textClass}`}><span className={`h-1.5 w-1.5 rounded-full ${health.dotClass}`} />{t(`health${provider.healthStatus ? provider.healthStatus.charAt(0).toUpperCase() + provider.healthStatus.slice(1) : 'Unknown'}`)}</span>}
        description={<span className="font-mono text-xs">{provider.id}</span>}
        onOpenChange={(open) => { if (!open && !loading) onClose(); }}
        open
        size="lg"
        slotProps={{ body: { className: 'p-0 xl:p-0' } }}
        title={provider.displayName}
      >

        <div className="sticky top-0 z-10 flex border-b border-neutral-200 bg-white px-5 dark:border-neutral-800 dark:bg-neutral-900" role="tablist">
          <button aria-selected={tab === 'overview'} className={tabClass('overview')} onClick={() => setTab('overview')} role="tab" type="button">{t('overview')}</button>
          <button aria-selected={tab === 'buckets'} className={tabClass('buckets')} onClick={() => { setTab('buckets'); void loadBuckets(); }} role="tab" type="button">{t('buckets')}</button>
          <button aria-selected={tab === 'files'} className={tabClass('files')} onClick={() => { setTab('files'); void loadObjects(''); }} role="tab" type="button">{t('files')}</button>
        </div>

        {error && <div className="mx-5 mt-3 flex items-center gap-2 rounded-md bg-red-50 px-3 py-2 text-xs text-red-700 dark:bg-red-950/30 dark:text-red-300" role="alert">{error}<button aria-label={t('cancel')} className="ml-auto rounded p-1 hover:bg-red-100 dark:hover:bg-red-900/40" onClick={() => setError(null)} title={t('cancel')} type="button"><X aria-hidden="true" size={14} /></button></div>}

        <div className="p-5">
          {tab === 'overview' && (
            <div className="space-y-4">
              <div className="rounded-lg border border-neutral-200 p-4 dark:border-neutral-700">
                <div className="flex items-center justify-between gap-2">
                  <h3 className="text-xs font-semibold">{t('readinessTitle')}</h3>
                  <div className="flex items-center gap-2">
                    {meta.credentialFields?.consoleUrl && (
                      <a
                        href={meta.credentialFields.consoleUrl}
                        target="_blank"
                        rel="noreferrer"
                        className="text-[11px] font-medium text-blue-600 hover:underline dark:text-blue-400"
                      >
                        {t('openClawConsole')} ↗
                      </a>
                    )}
                    <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading} onClick={runDiagnostics}>
                      {loading ? t('diagnosticsRunning') : t('runDiagnostics')}
                    </button>
                  </div>
                </div>
                <div className="mt-3 space-y-2">
                  {readinessChecks.map((check) => (
                    <div key={check.label} className="flex items-center justify-between rounded-md bg-neutral-50 px-3 py-2 text-xs dark:bg-neutral-800/60">
                      <span className="font-medium text-neutral-700 dark:text-neutral-200">{check.label}</span>
                      <span className={`flex items-center gap-1.5 ${check.ok ? 'text-emerald-600 dark:text-emerald-400' : 'text-amber-600 dark:text-amber-400'}`}>
                        {check.ok ? <Check aria-hidden="true" size={14} /> : '!'} {check.detail}
                      </span>
                    </div>
                  ))}
                </div>
                {diagnosticsLoaded && bucket && (
                  <div className={`mt-3 flex items-center gap-2 rounded p-2 text-xs ${bucket.exists ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300' : 'bg-red-50 text-red-700 dark:bg-red-950/30 dark:text-red-300'}`}>
                    {bucket.exists ? <Check aria-hidden="true" size={14} /> : <X aria-hidden="true" size={14} />}{bucket.exists ? t('bucketReachable') : t('bucketUnreachable')}
                  </div>
                )}
              </div>

              <div className="grid grid-cols-2 gap-3">
                <InfoCard label={t('endpoint')} value={provider.endpointUrl} mono wide />
                <InfoCard label={t('bucket')} value={provider.bucket} />
                {provider.region && <InfoCard label={t('region')} value={provider.region} />}
                <InfoCard label={t('kind')} value={meta.label} />
                <InfoCard label={t('pathStyleLabel')} value={provider.pathStyle ? t('yes') : t('no')} />
                <InfoCard label={t('strictTlsLabel')} value={provider.strictTls ? t('yes') : t('no')} />
                <InfoCard label={t('credentialRef')} value={provider.credentialConfigured ? t('configured') : t('credentialMissing')} />
                {provider.serverSideEncryptionMode && <InfoCard label={t('sse')} value={provider.serverSideEncryptionMode} />}
                {provider.defaultStorageClass && <InfoCard label={t('storageClass')} value={provider.defaultStorageClass} />}
              </div>

              <div className="rounded-lg border border-neutral-200 p-4 dark:border-neutral-700">
                <h3 className="text-xs font-semibold">{t('defaultBinding')}</h3>
                <p className="mt-0.5 text-[11px] text-neutral-500">{t('bindingDesc')}</p>
                <div className="mt-2 text-xs text-neutral-600 dark:text-neutral-300">{t('currentBinding')} {binding?.providerId ? `${binding.providerId}${binding.spaceId ? ` → ${binding.spaceId}` : ` (${t('tenantDefault')})`}` : t('notConfigured')}</div>
                <div className="mt-2 flex gap-2">
                  <select value={bindingProviderId} onChange={(e) => setBindingProviderId(e.target.value)} className="h-8 rounded-md border border-neutral-300 px-2 text-xs dark:border-neutral-600 dark:bg-neutral-800"><option value="">{t('selectProvider')}</option>{providers.filter((p) => p.status === 'active').map((p) => <option key={p.id} value={p.id}>{p.displayName}</option>)}</select>
                  <input value={bindingSpaceId} onChange={(e) => setBindingSpaceId(e.target.value)} className="h-8 flex-1 rounded-md border border-neutral-300 px-2 text-xs dark:border-neutral-600 dark:bg-neutral-800" placeholder={t('spaceIdOptional')} />
                  <button type="button" className={PRIMARY_BUTTON_CLASS} disabled={!bindingProviderId || pending} onClick={() => onSetDefaultBinding(bindingProviderId, bindingSpaceId || undefined)}>{t('set')}</button>
                  {binding && <button type="button" className="text-xs text-red-600" onClick={() => onDeleteDefaultBinding()}>{t('clear')}</button>}
                </div>
              </div>

              <div className="rounded-lg border border-neutral-200 p-4 dark:border-neutral-700">
                <div className="flex items-center justify-between">
                  <h3 className="text-xs font-semibold">{t('diagnostics')}</h3>
                  <div className="flex gap-2"><button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading} onClick={loadCapabilities}>{t('capabilities')}</button><button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading} onClick={headBucket}>{t('bucketCheck')}</button></div>
                </div>
                {capabilities && (
                  <div className="mt-2 grid grid-cols-2 gap-2">
                    <CapCard label={t('capMultipart')} ok={capabilities.supportsMultipartUpload} desc={t('capLargeFile')} />
                    <CapCard label={t('capPresignedDownload')} ok={capabilities.supportsPresignedDownload} desc={t('capDirectDownload')} />
                    <CapCard label={t('capSse')} ok={capabilities.supportsServerSideEncryption} desc={capabilities.supportedServerSideEncryptionModes.join(', ') || t('capNotAvailable')} />
                    <CapCard label={t('capStorageClasses')} ok={capabilities.supportsStorageClass} desc={capabilities.supportedStorageClasses.slice(0, 3).join(', ') || t('capNotAvailable')} />
                    <CapCard label={t('capCredentialRotation')} ok={capabilities.supportsCredentialRotation} desc={t('capLiveUpdate')} />
                  </div>
                )}
              </div>
            </div>
          )}

          {tab === 'buckets' && (
            <div className="space-y-3">
              <div className="rounded-md bg-neutral-50 p-3 dark:bg-neutral-800"><div className="text-xs text-neutral-500">{t('configuredBucket')}</div><div className="text-sm font-medium">{provider.bucket}</div>{bucketExists !== null && <div className={`mt-1 text-xs ${bucketExists ? 'text-emerald-600' : 'text-red-600'}`}>{bucketExists ? `✓ ${t('exists')}` : `✕ ${t('doesNotExist')}`}</div>}</div>
              <div className="flex flex-wrap gap-2">
                <button onClick={headBucket} disabled={loading} className={SECONDARY_BUTTON_CLASS}>{t('checkExists')}</button>
                <button onClick={createBucket} disabled={loading} className="rounded-md bg-emerald-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-emerald-700 disabled:opacity-50">{t('createBucket')}</button>
                <button onClick={() => setDeleteTarget({ kind: 'bucket' })} disabled={loading} className="rounded-md bg-red-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-red-700 disabled:opacity-50">{t('deleteBucket')}</button>
                <button onClick={loadBuckets} disabled={loading} className={SECONDARY_BUTTON_CLASS}>{t('listAll')}</button>
              </div>
              {buckets.length > 0 && <div className="rounded-md border dark:border-neutral-700">{buckets.map((b) => <div key={b.bucket} className={`flex items-center justify-between border-b px-3 py-2 text-xs dark:border-neutral-800 ${b.configured ? 'bg-blue-50 dark:bg-blue-950/20' : ''}`}><span>{b.bucket}{b.configured && <span className="ml-2 text-blue-600">({t('configured')})</span>}</span>{b.creationDate && <span className="text-neutral-400">{b.creationDate}</span>}</div>)}</div>}
            </div>
          )}

          {tab === 'files' && (
            <div className="space-y-3">
              <div className="flex items-center gap-2">
                <button onClick={() => loadObjects('')} disabled={loading} className={SECONDARY_BUTTON_CLASS}>{t('root')}</button>
                {currentPrefix && <button onClick={() => { const parts = currentPrefix.split('/').filter(Boolean); parts.pop(); loadObjects(parts.length > 0 ? parts.join('/') + '/' : ''); }} disabled={loading} className={SECONDARY_BUTTON_CLASS}>{t('up')}</button>}
                <span className="font-mono text-xs text-neutral-500">/{currentPrefix}</span>
              </div>
              <div className="rounded-md border dark:border-neutral-700 max-h-[60vh] overflow-y-auto">
                <div className="grid grid-cols-[1fr_80px_120px_60px] gap-2 bg-neutral-50 px-3 py-2 text-[10px] font-semibold uppercase text-neutral-500 dark:bg-neutral-800"><span>{t('nameHeader')}</span><span className="text-right">{t('sizeHeader')}</span><span className="text-right">{t('modifiedHeader')}</span><span className="text-right">{t('actHeader')}</span></div>
                {objects.length === 0 && !loading && <div className="px-3 py-6 text-center text-xs text-neutral-400">{t('empty')}</div>}
                {objects.map((obj) => (
                  <div key={obj.key} className="grid grid-cols-[1fr_80px_120px_60px] gap-2 border-t px-3 py-1.5 text-xs hover:bg-neutral-50 dark:border-neutral-800 dark:hover:bg-neutral-800">
                    <span className="truncate">{obj.isFolder ? <button onClick={() => loadObjects(obj.key)} className="inline-flex items-center gap-1 text-blue-600 hover:underline" type="button"><Folder aria-hidden="true" size={14} />{obj.key.split('/').filter(Boolean).pop()}/</button> : <span className="inline-flex items-center gap-1"><File aria-hidden="true" size={14} />{obj.key.split('/').pop()}</span>}</span>
                    <span className="text-right text-neutral-500">{obj.isFolder ? '-' : formatDriveBytes(obj.sizeBytes)}</span>
                    <span className="text-right text-neutral-400">{obj.lastModified || '-'}</span>
                    <span className="text-right">{!obj.isFolder && <button onClick={() => setDeleteTarget({ key: obj.key, kind: 'object' })} className="text-red-600 hover:text-red-800">{t('del')}</button>}</span>
                  </div>
                ))}
                {hasMore && <div className="border-t px-3 py-2 dark:border-neutral-800"><button onClick={() => loadObjects(currentPrefix, pageToken || undefined)} disabled={loading} className="text-xs text-blue-600 hover:underline">{t('loadMore')}</button></div>}
              </div>
            </div>
          )}
        </div>
      </OperationDrawer>
    <ConfirmDialog
      cancelLabel={t('cancel')}
      closeOnConfirm={false}
      confirmLabel={deleteTarget?.kind === 'bucket' ? t('deleteBucket') : t('del')}
      confirmLoading={loading}
      description={deleteTarget?.kind === 'bucket'
        ? t('deleteBucketConfirm', { bucket: provider.bucket })
        : deleteTarget?.kind === 'object'
          ? t('deleteObjectConfirm', { key: deleteTarget.key })
          : undefined}
      onConfirm={() => {
        if (deleteTarget?.kind === 'bucket') void deleteBucket();
        if (deleteTarget?.kind === 'object') void deleteObject(deleteTarget.key);
      }}
      onOpenChange={(open) => { if (!open && !loading) setDeleteTarget(null); }}
      open={Boolean(deleteTarget)}
      title={deleteTarget?.kind === 'bucket' ? t('deleteBucket') : t('del')}
      tone="danger"
    />
    </>
  );
}

function InfoCard({ label, value, mono, wide }: { label: string; value: string; mono?: boolean; wide?: boolean }) {
  return (
    <div className={`rounded-md border border-neutral-100 p-2.5 dark:border-neutral-800 ${wide ? 'col-span-2' : ''}`}>
      <div className="text-[10px] text-neutral-500">{label}</div>
      <div className={`mt-0.5 text-xs font-medium leading-relaxed break-all ${mono ? 'font-mono' : ''}`}>{value}</div>
    </div>
  );
}

function CapCard({ label, ok, desc }: { label: string; ok: boolean; desc?: string }) {
  return (
    <div className={`rounded border p-2 text-xs ${ok ? 'border-emerald-200 bg-emerald-50/50' : 'border-neutral-200 bg-neutral-50 dark:border-neutral-800 dark:bg-neutral-900/50'}`}>
      <div className="flex items-center gap-1">
        {ok ? <Check aria-hidden="true" className="text-emerald-500" size={12} />
          : <X aria-hidden="true" className="text-neutral-400" size={12} />}
        <span className="font-medium">{label}</span>
      </div>
      {desc && <p className="mt-0.5 text-[10px] text-neutral-500">{desc}</p>}
    </div>
  );
}
