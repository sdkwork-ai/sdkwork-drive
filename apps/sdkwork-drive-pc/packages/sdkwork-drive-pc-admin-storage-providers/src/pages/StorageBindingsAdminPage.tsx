import React, { useEffect, useMemo, useState } from 'react';
import {
  CircleAlert,
  LoaderCircle,
  RefreshCw,
  X,
} from 'lucide-react';
import {
  Drawer,
  DrawerBody,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
} from '@sdkwork/ui-pc-react';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import {
  createStorageProviderAdminService,
  type StorageProviderAdminService,
} from '../services/storageProviderAdminService';
import type { StorageProviderBindingView, StorageProviderView } from '../types/storageProviderAdminTypes';
import { SPACE_TYPES, getSpaceTypeMeta, resolveSpaceTypeDescription, resolveSpaceTypeLabel } from '../utils/spaceTypeConfig';
import { getProviderKindMeta } from '../utils/providerKindConfig';
import {
  PRIMARY_BUTTON_CLASS,
  SELECT_CLASS,
  CARD_CLASS,
  BADGE_BASE_CLASS,
  ICON_BUTTON_CLASS,
  INPUT_CLASS,
  SECONDARY_BUTTON_CLASS,
} from '../utils/uiPrimitives';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { useTranslation } from '../hooks/useTranslation';

interface StorageBindingsAdminPageProps {
  adminStorageSdkClient: DriveAdminStorageSdkClient;
  getSession: () => SessionSnapshot;
}

interface SpaceTypeBindingRow {
  spaceType: string;
  providerId: string;
  bucket: string;
  storageRootPrefix?: string;
  bindingId?: string;
  configured: boolean;
}

function defaultSpaceTypeRootPrefix(tenantId: string, spaceType: string): string {
  return `sdkwork-drive/v1/tenants/${tenantId}/space-types/${spaceType}`;
}

