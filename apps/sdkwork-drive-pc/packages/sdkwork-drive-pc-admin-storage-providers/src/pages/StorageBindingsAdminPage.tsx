import React, { useEffect, useMemo, useState } from 'react';
import {
  CircleAlert,
  Info,
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
  Modal,
  ModalBody,
  ModalContent,
  ModalDescription,
  ModalFooter,
  ModalHeader,
  ModalTitle,
} from '@sdkwork/ui-pc-react';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import {
  createStorageProviderAdminService,
  type StorageProviderAdminService,
} from '../services/storageProviderAdminService';
import type {
  BusinessStorageBindingView,
  StorageLogicalBucketView,
  StorageProviderBindingView,
  StorageProviderView,
} from '../types/storageProviderAdminTypes';
import { SPACE_TYPES, getSpaceTypeMeta, resolveSpaceTypeDescription, resolveSpaceTypeLabel } from '../utils/spaceTypeConfig';
import { getProviderKindMeta, providerDisplayName } from '../utils/providerKindConfig';
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
import { StorageBucketReadiness } from '../components/StorageBucketReadiness';
import { StorageObjectBrowser } from '../components/StorageObjectBrowser';
import { useTranslation } from '../hooks/useTranslation';
import { useProviderOptions } from '../hooks/useProviderOptions';

interface StorageBindingsAdminPageProps {
  adminStorageSdkClient: DriveAdminStorageSdkClient;
  getSession: () => SessionSnapshot;
  /**
   * Hand-off to the provider-configuration surface.
   *
   * Optional on purpose: a host that mounts this page without the provider pages
   * mounted (the Web Server admin bridges only the resource the menu selected)
   * simply gets the written hint instead of a dead button.
   */
  onManageProviders?: () => void;
}

/** One row of the space-type section: a space type and the target it resolves to. */
interface SpaceTypeBindingRow {
  spaceType: string;
  logicalBucketId: string;
  logicalBucketName: string;
  logicalBucketKey: string;
  logicalObjectPrefix: string;
  providerId: string;
  providerName: string;
  bucket: string;
  writePrefix: string;
  resolvedRootPrefix: string;
  /** Legacy direct-provider binding shown only when no logical binding exists. */
  legacy: boolean;
  bindingId?: string;
  configured: boolean;
}

type ClearTarget =
  | { kind: 'space_type'; spaceType: string }
  | { kind: 'space'; spaceId: string }

interface BrowseTarget {
  key: string;
  label: string;
  providerId: string;
  rootPrefix: string;
}

/** Maximum page size the list operations accept (`PAGINATION_SPEC.md` §3). */
const BINDINGS_PAGE_SIZE = 200;

function defaultSpaceTypeRootPrefix(tenantId: string, spaceType: string): string {
  return `sdkwork-drive/v1/tenants/${tenantId}/space-types/${spaceType}`;
}

function defaultBusinessWritePrefix(spaceType: string): string {
  return `space-types/${spaceType}`;
}

function composePrefix(...parts: Array<string | undefined>): string {
  return parts
    .map((part) => (part ?? '').trim())
    .filter((part) => part !== '')
    .join('/');
}

function defaultSpaceRootPrefix(tenantId: string, spaceId: string): string {
  return `sdkwork-drive/v1/tenants/${tenantId}/spaces/${spaceId}`;
}

export function StorageBindingsAdminPage({
  adminStorageSdkClient,
  getSession,
  onManageProviders,
}: StorageBindingsAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo<StorageProviderAdminService>(
    () => createStorageProviderAdminService({ adminStorageSdkClient, getSession }),
    [adminStorageSdkClient, getSession],
  );

  const tenantId = getSession().context?.tenantId ?? '';

  /**
   * Option set behind the provider switches on this page.
   *
   * Every configuration is needed, not the first cursor page: a switch that
   * omits the providers past row 20 cannot route a binding to them, and a row
   * already bound to such a provider would render as unassigned. The hook reads
   * one maximum-size page and exposes a continuation for the rest.
   */
  const providerOptions = useProviderOptions(service);
  const providers = providerOptions.items;

  /**
   * The three resolution steps, each read as its own set.
   *
   * The unfiltered binding list is one page window ordered space → space type →
   * tenant, so the sections must not share it: a tenant with enough space-scoped
   * bindings would otherwise render every space type as unbound.
   */
  const [spaceTypeBindings, setSpaceTypeBindings] = useState<StorageProviderBindingView[]>([]);
  const [spaceBindings, setSpaceBindings] = useState<StorageProviderBindingView[]>([]);
  const [spaceBindingsNextToken, setSpaceBindingsNextToken] = useState<string | undefined>();
  const [spaceBindingsHasMore, setSpaceBindingsHasMore] = useState(false);
  const [loadingMoreSpaceBindings, setLoadingMoreSpaceBindings] = useState(false);
  const [tenantBinding, setTenantBinding] = useState<StorageProviderBindingView | undefined>();

  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [notice, setNotice] = useState<{ type: 'success' | 'error'; messageKey: string; params?: Record<string, string> } | null>(null);

  // --- tenant storage-provider configuration -------------------------------
  const [editingTenantConfig, setEditingTenantConfig] = useState(false);
  const [tenantProviderId, setTenantProviderId] = useState('');

  // --- logical storage plane ------------------------------------------------
  const [logicalBuckets, setLogicalBuckets] = useState<StorageLogicalBucketView[]>([]);
  const [businessBindings, setBusinessBindings] = useState<BusinessStorageBindingView[]>([]);
  const [logicalBucketDrafts, setLogicalBucketDrafts] = useState<
    Record<string, { providerId: string; objectPrefix: string }>
  >({});

  // --- space-type binding editor -------------------------------------------
  const [editingType, setEditingType] = useState<string | null>(null);
  const [editLogicalBucketId, setEditLogicalBucketId] = useState('');
  const [editWritePrefix, setEditWritePrefix] = useState('');

  // --- space-scope binding editor ------------------------------------------
  const [editingSpaceId, setEditingSpaceId] = useState<string | null>(null);
  const [spaceIdInput, setSpaceIdInput] = useState('');
  const [spaceProviderId, setSpaceProviderId] = useState('');
  const [spaceRootPrefix, setSpaceRootPrefix] = useState('');
  const [useCustomSpacePrefix, setUseCustomSpacePrefix] = useState(false);

  const [clearTarget, setClearTarget] = useState<ClearTarget | null>(null);
  const [bindingFilter, setBindingFilter] = useState<'all' | 'bound' | 'unbound' | 'system' | 'user'>('all');
  /**
   * Binding whose stored objects are being browsed. The row is held rather than
   * its provider id so the dialog keeps its title and root prefix even if the
   * provider list is refreshed underneath it.
   */
  const [browseTarget, setBrowseTarget] = useState<BrowseTarget | null>(null);

  const activeProviders = providers.filter((p) => p.status === 'active');
  const tenantProvider = providers.find((p) => p.id === tenantBinding?.providerId);
  const selectedLogicalBucket = logicalBuckets.find((bucket) => bucket.id === editLogicalBucketId);
  const selectedLogicalProvider = providers.find((p) => p.id === selectedLogicalBucket?.providerId);
  const selectedSpaceProvider = providers.find((p) => p.id === spaceProviderId);
  const tenantConfigProvider = providers.find((p) => p.id === tenantProviderId);
  const browseProvider = browseTarget
    ? providers.find((p) => p.id === browseTarget.providerId)
    : undefined;
  const logicalTenantBinding = businessBindings.find(
    (binding) => binding.scopeType === 'tenant' && binding.lifecycleStatus === 'active',
  );

  const load = (signal?: AbortSignal) => {
    setLoading(true);
    Promise.all([
      // Option set, not a table page — see `useProviderOptions`.
      providerOptions.reload(signal),
      service.listBindingsPage({
        bindingScope: 'space_type',
        pageSize: BINDINGS_PAGE_SIZE,
        signal,
      }),
      service.listBindingsPage({
        bindingScope: 'space',
        pageSize: BINDINGS_PAGE_SIZE,
        signal,
      }),
      service.getDefaultBinding(undefined, { signal }),
      service.listLogicalBuckets({ signal }),
      service.listBusinessBindings({ pageSize: BINDINGS_PAGE_SIZE, signal }),
    ])
      .then(([, spaceTypePage, spacePage, tenantConfig, logicalBucketPage, businessBindingPage]) => {
        setSpaceTypeBindings(spaceTypePage.items.filter((binding) => binding.lifecycleStatus === 'active'));
        setSpaceBindings(spacePage.items.filter((binding) => binding.lifecycleStatus === 'active'));
        setSpaceBindingsNextToken(spacePage.nextPageToken);
        setSpaceBindingsHasMore(spacePage.hasMore);
        setTenantBinding(tenantConfig?.providerId ? tenantConfig : undefined);
        setTenantProviderId(tenantConfig?.providerId ?? '');
        setLogicalBuckets(logicalBucketPage);
        setBusinessBindings(businessBindingPage);
        setLogicalBucketDrafts(
          Object.fromEntries(
            logicalBucketPage.map((bucket) => [
              bucket.id,
              { providerId: bucket.providerId ?? '', objectPrefix: bucket.objectPrefix },
            ]),
          ),
        );
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

  const loadMoreSpaceBindings = async () => {
    if (!spaceBindingsNextToken) return;
    setLoadingMoreSpaceBindings(true);
    try {
      const page = await service.listBindingsPage({
        bindingScope: 'space',
        pageSize: BINDINGS_PAGE_SIZE,
        pageToken: spaceBindingsNextToken,
      });
      setSpaceBindings((current) => [
        ...current,
        ...page.items.filter((binding) => binding.lifecycleStatus === 'active'),
      ]);
      setSpaceBindingsNextToken(page.nextPageToken);
      setSpaceBindingsHasMore(page.hasMore);
    } catch (err) {
      setNotice({
        type: 'error',
        messageKey: 'bindingsNoticeLoadFailed',
        params: { detail: err instanceof Error ? err.message : '' },
      });
    } finally {
      setLoadingMoreSpaceBindings(false);
    }
  };

  const spaceTypeBindingRows: SpaceTypeBindingRow[] = useMemo(() => {
    const logicalById = new Map(logicalBuckets.map((bucket) => [bucket.id, bucket]));
    const businessByType = new Map(
      businessBindings
        .filter((binding) => binding.scopeType === 'space_type' && binding.lifecycleStatus === 'active')
        .map((binding) => [binding.scopeId, binding]),
    );
    return SPACE_TYPES.map((st) => {
      const business = businessByType.get(st.value);
      if (business) {
        const bucket = logicalById.get(business.logicalBucketId);
        const provider = providers.find((candidate) => candidate.id === business.providerId);
        return {
          spaceType: st.value,
          logicalBucketId: business.logicalBucketId,
          logicalBucketName: business.logicalBucketName,
          logicalBucketKey: business.logicalBucketKey,
          logicalObjectPrefix: bucket?.objectPrefix ?? '',
          providerId: business.providerId ?? '',
          providerName: provider ? providerDisplayName(t, provider) : '',
          bucket: provider?.bucket ?? '',
          writePrefix: business.writePrefix,
          resolvedRootPrefix: composePrefix(
            business.storageRootPrefix,
            bucket?.objectPrefix,
            business.writePrefix,
          ),
          legacy: false,
          bindingId: business.id,
          configured: true,
        };
      }

      // Existing installations may still carry a direct space-type provider
      // binding. It remains effective (legacy fallback in the resolver), so the
      // page must show it instead of reporting an already-working type unbound.
      const legacy = spaceTypeBindings.find((binding) => binding.purpose === st.value);
      if (legacy) {
        const provider = providers.find((candidate) => candidate.id === legacy.providerId);
        return {
          spaceType: st.value,
          logicalBucketId: '',
          logicalBucketName: '',
          logicalBucketKey: '',
          logicalObjectPrefix: '',
          providerId: legacy.providerId,
          providerName: provider ? providerDisplayName(t, provider) : legacy.providerId,
          bucket: legacy.storageProvider?.bucket ?? provider?.bucket ?? '',
          writePrefix: '',
          resolvedRootPrefix:
            legacy.storageRootPrefix ?? defaultSpaceTypeRootPrefix(tenantId, st.value),
          legacy: true,
          bindingId: legacy.id,
          configured: true,
        };
      }

      return {
        spaceType: st.value,
        logicalBucketId: '',
        logicalBucketName: '',
        logicalBucketKey: '',
        logicalObjectPrefix: '',
        providerId: '',
        providerName: '',
        bucket: '',
        writePrefix: '',
        resolvedRootPrefix: defaultSpaceTypeRootPrefix(tenantId, st.value),
        legacy: false,
        configured: false,
      };
    });
  }, [businessBindings, logicalBuckets, providers, spaceTypeBindings, t, tenantId]);

  const boundCount = spaceTypeBindingRows.filter((b) => b.configured).length;

  const filteredBindings = spaceTypeBindingRows.filter((binding) => {
    const stMeta = getSpaceTypeMeta(binding.spaceType);
    if (bindingFilter === 'bound') return binding.configured;
    if (bindingFilter === 'unbound') return !binding.configured;
    if (bindingFilter === 'system') return stMeta.isSystem;
    if (bindingFilter === 'user') return !stMeta.isSystem;
    return true;
  });

  const spaceTypeLabel = (spaceType: string) => resolveSpaceTypeLabel(getSpaceTypeMeta(spaceType), t);
  const spaceTypeDescription = (spaceType: string) => resolveSpaceTypeDescription(getSpaceTypeMeta(spaceType), t);
  const providerLabel = (provider: StorageProviderView) =>
    `[${getProviderKindMeta(provider.providerKind).shortLabel}] ${providerDisplayName(t, provider)}`;

  // --- tenant configuration handlers ---------------------------------------
  const openTenantConfig = () => {
    setTenantProviderId(tenantBinding?.providerId ?? (activeProviders[0]?.id ?? ''));
    setEditingTenantConfig(true);
  };

  const handleSaveTenantConfig = async () => {
    if (!tenantProviderId) return;
    setPending(true);
    setNotice(null);
    try {
      await service.setDefaultBinding({ providerId: tenantProviderId });
      setNotice({ type: 'success', messageKey: 'bindingsTenantConfigSaved' });
      setEditingTenantConfig(false);
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

  // --- space-type handlers --------------------------------------------------
  const handleEdit = (spaceType: string) => {
    const existing = spaceTypeBindingRows.find((b) => b.spaceType === spaceType);
    setEditingType(spaceType);
    // Default to the bound logical bucket, then the first available one: an
    // unbound row has no logical bucket and the picker must not open empty.
    setEditLogicalBucketId(
      (existing?.configured ? existing.logicalBucketId : '') || logicalBuckets[0]?.id || '',
    );
    setEditWritePrefix(existing?.writePrefix || defaultBusinessWritePrefix(spaceType));
  };

  const handleSave = async () => {
    if (!editingType || !editLogicalBucketId) return;
    const bucket = logicalBuckets.find((candidate) => candidate.id === editLogicalBucketId);
    if (!bucket?.providerId) {
      setNotice({ type: 'error', messageKey: 'bindingsLogicalBucketProviderMissing' });
      return;
    }
    setPending(true);
    setNotice(null);
    try {
      await service.updateBusinessBinding('space_type', editingType, {
        logicalBucketId: editLogicalBucketId,
        writePrefix: editWritePrefix.trim() || defaultBusinessWritePrefix(editingType),
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

  const saveLogicalBucket = async (logicalBucketId: string) => {
    const draft = logicalBucketDrafts[logicalBucketId];
    if (!draft) return;
    setPending(true);
    setNotice(null);
    try {
      await service.updateLogicalBucket(logicalBucketId, {
        objectPrefix: draft.objectPrefix,
        providerId: draft.providerId || undefined,
      });
      setNotice({ type: 'success', messageKey: 'bindingsLogicalBucketSaved' });
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

  // --- space-scope handlers -------------------------------------------------
  const openSpaceBinding = (spaceId: string) => {
    const existing = spaceBindings.find((binding) => binding.spaceId === spaceId);
    setEditingSpaceId(spaceId);
    setSpaceIdInput(spaceId);
    setSpaceProviderId(existing?.providerId || activeProviders[0]?.id || '');
    const defaultPrefix = defaultSpaceRootPrefix(tenantId, spaceId);
    const existingPrefix = existing?.storageRootPrefix;
    setUseCustomSpacePrefix(Boolean(existingPrefix && existingPrefix !== defaultPrefix));
    setSpaceRootPrefix(existingPrefix ?? defaultPrefix);
  };

  const handleSaveSpaceBinding = async () => {
    const spaceId = spaceIdInput.trim();
    if (!spaceId || !spaceProviderId) return;
    setPending(true);
    setNotice(null);
    try {
      await service.setDefaultBinding({
        spaceId,
        providerId: spaceProviderId,
        storageRootPrefix: useCustomSpacePrefix ? spaceRootPrefix.trim() : undefined,
      });
      setNotice({
        type: 'success',
        messageKey: 'bindingsSpaceScopeSaved',
        params: { spaceId },
      });
      setEditingSpaceId(null);
      load();
    } catch (err) {
      // The backend validates that the space exists and belongs to this tenant,
      // so its detail is the operator's only useful answer for a typo.
      setNotice({
        type: 'error',
        messageKey: 'bindingsNoticeSaveFailed',
        params: { detail: err instanceof Error ? err.message : '' },
      });
    } finally {
      setPending(false);
    }
  };

  // --- clears ---------------------------------------------------------------
  const handleClear = async () => {
    if (!clearTarget) return;
    setPending(true);
    setNotice(null);
    try {
      if (clearTarget.kind === 'space_type') {
        const row = spaceTypeBindingRows.find((binding) => binding.spaceType === clearTarget.spaceType);
        if (row?.legacy) {
          await service.deleteSpaceTypeBinding(clearTarget.spaceType);
        } else {
          await service.deleteBusinessBinding('space_type', clearTarget.spaceType);
        }
        setNotice({
          type: 'success',
          messageKey: 'bindingsNoticeCleared',
          params: { label: spaceTypeLabel(clearTarget.spaceType) },
        });
      } else {
        await service.deleteDefaultBinding(clearTarget.spaceId);
        setNotice({
          type: 'success',
          messageKey: 'bindingsSpaceScopeCleared',
          params: { spaceId: clearTarget.spaceId },
        });
      }
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

  const clearMessage = () => {
    if (!clearTarget) return '';
    if (clearTarget.kind === 'space_type') {
      return t('bindingsClearMessage', { label: spaceTypeLabel(clearTarget.spaceType) });
    }
    return t('bindingsClearSpaceMessage', { spaceId: clearTarget.spaceId });
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
        {/* The resolution order is the page's subject: an operator has to know
            which rule wins before editing one of them. */}
        <div className="mb-4 flex items-start gap-2 rounded-lg border border-neutral-200 bg-white px-4 py-3 text-xs text-neutral-600 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-300">
          <Info aria-hidden="true" className="mt-0.5 shrink-0 text-neutral-400" size={14} />
          <span>{t('bindingsResolutionHint')}</span>
        </div>

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

        {/* The provider switches read one maximum-size option page; when the
            tenant has more configurations than that, the continuation is
            explicit rather than a silently truncated picker. */}
        {providerOptions.hasMore && !loading && (
          <div className="mb-4 flex flex-wrap items-center gap-3 rounded-lg border border-neutral-200 bg-white px-4 py-3 text-xs text-neutral-600 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-300">
            <span>{t('providerOptionsPartial', { count: providers.length })}</span>
            <button
              type="button"
              className={SECONDARY_BUTTON_CLASS}
              disabled={providerOptions.loading}
              onClick={() => void providerOptions.loadMore()}
            >
              {t('providerOptionsLoadMore')}
            </button>
          </div>
        )}

        {activeProviders.length === 0 && !loading && (
          <div className="mb-4 flex items-start gap-3 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800 dark:border-amber-900 dark:bg-amber-950/30 dark:text-amber-200">
            <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
            <span>{t('bindingsNoActiveProviders')}</span>
          </div>
        )}

        {/*
          The logical layer: business routes bind to these buckets, and each
          bucket resolves to a provider configuration. Moving to another vendor
          is one provider change here, not one edit per space type.
        */}
        {!loading && logicalBuckets.length > 0 && (
          <div className={`${CARD_CLASS} mb-4`} data-testid="logical-storage-buckets">
            <div className="flex flex-wrap items-start justify-between gap-3 border-b border-neutral-100 px-5 py-3 dark:border-neutral-800">
              <div>
                <h3 className="text-sm font-semibold">{t('bindingsLogicalBucketsTitle')}</h3>
                <p className="mt-0.5 text-[11px] text-neutral-500">{t('bindingsLogicalBucketsDesc')}</p>
              </div>
              {onManageProviders ? (
                <button type="button" className={SECONDARY_BUTTON_CLASS} onClick={onManageProviders}>
                  {t('bindingsTenantConfigManage')}
                </button>
              ) : null}
            </div>
            <div className="divide-y divide-neutral-100 dark:divide-neutral-800">
              {logicalBuckets.map((bucket) => {
                const draft = logicalBucketDrafts[bucket.id] ?? {
                  providerId: bucket.providerId ?? '',
                  objectPrefix: bucket.objectPrefix,
                };
                return (
                  <div
                    key={bucket.id}
                    className="grid gap-3 px-5 py-3 lg:grid-cols-[1.2fr_1fr_1.2fr_auto] lg:items-end"
                  >
                    <div className="min-w-0">
                      <div className="text-sm font-semibold text-neutral-900 dark:text-neutral-100">
                        {bucket.displayName}
                      </div>
                      <div className="font-mono text-[10px] text-neutral-500">
                        {bucket.bucketKey} · {bucket.objectPrefix || '--'}
                      </div>
                    </div>
                    <label className="grid gap-1 text-[11px] font-medium text-neutral-500">
                      {t('bindingsObjectPrefixLabel')}
                      <input
                        value={draft.objectPrefix}
                        onChange={(event) => setLogicalBucketDrafts((current) => ({
                          ...current,
                          [bucket.id]: { ...draft, objectPrefix: event.target.value },
                        }))}
                        className={`${INPUT_CLASS} font-mono text-xs`}
                      />
                    </label>
                    <label className="grid gap-1 text-[11px] font-medium text-neutral-500">
                      {t('bindingsSelectProvider')}
                      <select
                        value={draft.providerId}
                        onChange={(event) => setLogicalBucketDrafts((current) => ({
                          ...current,
                          [bucket.id]: { ...draft, providerId: event.target.value },
                        }))}
                        className={SELECT_CLASS}
                      >
                        {activeProviders.map((provider) => (
                          <option key={provider.id} value={provider.id}>{providerLabel(provider)}</option>
                        ))}
                      </select>
                    </label>
                    <button
                      type="button"
                      className={PRIMARY_BUTTON_CLASS}
                      disabled={pending || !draft.providerId}
                      onClick={() => void saveLogicalBucket(bucket.id)}
                    >
                      {t('save')}
                    </button>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/*
          The tenant's storage-provider configuration, not a binding editor.

          This is the lowest-priority rule and the one that must always exist: an
          operator reads it to know where everything unbound lands (endpoint,
          bucket, credential, prefix), and can point it at another provider
          configuration. It deliberately offers no "clear" — a tenant with no
          storage configuration cannot write at all, so the honest states are
          "configured" and "not configured yet, go configure one".
        */}
        {!loading && !logicalTenantBinding && (
          <div className={`${CARD_CLASS} mb-4`} data-testid="tenant-storage-configuration">
            <div className="border-b border-neutral-100 px-5 py-3 dark:border-neutral-800">
              <h3 className="text-sm font-semibold">{t('bindingsTenantConfigTitle')}</h3>
              <p className="mt-0.5 text-[11px] text-neutral-500">{t('bindingsTenantConfigDesc')}</p>
            </div>
            <div className="px-5 py-4">
              {tenantBinding?.providerId ? (
                <div className="grid gap-3">
                  <div className="flex flex-wrap items-center gap-3">
                    {(() => {
                      const meta = getProviderKindMeta(tenantProvider?.providerKind ?? '');
                      return (
                        <span className={`inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium ${meta.bgClass} ${meta.textClass}`}>
                          {meta.icon}{' '}
                          {tenantProvider
                            ? providerDisplayName(t, tenantProvider)
                            : tenantBinding.providerId}
                        </span>
                      );
                    })()}
                    <span className={`${BADGE_BASE_CLASS} ${tenantProvider?.credentialConfigured ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300' : 'bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300'}`}>
                      {tenantProvider?.credentialConfigured ? t('credentialSet') : t('credentialMissing')}
                    </span>
                    <span className="flex-1" />
                    <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={activeProviders.length === 0} onClick={openTenantConfig}>
                      {t('bindingsTenantConfigChange')}
                    </button>
                    {onManageProviders ? (
                      <button type="button" className={SECONDARY_BUTTON_CLASS} onClick={onManageProviders}>
                        {t('bindingsTenantConfigManage')}
                      </button>
                    ) : null}
                  </div>
                  <dl className="grid grid-cols-1 gap-x-6 gap-y-1 text-xs sm:grid-cols-2">
                    <div className="flex gap-2">
                      <dt className="text-neutral-500">{t('bindingsBucketLabel')}</dt>
                      <dd className="font-mono text-neutral-800 dark:text-neutral-100">{tenantProvider?.bucket ?? tenantBinding.storageProvider?.bucket ?? '--'}</dd>
                    </div>
                    <div className="flex gap-2">
                      <dt className="text-neutral-500">{t('endpoint')}</dt>
                      <dd className="min-w-0 truncate font-mono text-neutral-800 dark:text-neutral-100">{tenantProvider?.endpointUrl ?? '--'}</dd>
                    </div>
                    <div className="flex gap-2">
                      <dt className="text-neutral-500">{t('bindingsPrefixLabel')}</dt>
                      <dd className="min-w-0 break-all font-mono text-neutral-800 dark:text-neutral-100">{tenantBinding.storageRootPrefix ?? `sdkwork-drive/v1/tenants/${tenantId}`}</dd>
                    </div>
                    {tenantProvider?.region ? (
                      <div className="flex gap-2">
                        <dt className="text-neutral-500">{t('region')}</dt>
                        <dd className="font-mono text-neutral-800 dark:text-neutral-100">{tenantProvider.region}</dd>
                      </div>
                    ) : null}
                  </dl>
                  {!onManageProviders ? (
                    <p className="text-[11px] text-neutral-500">{t('bindingsTenantConfigManageHint')}</p>
                  ) : null}
                </div>
              ) : (
                <div className="flex flex-wrap items-center gap-3">
                  <CircleAlert aria-hidden="true" className="shrink-0 text-amber-600 dark:text-amber-400" size={16} />
                  <div className="min-w-0">
                    <div className="text-sm font-medium text-amber-800 dark:text-amber-200">{t('bindingsTenantConfigUnset')}</div>
                    <p className="mt-0.5 text-xs text-neutral-500">{t('bindingsTenantConfigUnsetDesc')}</p>
                  </div>
                  <span className="flex-1" />
                  <button
                    type="button"
                    className={PRIMARY_BUTTON_CLASS}
                    disabled={activeProviders.length === 0}
                    onClick={openTenantConfig}
                  >
                    {t('bindingsTenantConfigSet')}
                  </button>
                  {onManageProviders ? (
                    <button type="button" className={SECONDARY_BUTTON_CLASS} onClick={onManageProviders}>
                      {t('bindingsTenantConfigManage')}
                    </button>
                  ) : null}
                </div>
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
              <table className="w-full min-w-[1180px] text-left text-sm">
                <thead className="border-b border-neutral-200 bg-neutral-50 text-xs font-medium text-neutral-500 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-400">
                  <tr>
                    <th className="px-5 py-3 font-semibold">{t('bindingsColSpaceType')}</th>
                    <th className="px-5 py-3 font-semibold">{t('bindingsColDescription')}</th>
                    <th className="px-5 py-3 font-semibold">{t('bindingsColLogicalBucket')}</th>
                    <th className="px-5 py-3 font-semibold">{t('bindingsColBucket')}</th>
                    <th className="px-5 py-3 font-semibold">{t('bindingsColPrefix')}</th>
                    <th className="px-5 py-3 font-semibold">{t('bindingsColStatus')}</th>
                    <th className="px-5 py-3 text-right font-semibold">{t('colActions')}</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-neutral-100 dark:divide-neutral-800">
                  {filteredBindings.map((binding) => {
                    const stMeta = getSpaceTypeMeta(binding.spaceType);
                    const SpaceTypeIcon = stMeta.icon;
                    const provider = providers.find((p) => p.id === binding.providerId);
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
                          {binding.configured ? (
                            <div className="grid gap-0.5">
                              <span className="text-xs font-medium text-neutral-900 dark:text-neutral-100">
                                {binding.logicalBucketName || binding.providerName || binding.providerId}
                              </span>
                              <span className="text-[10px] text-neutral-500">
                                {binding.legacy
                                  ? t('bindingsLegacyBinding')
                                  : binding.providerName || binding.providerId}
                              </span>
                            </div>
                          ) : (
                            <span className="text-xs text-neutral-400">{t('bindingsNotAssigned')}</span>
                          )}
                        </td>

                        <td className="px-5 py-3">
                          {binding.configured ? (
                            <span className="font-mono text-xs text-neutral-700 dark:text-neutral-300">{binding.bucket || '--'}</span>
                          ) : (
                            <span className="text-xs text-neutral-400">--</span>
                          )}
                        </td>

                        <td className="px-5 py-3">
                          {binding.configured ? (
                            <span className="font-mono text-[10px] leading-relaxed break-all text-neutral-500">
                              {binding.resolvedRootPrefix}
                            </span>
                          ) : (
                            <span className="text-xs text-neutral-400">--</span>
                          )}
                        </td>

                        <td className="px-5 py-3">
                          {binding.configured ? (
                            <div className="flex flex-wrap items-center gap-1.5">
                              <span className={`${BADGE_BASE_CLASS} bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300`}>
                                <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                                {t('bindingsStatusBound')}
                              </span>
                              {provider && !provider.credentialConfigured && (
                                <span className={`${BADGE_BASE_CLASS} bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300`} title={t('bindingsCredentialWarning')}>
                                  <span className="h-1.5 w-1.5 rounded-full bg-amber-500" />
                                  {t('bindingsStatusCredentialMissing')}
                                </span>
                              )}
                            </div>
                          ) : (
                            <span className={`${BADGE_BASE_CLASS} bg-neutral-100 text-neutral-500 dark:bg-neutral-800 dark:text-neutral-400`}>
                              <span className="h-1.5 w-1.5 rounded-full bg-neutral-400" />
                              {t('bindingsStatusUnbound')}
                            </span>
                          )}
                        </td>

                        <td className="px-5 py-3 text-right">
                          <div className="flex items-center justify-end gap-1">
                              {/* Read-only, so it leads: an operator checks what a
                                  space type already stores before rebinding it. */}
                              {binding.configured && provider && (
                                <button
                                  type="button"
                                  className="rounded-md px-2.5 py-1 text-xs font-medium text-neutral-600 hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800"
                                  onClick={() => setBrowseTarget({
                                    key: `space-type-${binding.spaceType}`,
                                    label: stLabel,
                                    providerId: binding.providerId,
                                    rootPrefix: binding.resolvedRootPrefix,
                                  })}
                                >
                                  {t('bindingsBrowseFiles')}
                                </button>
                              )}
                              <button
                                type="button"
                                className="rounded-md px-2.5 py-1 text-xs font-medium text-blue-600 hover:bg-blue-50 dark:text-blue-400 dark:hover:bg-blue-950/30"
                                onClick={() => handleEdit(binding.spaceType)}
                                disabled={logicalBuckets.length === 0}
                              >
                                {binding.configured ? t('bindingsChange') : t('bindingsAssign')}
                              </button>
                              {binding.configured && (
                                <button
                                  type="button"
                                  className="rounded-md px-2.5 py-1 text-xs font-medium text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
                                  onClick={() => setClearTarget({ kind: 'space_type', spaceType: binding.spaceType })}
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

            {/*
              Space-scoped bindings: the first rule the writer consults. They are
              normally written by the space-provisioning flow rather than by hand,
              so this section is an inventory an operator reads (and can correct)
              instead of a form they must fill for every space.
            */}
            <div className={`${CARD_CLASS} mt-4`} data-testid="space-scope-bindings">
              <div className="flex flex-wrap items-center gap-3 border-b border-neutral-100 px-5 py-3 dark:border-neutral-800">
                <div className="min-w-0">
                  <h3 className="text-sm font-semibold">{t('bindingsSpaceScopeTitle')}</h3>
                  <p className="mt-0.5 text-[11px] text-neutral-500">{t('bindingsSpaceScopeDesc')}</p>
                </div>
                <span className="flex-1" />
                <button
                  type="button"
                  className={SECONDARY_BUTTON_CLASS}
                  disabled={activeProviders.length === 0}
                  onClick={() => openSpaceBinding('')}
                >
                  {t('bindingsSpaceScopeAdd')}
                </button>
              </div>
              <div className="overflow-x-auto">
                <table className="w-full min-w-[900px] text-left text-sm">
                  <thead className="border-b border-neutral-200 bg-neutral-50 text-xs font-medium text-neutral-500 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-400">
                    <tr>
                      <th className="px-5 py-3 font-semibold">{t('bindingsSpaceIdLabel')}</th>
                      <th className="px-5 py-3 font-semibold">{t('bindingsColProvider')}</th>
                      <th className="px-5 py-3 font-semibold">{t('bindingsColBucket')}</th>
                      <th className="px-5 py-3 font-semibold">{t('bindingsColPrefix')}</th>
                      <th className="px-5 py-3 text-right font-semibold">{t('colActions')}</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-neutral-100 dark:divide-neutral-800">
                    {spaceBindings.map((binding) => {
                      const spaceId = binding.spaceId ?? '';
                      const provider = providers.find((p) => p.id === binding.providerId);
                      const providerMeta = provider ? getProviderKindMeta(provider.providerKind) : null;
                      return (
                        <tr key={binding.id} className="hover:bg-neutral-50 dark:hover:bg-neutral-800/50">
                          <td className="px-5 py-3 font-mono text-xs text-neutral-800 dark:text-neutral-100">{spaceId}</td>
                          <td className="px-5 py-3">
                            {providerMeta && provider ? (
                              <div className="flex items-center gap-1.5">
                                <span className={`inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] font-bold ${providerMeta.bgClass} ${providerMeta.textClass}`}>
                                  {providerMeta.icon}
                                </span>
                                <span className="text-xs font-medium">{providerDisplayName(t, provider)}</span>
                              </div>
                            ) : (
                              <span className="font-mono text-xs text-neutral-600 dark:text-neutral-300">{binding.providerId}</span>
                            )}
                          </td>
                          <td className="px-5 py-3 font-mono text-xs text-neutral-700 dark:text-neutral-300">
                            {binding.storageProvider?.bucket ?? provider?.bucket ?? '--'}
                          </td>
                          <td className="px-5 py-3 font-mono text-[10px] break-all text-neutral-500">
                            {binding.storageRootPrefix ?? defaultSpaceRootPrefix(tenantId, spaceId)}
                          </td>
                          <td className="px-5 py-3 text-right">
                            <div className="flex items-center justify-end gap-1">
                              {provider && (
                                <button
                                  type="button"
                                  className="rounded-md px-2.5 py-1 text-xs font-medium text-neutral-600 hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800"
                                  onClick={() => setBrowseTarget({
                                    key: `space-${spaceId}`,
                                    label: spaceId,
                                    providerId: binding.providerId,
                                    rootPrefix: binding.storageRootPrefix ?? defaultSpaceRootPrefix(tenantId, spaceId),
                                  })}
                                >
                                  {t('bindingsBrowseFiles')}
                                </button>
                              )}
                              <button
                                type="button"
                                className="rounded-md px-2.5 py-1 text-xs font-medium text-blue-600 hover:bg-blue-50 dark:text-blue-400 dark:hover:bg-blue-950/30"
                                disabled={activeProviders.length === 0}
                                onClick={() => openSpaceBinding(spaceId)}
                              >
                                {t('bindingsChange')}
                              </button>
                              <button
                                type="button"
                                className="rounded-md px-2.5 py-1 text-xs font-medium text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
                                disabled={pending}
                                onClick={() => setClearTarget({ kind: 'space', spaceId })}
                              >
                                {t('clear')}
                              </button>
                            </div>
                          </td>
                        </tr>
                      );
                    })}
                    {spaceBindings.length === 0 && (
                      <tr>
                        <td colSpan={5} className="px-5 py-8 text-center text-xs text-neutral-500">
                          {t('bindingsSpaceScopeEmpty')}
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </div>
              {spaceBindingsHasMore && (
                <div className="flex items-center justify-between gap-3 border-t border-neutral-100 px-5 py-3 dark:border-neutral-800">
                  <span className="text-xs text-neutral-500">{t('bindingsSpaceScopePartial', { count: spaceBindings.length })}</span>
                  <button
                    type="button"
                    className={SECONDARY_BUTTON_CLASS}
                    disabled={loadingMoreSpaceBindings}
                    onClick={() => void loadMoreSpaceBindings()}
                  >
                    {loadingMoreSpaceBindings ? t('loading') : t('providerOptionsLoadMore')}
                  </button>
                </div>
              )}
            </div>
          </>
        )}
      </div>

      {/* Tenant configuration: which provider configuration the tenant writes to. */}
      <Drawer open={editingTenantConfig} onOpenChange={(open) => { if (!pending) setEditingTenantConfig(open); }}>
        <DrawerContent size="md">
          <DrawerHeader>
            <DrawerTitle>{t('bindingsTenantConfigTitle')}</DrawerTitle>
            <DrawerDescription>{t('bindingsTenantConfigDesc')}</DrawerDescription>
          </DrawerHeader>
          <DrawerBody className="grid content-start gap-5">
            <label className="grid gap-2 text-xs font-medium text-neutral-600 dark:text-neutral-300">
              {t('bindingsSelectProvider')}
              <select value={tenantProviderId} onChange={(event) => setTenantProviderId(event.target.value)} className={SELECT_CLASS}>
                {activeProviders.map((provider) => (
                  <option key={provider.id} value={provider.id}>{providerLabel(provider)}</option>
                ))}
              </select>
            </label>
            {tenantConfigProvider ? (
              <>
                <div className="grid gap-1 rounded-md border border-neutral-200 bg-neutral-50 p-3 text-xs text-neutral-500 dark:border-neutral-700 dark:bg-neutral-800">
                  <div className="flex gap-2">
                    <span>{t('bindingsBucketLabel')}</span>
                    <span className="font-mono text-neutral-800 dark:text-neutral-100">{tenantConfigProvider.bucket || '--'}</span>
                  </div>
                  <div className="flex gap-2">
                    <span>{t('endpoint')}</span>
                    <span className="min-w-0 break-all font-mono text-neutral-800 dark:text-neutral-100">{tenantConfigProvider.endpointUrl}</span>
                  </div>
                  <div className="flex gap-2">
                    <span>{t('bindingsPrefixLabel')}</span>
                    <span className="min-w-0 break-all font-mono text-neutral-800 dark:text-neutral-100">{`sdkwork-drive/v1/tenants/${tenantId}`}</span>
                  </div>
                </div>
                <StorageBucketReadiness provider={tenantConfigProvider} service={service} />
              </>
            ) : null}
          </DrawerBody>
          <DrawerFooter>
            <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={pending} onClick={() => setEditingTenantConfig(false)}>{t('cancel')}</button>
            <button type="button" className={PRIMARY_BUTTON_CLASS} disabled={pending || !tenantProviderId} onClick={() => void handleSaveTenantConfig()}>{pending ? t('saving') : t('save')}</button>
          </DrawerFooter>
        </DrawerContent>
      </Drawer>

      {/* Space-type binding: which logical bucket (and therefore provider) the
          type's objects route through, and its write prefix inside that bucket. */}
      <Drawer open={editingType !== null} onOpenChange={(open) => { if (!open && !pending) setEditingType(null); }}>
        <DrawerContent size="md">
          <DrawerHeader>
            <DrawerTitle>{editingType ? spaceTypeLabel(editingType) : t('bindingsAssign')}</DrawerTitle>
            <DrawerDescription>{t('bindingsLogicalBucketAssignDesc')}</DrawerDescription>
          </DrawerHeader>
          <DrawerBody className="grid content-start gap-5">
            <label className="grid gap-2 text-xs font-medium text-neutral-600 dark:text-neutral-300">
              {t('bindingsSelectLogicalBucket')}
              <select value={editLogicalBucketId} onChange={(event) => setEditLogicalBucketId(event.target.value)} className={SELECT_CLASS}>
                {logicalBuckets.map((bucket) => (
                  <option key={bucket.id} value={bucket.id}>
                    {bucket.displayName} · {bucket.bucketKey}
                  </option>
                ))}
              </select>
            </label>
            {selectedLogicalBucket ? (
              <div className="grid gap-1 rounded-md border border-neutral-200 bg-neutral-50 p-3 text-xs text-neutral-500 dark:border-neutral-700 dark:bg-neutral-800">
                <div className="flex gap-2">
                  <span>{t('bindingsColProvider')}</span>
                  <span className="text-neutral-800 dark:text-neutral-100">
                    {selectedLogicalProvider
                      ? providerLabel(selectedLogicalProvider)
                      : t('bindingsProviderUnavailable')}
                  </span>
                </div>
                <div className="flex gap-2">
                  <span>{t('bindingsBucketLabel')}</span>
                  <span className="font-mono text-neutral-800 dark:text-neutral-100">{selectedLogicalProvider?.bucket || '--'}</span>
                </div>
                <div className="flex gap-2">
                  <span>{t('bindingsObjectPrefixLabel')}</span>
                  <span className="font-mono text-neutral-800 dark:text-neutral-100">{selectedLogicalBucket.objectPrefix || '--'}</span>
                </div>
              </div>
            ) : null}
            {selectedLogicalProvider ? (
              <StorageBucketReadiness provider={selectedLogicalProvider} service={service} />
            ) : null}
            <label className="grid gap-2 text-xs font-medium text-neutral-600 dark:text-neutral-300">
              {t('bindingsWritePrefixLabel')}
              <input
                value={editWritePrefix}
                onChange={(event) => setEditWritePrefix(event.target.value)}
                className={`${INPUT_CLASS} font-mono text-xs`}
                placeholder={editingType ? defaultBusinessWritePrefix(editingType) : ''}
              />
            </label>
            <p className="text-[11px] text-neutral-500">{t('bindingsWritePrefixHint')}</p>
          </DrawerBody>
          <DrawerFooter>
            <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={pending} onClick={() => setEditingType(null)}>{t('cancel')}</button>
            <button type="button" className={PRIMARY_BUTTON_CLASS} disabled={pending || !editLogicalBucketId || !selectedLogicalBucket?.providerId} onClick={() => void handleSave()}>{pending ? t('saving') : t('save')}</button>
          </DrawerFooter>
        </DrawerContent>
      </Drawer>

      {/* Space-scope binding: one space pinning its own bucket. */}
      <Drawer open={editingSpaceId !== null} onOpenChange={(open) => { if (!open && !pending) setEditingSpaceId(null); }}>
        <DrawerContent size="md">
          <DrawerHeader>
            <DrawerTitle>{t('bindingsSpaceScopeTitle')}</DrawerTitle>
            <DrawerDescription>{t('bindingsSpaceScopeDesc')}</DrawerDescription>
          </DrawerHeader>
          <DrawerBody className="grid content-start gap-5">
            <label className="grid gap-2 text-xs font-medium text-neutral-600 dark:text-neutral-300">
              {t('bindingsSpaceIdLabel')}
              <input
                value={spaceIdInput}
                onChange={(event) => {
                  setSpaceIdInput(event.target.value);
                  if (!useCustomSpacePrefix) {
                    setSpaceRootPrefix(defaultSpaceRootPrefix(tenantId, event.target.value.trim()));
                  }
                }}
                disabled={Boolean(editingSpaceId)}
                placeholder={t('bindingsSpaceIdPlaceholder')}
                className={`${INPUT_CLASS} font-mono text-xs disabled:opacity-60`}
              />
            </label>
            <label className="grid gap-2 text-xs font-medium text-neutral-600 dark:text-neutral-300">
              {t('bindingsSelectProvider')}
              <select value={spaceProviderId} onChange={(event) => setSpaceProviderId(event.target.value)} className={SELECT_CLASS}>
                {activeProviders.map((provider) => (
                  <option key={provider.id} value={provider.id}>{providerLabel(provider)}</option>
                ))}
              </select>
            </label>
            {selectedSpaceProvider ? (
              <StorageBucketReadiness provider={selectedSpaceProvider} service={service} />
            ) : null}
            <label className="flex items-center gap-2 text-xs text-neutral-600 dark:text-neutral-300">
              <input
                type="checkbox"
                checked={useCustomSpacePrefix}
                onChange={(event) => {
                  setUseCustomSpacePrefix(event.target.checked);
                  if (!event.target.checked) {
                    setSpaceRootPrefix(defaultSpaceRootPrefix(tenantId, spaceIdInput.trim()));
                  }
                }}
              />
              {t('bindingsCustomPrefix')}
            </label>
            {useCustomSpacePrefix ? (
              <input value={spaceRootPrefix} onChange={(event) => setSpaceRootPrefix(event.target.value)} className={`${INPUT_CLASS} font-mono text-xs`} />
            ) : (
              <p className="break-all font-mono text-[11px] text-neutral-500">
                {spaceIdInput.trim() ? defaultSpaceRootPrefix(tenantId, spaceIdInput.trim()) : ''}
              </p>
            )}
          </DrawerBody>
          <DrawerFooter>
            <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={pending} onClick={() => setEditingSpaceId(null)}>{t('cancel')}</button>
            <button type="button" className={PRIMARY_BUTTON_CLASS} disabled={pending || !spaceIdInput.trim() || !spaceProviderId} onClick={() => void handleSaveSpaceBinding()}>{pending ? t('saving') : t('save')}</button>
          </DrawerFooter>
        </DrawerContent>
      </Drawer>

      {/*
        The browser here is `StorageObjectBrowser` — the provider package's own
        object browser. It is NOT the one the bucket page mounts: that page uses
        `BucketObjectManagerDialog` from `sdkwork-drive-pc-admin-storage-buckets`,
        which adds the category rail, the bucket-scoped requests and the shared
        preview/editor surface. Two object browsers therefore exist today; this
        one is the older, simpler plane and does not preview files.

        It is keyed by binding: opening another row must mount a fresh prefix and
        page token instead of inheriting the previous one's.
      */}
      <Modal
        open={browseTarget !== null && browseProvider !== undefined}
        onOpenChange={(open) => {
          if (!open) {
            setBrowseTarget(null);
          }
        }}
      >
        <ModalContent size="xl">
          <ModalHeader>
            <ModalTitle>
              {t('bindingsBrowserTitle', { label: browseTarget ? browseTarget.label : '' })}
            </ModalTitle>
            <ModalDescription>{t('bindingsBrowserDescription')}</ModalDescription>
          </ModalHeader>
          <ModalBody className="grid content-start gap-4">
            {browseTarget && browseProvider ? (
              <StorageObjectBrowser
                key={`browse-${browseTarget.key}-${browseProvider.id}`}
                initialPrefix={browseTarget.rootPrefix}
                provider={browseProvider}
                service={service}
              />
            ) : null}
          </ModalBody>
          <ModalFooter>
            <button type="button" className={SECONDARY_BUTTON_CLASS} onClick={() => setBrowseTarget(null)}>
              {t('bindingsBrowserClose')}
            </button>
          </ModalFooter>
        </ModalContent>
      </Modal>

      <ConfirmDialog
        open={!!clearTarget}
        title={t('bindingsClearTitle')}
        message={clearMessage()}
        confirmLabel={t('bindingsClearConfirm')}
        variant="danger"
        onConfirm={() => { if (clearTarget) void handleClear(); }}
        onCancel={() => setClearTarget(null)}
      />
    </main>
  );
}