export function StorageBindingsAdminPage({
  adminStorageSdkClient,
  getSession,
}: StorageBindingsAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo<StorageProviderAdminService>(
    () => createStorageProviderAdminService({ adminStorageSdkClient, getSession }),
    [adminStorageSdkClient, getSession],
  );

  const tenantId = getSession().context?.tenantId ?? '';

  const [providers, setProviders] = useState<StorageProviderView[]>([]);
  const [bindings, setBindings] = useState<StorageProviderBindingView[]>([]);
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [notice, setNotice] = useState<{ type: 'success' | 'error'; messageKey: string; params?: Record<string, string> } | null>(null);
  const [editingType, setEditingType] = useState<string | null>(null);
  const [editProviderId, setEditProviderId] = useState('');
  const [editRootPrefix, setEditRootPrefix] = useState('');
  const [useCustomPrefix, setUseCustomPrefix] = useState(false);
  const [clearTarget, setClearTarget] = useState<string | null>(null);
  const [bindingFilter, setBindingFilter] = useState<'all' | 'bound' | 'unbound' | 'system' | 'user'>('all');
  const [tenantBinding, setTenantBinding] = useState<StorageProviderBindingView | undefined>();
  const [tenantProviderId, setTenantProviderId] = useState('');
  const [editingTenantDefault, setEditingTenantDefault] = useState(false);

  const activeProviders = providers.filter((p) => p.status === 'active');

  const load = (signal?: AbortSignal) => {
    setLoading(true);
    Promise.all([
      service.listProviders({ signal }),
      service.listBindings({ signal }),
      service.getDefaultBinding(undefined, { signal }),
    ])
      .then(([p, b, tenantDefault]) => {
        setProviders(p);
        setBindings(b);
        setTenantBinding(tenantDefault?.providerId ? tenantDefault : undefined);
        setTenantProviderId(tenantDefault?.providerId ?? '');
      })
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) {
          setNotice({ type: 'error', messageKey: 'bindingsNoticeLoadFailed' });
        }
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    const c = new AbortController();
    load(c.signal);
    return () => c.abort();
  }, [service]);

  const spaceTypeBindings: SpaceTypeBindingRow[] = useMemo(() => {
    return SPACE_TYPES.map((st) => {
      const binding = bindings.find(
        (b) => b.bindingScope === 'space_type' && b.purpose === st.value && b.lifecycleStatus === 'active',
      );
      if (binding) {
        return {
          spaceType: st.value,
          providerId: binding.providerId,
          bucket: binding.storageProvider?.bucket ?? '',
          storageRootPrefix: binding.storageRootPrefix,
          bindingId: binding.id,
          configured: true,
        };
      }
      return {
        spaceType: st.value,
        providerId: '',
        bucket: '',
        configured: false,
      };
    });
  }, [bindings]);

  const boundCount = spaceTypeBindings.filter((b) => b.configured).length;

  const filteredBindings = spaceTypeBindings.filter((binding) => {
    const stMeta = getSpaceTypeMeta(binding.spaceType);
    if (bindingFilter === 'bound') return binding.configured;
    if (bindingFilter === 'unbound') return !binding.configured;
    if (bindingFilter === 'system') return stMeta.isSystem;
    if (bindingFilter === 'user') return !stMeta.isSystem;
    return true;
  });

  const spaceTypeLabel = (spaceType: string) => resolveSpaceTypeLabel(getSpaceTypeMeta(spaceType), t);
  const spaceTypeDescription = (spaceType: string) => resolveSpaceTypeDescription(getSpaceTypeMeta(spaceType), t);

  const selectedProvider = providers.find((p) => p.id === editProviderId);
  const tenantDefaultProvider = providers.find((p) => p.id === (editingTenantDefault ? tenantProviderId : tenantBinding?.providerId));

  const handleEdit = (spaceType: string) => {
    const existing = spaceTypeBindings.find((b) => b.spaceType === spaceType);
    setEditingType(spaceType);
    setEditProviderId(existing?.providerId ?? (activeProviders[0]?.id ?? ''));
    const defaultPrefix = defaultSpaceTypeRootPrefix(tenantId, spaceType);
    const existingPrefix = existing?.storageRootPrefix;
    setUseCustomPrefix(Boolean(existingPrefix && existingPrefix !== defaultPrefix));
    setEditRootPrefix(existingPrefix ?? defaultPrefix);
  };

  const handleSave = async () => {
    if (!editingType || !editProviderId) return;
    setPending(true);
    setNotice(null);
    try {
      const defaultPrefix = defaultSpaceTypeRootPrefix(tenantId, editingType);
      await service.setSpaceTypeBinding({
        spaceType: editingType,
        providerId: editProviderId,
        storageRootPrefix: useCustomPrefix ? editRootPrefix.trim() : undefined,
      });
      setNotice({
        type: 'success',
        messageKey: 'bindingsNoticeSaved',
        params: { label: spaceTypeLabel(editingType) },
      });
      setEditingType(null);
      load();
    } catch (err) {
      setNotice({
        type: 'error',
        messageKey: 'bindingsNoticeSaveFailed',
        params: { detail: err instanceof Error ? err.message : '' },
      });
    } finally {
      setPending(false);
    }
  };

  const handleClear = async (spaceType: string) => {
    setPending(true);
    setNotice(null);
    try {
      await service.deleteSpaceTypeBinding(spaceType);
      setNotice({
        type: 'success',
        messageKey: 'bindingsNoticeCleared',
        params: { label: spaceTypeLabel(spaceType) },
      });
      setClearTarget(null);
      load();
    } catch (err) {
      setNotice({
        type: 'error',
        messageKey: 'bindingsNoticeClearFailed',
        params: { detail: err instanceof Error ? err.message : '' },
      });
    } finally {
      setPending(false);
    }
  };

  const handleSaveTenantDefault = async () => {
    if (!tenantProviderId) return;
    setPending(true);
    setNotice(null);
    try {
      await service.setDefaultBinding({ providerId: tenantProviderId });
      setNotice({ type: 'success', messageKey: 'bindingsTenantDefaultSaved' });
      setEditingTenantDefault(false);
      load();
    } catch (err) {
      setNotice({
        type: 'error',
        messageKey: 'bindingsNoticeSaveFailed',
        params: { detail: err instanceof Error ? err.message : '' },
      });
    } finally {
      setPending(false);
    }
  };

  const handleClearTenantDefault = async () => {
    setPending(true);
    setNotice(null);
    try {
      await service.deleteDefaultBinding();
      setNotice({ type: 'success', messageKey: 'bindingsTenantDefaultCleared' });
      setEditingTenantDefault(false);
      load();
    } catch (err) {
      setNotice({
        type: 'error',
        messageKey: 'bindingsNoticeClearFailed',
        params: { detail: err instanceof Error ? err.message : '' },
      });
    } finally {
      setPending(false);
    }
  };

  return (
    <main className="flex h-full flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <div aria-label={t('bindingsPageTitle')} className="flex shrink-0 items-center justify-between gap-3 px-4 pt-4 sm:px-6 sm:pt-6">
        <span className={`${BADGE_BASE_CLASS} bg-blue-50 text-blue-700 dark:bg-blue-950/30 dark:text-blue-300`}>{t('bindingsSummary', { bound: boundCount, total: SPACE_TYPES.length })}</span>
        <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading} onClick={() => load()}>
          <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
          {t('refresh')}
        </button>
      </div>

      <div className="flex-1 overflow-auto p-4 sm:p-6">
        {notice && (
          <div className={`mb-4 flex items-center gap-3 rounded-lg border px-4 py-3 text-sm ${
            notice.type === 'success'
              ? 'border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950/30 dark:text-emerald-200'
              : 'border-red-200 bg-red-50 text-red-800 dark:border-red-900 dark:bg-red-950/30 dark:text-red-200'
          }`}>
            <span className="flex-1">{t(notice.messageKey, notice.params)}</span>
            <button type="button" className={ICON_BUTTON_CLASS} aria-label={t('dismiss')} title={t('dismiss')} onClick={() => setNotice(null)}><X aria-hidden="true" size={15} /></button>
          </div>
        )}

        {activeProviders.length === 0 && !loading && (
          <div className="mb-4 flex items-start gap-3 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800 dark:border-amber-900 dark:bg-amber-950/30 dark:text-amber-200">
            <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
            <span>{t('bindingsNoActiveProviders')}</span>
          </div>
        )}

        {!loading && activeProviders.length > 0 && (
          <div className={`${CARD_CLASS} mb-4`}>
            <div className="border-b border-neutral-100 px-5 py-3 dark:border-neutral-800">
              <h3 className="text-sm font-semibold">{t('bindingsTenantDefaultTitle')}</h3>
              <p className="mt-0.5 text-[11px] text-neutral-500">{t('bindingsTenantDefaultDesc')}</p>
            </div>
            <div className="flex flex-wrap items-center gap-3 px-5 py-4">
              {tenantBinding?.providerId && tenantDefaultProvider ? (
                <>
                  <span className={`inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium ${getProviderKindMeta(tenantDefaultProvider.providerKind).bgClass} ${getProviderKindMeta(tenantDefaultProvider.providerKind).textClass}`}>
                    {getProviderKindMeta(tenantDefaultProvider.providerKind).icon} {tenantDefaultProvider.displayName}
                  </span>
                  <span className="font-mono text-xs text-neutral-500">{tenantDefaultProvider.bucket}</span>
                  <button type="button" className="text-xs font-medium text-blue-600" onClick={() => { setTenantProviderId(tenantBinding.providerId); setEditingTenantDefault(true); }}>
                    {t('bindingsChange')}
                  </button>
                  <button type="button" className="text-xs text-red-600" onClick={handleClearTenantDefault} disabled={pending}>
                    {t('bindingsTenantDefaultClear')}
                  </button>
                </>
              ) : (
                <>
                  <span className="text-xs text-neutral-500">{t('bindingsTenantDefaultUnset')}</span>
                  <button
                    type="button"
                    className={PRIMARY_BUTTON_CLASS}
                    onClick={() => {
                      setTenantProviderId(activeProviders[0]?.id ?? '');
                      setEditingTenantDefault(true);
                    }}
                  >
                    {t('bindingsTenantDefaultSet')}
                  </button>
                </>
              )}
            </div>
          </div>
        )}

        {loading ? (
          <div className="flex min-h-[360px] items-center justify-center rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900">
            <div className="flex items-center gap-3 text-sm text-neutral-500">
              <LoaderCircle aria-hidden="true" className="animate-spin" size={19} />
              {t('bindingsLoading')}
            </div>
          </div>
        ) : (
          <>
            <div className="mb-3 flex flex-wrap items-center gap-2">
              {(
                [
                  ['all', 'bindingsFilterAll'],
                  ['bound', 'bindingsFilterBound'],
                  ['unbound', 'bindingsFilterUnbound'],
                  ['system', 'bindingsFilterSystem'],
                  ['user', 'bindingsFilterUser'],
                ] as const
              ).map(([filter, labelKey]) => (
                <button
                  key={filter}
                  type="button"
                  onClick={() => setBindingFilter(filter)}
                  className={`rounded-md px-2.5 py-1 text-xs font-medium transition-colors ${
                    bindingFilter === filter
                      ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300'
                      : 'bg-white text-neutral-600 hover:bg-neutral-100 dark:bg-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-800'
                  }`}
                >
                  {t(labelKey)}
                </button>
              ))}
            </div>
          <div className={`${CARD_CLASS} overflow-x-auto`}>
            <table className="w-full min-w-[1080px] text-left text-sm">
              <thead className="border-b border-neutral-200 bg-neutral-50 text-xs font-medium text-neutral-500 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-400">
                <tr>
                  <th className="px-5 py-3 font-semibold">{t('bindingsColSpaceType')}</th>
                  <th className="px-5 py-3 font-semibold">{t('bindingsColDescription')}</th>
                  <th className="px-5 py-3 font-semibold">{t('bindingsColProvider')}</th>
                  <th className="px-5 py-3 font-semibold">{t('bindingsColBucket')}</th>
                  <th className="px-5 py-3 font-semibold">{t('bindingsColStatus')}</th>
                  <th className="px-5 py-3 text-right font-semibold">{t('colActions')}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-100 dark:divide-neutral-800">
                {filteredBindings.map((binding) => {
                  const stMeta = getSpaceTypeMeta(binding.spaceType);
                  const SpaceTypeIcon = stMeta.icon;
                  const provider = providers.find((p) => p.id === binding.providerId);
                  const providerMeta = provider ? getProviderKindMeta(provider.providerKind) : null;
                  const stLabel = spaceTypeLabel(binding.spaceType);
                  const stDesc = spaceTypeDescription(binding.spaceType);

                  return (
                    <tr key={binding.spaceType} className="hover:bg-neutral-50 dark:hover:bg-neutral-800/50">
                      <td className="px-5 py-3">
                        <div className="flex items-center gap-2.5">
                          <div className={`flex h-9 w-9 items-center justify-center rounded-lg ${stMeta.bgClass} ${stMeta.textClass}`}>
                            <SpaceTypeIcon aria-hidden="true" size={17} strokeWidth={1.8} />
                          </div>
                          <div>
                            <div className="text-sm font-semibold text-neutral-900 dark:text-neutral-100">{stLabel}</div>
                            <div className="font-mono text-[10px] text-neutral-500">{binding.spaceType}</div>
                          </div>
                        </div>
                      </td>

                      <td className="px-5 py-3">
                        <span className="text-xs text-neutral-600 dark:text-neutral-400">{stDesc}</span>
                        {stMeta.isSystem && (
                          <span className="ml-2 inline-flex items-center rounded bg-neutral-100 px-1.5 py-0.5 text-[9px] font-medium text-neutral-500 dark:bg-neutral-800 dark:text-neutral-400">
                            {t('bindingsSystemBadge')}
                          </span>
                        )}
                      </td>

                      <td className="px-5 py-3">
                        {binding.configured && provider && providerMeta ? (
                          <div className="flex items-center gap-1.5">
                            <span className={`inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-bold ${providerMeta.bgClass} ${providerMeta.textClass}`}>
                              {providerMeta.icon}
                            </span>
                            <span className="text-xs font-medium text-neutral-900 dark:text-neutral-100">{provider.displayName}</span>
                          </div>
                        ) : (
                          <span className="text-xs text-neutral-400">{t('bindingsNotAssigned')}</span>
                        )}
                      </td>

                      <td className="px-5 py-3">
                        {binding.configured ? (
                          <div>
                            <span className="font-mono text-xs text-neutral-700 dark:text-neutral-300">{binding.bucket || provider?.bucket || '--'}</span>
                            {binding.storageRootPrefix && (
                              <div className="mt-0.5 font-mono text-[10px] leading-relaxed break-all text-neutral-400">
                                {binding.storageRootPrefix}
                              </div>
                            )}
                          </div>
                        ) : (
                          <span className="text-xs text-neutral-400">--</span>
                        )}
                      </td>

                      <td className="px-5 py-3">
                        {binding.configured ? (
                          <span className={`${BADGE_BASE_CLASS} bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300`}>
                            <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                            {t('bindingsStatusBound')}
                          </span>
                        ) : (
                          <span className={`${BADGE_BASE_CLASS} bg-neutral-100 text-neutral-500 dark:bg-neutral-800 dark:text-neutral-400`}>
                            <span className="h-1.5 w-1.5 rounded-full bg-neutral-400" />
                            {t('bindingsStatusUnbound')}
                          </span>
                        )}
                      </td>

                      <td className="px-5 py-3 text-right">
                        <div className="flex items-center justify-end gap-1">
                            <button
                              type="button"
                              className="rounded-md px-2.5 py-1 text-xs font-medium text-blue-600 hover:bg-blue-50 dark:text-blue-400 dark:hover:bg-blue-950/30"
                              onClick={() => handleEdit(binding.spaceType)}
                              disabled={activeProviders.length === 0}
                            >
                              {binding.configured ? t('bindingsChange') : t('bindingsAssign')}
                            </button>
                            {binding.configured && (
                              <button
                                type="button"
                                className="rounded-md px-2.5 py-1 text-xs font-medium text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
                                onClick={() => setClearTarget(binding.spaceType)}
                                disabled={pending}
                              >
                                {t('clear')}
                              </button>
                            )}
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
          </>
        )}
      </div>

      <Drawer open={editingTenantDefault} onOpenChange={(open) => { if (!pending) setEditingTenantDefault(open); }}>
        <DrawerContent size="sm">
          <DrawerHeader>
            <DrawerTitle>{t('bindingsTenantDefaultTitle')}</DrawerTitle>
            <DrawerDescription>{t('bindingsTenantDefaultDesc')}</DrawerDescription>
          </DrawerHeader>
          <DrawerBody>
            <label className="grid gap-2 text-xs font-medium text-neutral-600 dark:text-neutral-300">
              {t('bindingsSelectProvider')}
              <select value={tenantProviderId} onChange={(event) => setTenantProviderId(event.target.value)} className={SELECT_CLASS}>
                {activeProviders.map((provider) => {
                  const meta = getProviderKindMeta(provider.providerKind);
                  return <option key={provider.id} value={provider.id}>[{meta.shortLabel}] {provider.displayName}</option>;
                })}
              </select>
            </label>
          </DrawerBody>
          <DrawerFooter>
            <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={pending} onClick={() => setEditingTenantDefault(false)}>{t('cancel')}</button>
            <button type="button" className={PRIMARY_BUTTON_CLASS} disabled={pending || !tenantProviderId} onClick={() => void handleSaveTenantDefault()}>{pending ? t('saving') : t('save')}</button>
          </DrawerFooter>
        </DrawerContent>
      </Drawer>

      <Drawer open={editingType !== null} onOpenChange={(open) => { if (!open && !pending) setEditingType(null); }}>
        <DrawerContent size="md">
          <DrawerHeader>
            <DrawerTitle>{editingType ? spaceTypeLabel(editingType) : t('bindingsAssign')}</DrawerTitle>
            <DrawerDescription>{t('bindingsPageDescription')}</DrawerDescription>
          </DrawerHeader>
          <DrawerBody className="grid content-start gap-5">
            <label className="grid gap-2 text-xs font-medium text-neutral-600 dark:text-neutral-300">
              {t('bindingsSelectProvider')}
              <select value={editProviderId} onChange={(event) => setEditProviderId(event.target.value)} className={SELECT_CLASS}>
                {activeProviders.map((provider) => {
                  const meta = getProviderKindMeta(provider.providerKind);
                  return <option key={provider.id} value={provider.id}>[{meta.shortLabel}] {provider.displayName}</option>;
                })}
              </select>
            </label>
            {selectedProvider ? (
              <div className="rounded-md border border-neutral-200 bg-neutral-50 p-3 text-xs text-neutral-500 dark:border-neutral-700 dark:bg-neutral-800">
                <div className="break-all font-mono">{selectedProvider.endpointUrl}</div>
                <div className="mt-1">{selectedProvider.region ? `${selectedProvider.region} · ` : ''}{selectedProvider.credentialConfigured ? t('configured') : t('credentialMissing')}</div>
                {!selectedProvider.credentialConfigured ? <div className="mt-2 text-amber-600 dark:text-amber-400">{t('bindingsCredentialWarning')}</div> : null}
              </div>
            ) : null}
            <label className="flex items-center gap-2 text-xs text-neutral-600 dark:text-neutral-300">
              <input type="checkbox" checked={useCustomPrefix} onChange={(event) => { setUseCustomPrefix(event.target.checked); if (!event.target.checked && editingType) setEditRootPrefix(defaultSpaceTypeRootPrefix(tenantId, editingType)); }} />
              {t('bindingsCustomPrefix')}
            </label>
            {useCustomPrefix ? (
              <label className="grid gap-2 text-xs font-medium text-neutral-600 dark:text-neutral-300">
                {t('bindingsCustomPrefix')}
                <input value={editRootPrefix} onChange={(event) => setEditRootPrefix(event.target.value)} className={`${INPUT_CLASS} font-mono text-xs`} placeholder={editingType ? defaultSpaceTypeRootPrefix(tenantId, editingType) : ''} />
              </label>
            ) : null}
          </DrawerBody>
          <DrawerFooter>
            <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={pending} onClick={() => setEditingType(null)}>{t('cancel')}</button>
            <button type="button" className={PRIMARY_BUTTON_CLASS} disabled={pending || !editProviderId} onClick={() => void handleSave()}>{pending ? t('saving') : t('save')}</button>
          </DrawerFooter>
        </DrawerContent>
      </Drawer>

      <ConfirmDialog
        open={!!clearTarget}
        title={t('bindingsClearTitle')}
        message={t('bindingsClearMessage', { label: clearTarget ? spaceTypeLabel(clearTarget) : '' })}
        confirmLabel={t('bindingsClearConfirm')}
        variant="danger"
        onConfirm={() => { if (clearTarget) void handleClear(clearTarget); }}
        onCancel={() => setClearTarget(null)}
      />
    </main>
  );
}
