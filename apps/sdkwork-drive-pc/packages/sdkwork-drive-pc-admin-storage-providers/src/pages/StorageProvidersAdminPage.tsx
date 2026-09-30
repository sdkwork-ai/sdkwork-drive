import React, { useEffect, useMemo, useState } from 'react';
import {
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  CircleAlert,
  HardDrive,
  KeyRound,
  LoaderCircle,
  Plus,
  RefreshCw,
  X,
} from 'lucide-react';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import { StorageProviderTable } from '../components/StorageProviderTable';
import { StorageProviderEditor } from '../components/StorageProviderEditor';
import { StorageProviderDetailDrawer } from '../components/StorageProviderDetailDrawer';
import {
  createStorageProviderAdminService,
  type StorageProviderAdminService,
} from '../services/storageProviderAdminService';
import type {
  CreateStorageProviderInput,
  StorageProviderVendorCapabilityDefaults,
  StorageProviderVendorCredentialFields,
  StorageProviderView,
  UpdateStorageProviderInput,
} from '../types/storageProviderAdminTypes';
import { PRIMARY_BUTTON_CLASS, BADGE_BASE_CLASS, ICON_BUTTON_CLASS, SECONDARY_BUTTON_CLASS, SELECT_CLASS } from '../utils/uiPrimitives';
import { getAllProviderKindMeta, providerKindLabel } from '../utils/providerKindConfig';
import { summarizeProviderAccountDefaults } from '../utils/providerAccountDefaultsSummary';
import {
  isPendingPlaceholder,
  mergePlaceholderProviders,
  pruneResolvedPlaceholders,
} from '../utils/placeholderCredentialTracking';
import { useTranslation } from '../hooks/useTranslation';
import { useProviderOptions } from '../hooks/useProviderOptions';

interface StorageProvidersAdminPageProps {
  adminStorageSdkClient: DriveAdminStorageSdkClient;
  getSession: () => SessionSnapshot;
}

type PageNotice = { type: 'success' | 'error'; messageKey: string; params?: Record<string, string> } | undefined;

export function StorageProvidersAdminPage({
  adminStorageSdkClient,
  getSession,
}: StorageProvidersAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo<StorageProviderAdminService>(
    () => createStorageProviderAdminService({ adminStorageSdkClient, getSession }),
    [adminStorageSdkClient, getSession],
  );
  const [providers, setProviders] = useState<StorageProviderView[]>([]);
  /**
   * Paged option set behind this page's provider switches.
   *
   * The table stays paginated; the switches (the drawer's default-binding
   * picker, the editor's id-uniqueness guard, the post-initialization health
   * sweep, the header count) read the option set instead, which is one
   * maximum-size page plus an explicit continuation. Reading them from the
   * table's page is what made a switch omit providers that sat past page 1.
   */
  const providerOptions = useProviderOptions(service);
  // Per-kind vendor vocabulary the last bootstrap run returned. Empty until the
  // operator runs "initialize accounts"; the editor then prefers these over its
  // own static catalog, so the labels match the account that was actually
  // minted rather than a table that may have drifted.
  const [vendorCredentialFields, setVendorCredentialFields] = useState<
    Record<string, StorageProviderVendorCredentialFields | undefined>
  >({});
  // Same idea for the two advanced controls: the per-vendor encryption-mode and
  // storage-class lists the server's contract layer owns. Kept separate from the
  // credential vocabulary because a vendor may legitimately offer no tier choice
  // (an empty list) while still having labelled fields.
  const [vendorCapabilities, setVendorCapabilities] = useState<
    Record<string, StorageProviderVendorCapabilityDefaults | undefined>
  >({});
  const [kindFilter, setKindFilter] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize] = useState(20);
  const [pageCursors, setPageCursors] = useState<Record<number, string | undefined>>({ 1: undefined });
  const [nextPageToken, setNextPageToken] = useState<string | undefined>();
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [notice, setNotice] = useState<PageNotice>();
  /**
   * Result of the post-initialization connectivity sweep, surfaced next to the
   * provider list as an "N 可用 / M 待配置" badge.
   *
   * `undefined` before the first run; the badge then stays hidden rather than
   * claiming zero, which would read as a failure that never happened.
   */
  const [selfCheck, setSelfCheck] = useState<
    { reachable: number; total: number; pending: number } | undefined
  >();
  /**
   * Providers bootstrapped with a placeholder credential that the operator has
   * not replaced yet. Drives the "待配置" hint in the list.
   */
  const [placeholderProviderIds, setPlaceholderProviderIds] = useState<ReadonlySet<string>>(
    () => new Set<string>(),
  );
  /** Providers a connectivity check reached; a placeholder never is one. */
  const [reachableProviderIds, setReachableProviderIds] = useState<ReadonlySet<string>>(
    () => new Set<string>(),
  );
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingProvider, setEditingProvider] = useState<StorageProviderView | undefined>();
  const [detailDrawerOpen, setDetailDrawerOpen] = useState(false);
  const [detailProvider, setDetailProvider] = useState<StorageProviderView | undefined>();
  const currentPageToken = pageCursors[page];
  // `'all'` is the selector's own "no filter" value. Anything else travels to
  // the server as a wire filter, including the literal `custom`, which the
  // backend expands to the whole `custom:<vendor>` family.
  const kindQuery = kindFilter === 'all' ? undefined : kindFilter;

  const refreshProviders = async (signal?: AbortSignal) => {
    const result = await service.listProvidersPage({
      providerKind: kindQuery,
      signal,
      pageSize,
      pageToken: currentPageToken,
    });
    setProviders(result.items);
    setNextPageToken(result.nextPageToken);
    setHasMore(Boolean(result.nextPageToken));
    setPageCursors((current) => {
      const next = { ...current };
      Object.keys(next)
        .map(Number)
        .filter((cursorPage) => cursorPage > page + 1)
        .forEach((cursorPage) => {
          delete next[cursorPage];
        });
      if (result.nextPageToken) {
        next[page + 1] = result.nextPageToken;
      } else {
        delete next[page + 1];
      }
      return next;
    });
    return result.items;
  };

  /**
   * Re-read the page-independent option set.
   *
   * Failures are swallowed on purpose: the table's own load notice already
   * reports an unreachable backend, and a switch that keeps its previous
   * options (or stays empty) is a better answer than a second identical error.
   */
  const refreshProviderOptions = async (signal?: AbortSignal) => {
    try {
      return await providerOptions.reload(signal);
    } catch {
      return providerOptions.items;
    }
  };

  /**
   * Refresh everything one operator action invalidates: the table page *and*
   * the option set, because a create/update/delete changes both.
   *
   * The paging effect below deliberately does not use this — turning a page must
   * not re-read the option set — which is why the two loads are separate.
   */
  const reload = (signal?: AbortSignal) => {
    setLoading(true);
    setNotice(undefined);
    return Promise.all([refreshProviders(signal), refreshProviderOptions(signal)])
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', messageKey: 'noticeLoadFailed' });
      })
      .finally(() => setLoading(false));
  };

  /**
   * The table is one server page, filtered by the server.
   *
   * `kindQuery` is part of the request, so the window is selected from the
   * filtered set: switching the provider-kind selector can never answer "no
   * rows" for a kind whose rows sit on a later page of the unfiltered list. The
   * option set is loaded by `useProviderOptions`, not here: no page turn can
   * change it.
   */
  useEffect(() => {
    const c = new AbortController();
    setLoading(true);
    setNotice(undefined);
    refreshProviders(c.signal)
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', messageKey: 'noticeLoadFailed' });
      })
      .finally(() => setLoading(false));
    return () => c.abort();
  }, [currentPageToken, kindQuery, service, page, pageSize]);

  const syncProviderViews = (items: StorageProviderView[], saved?: StorageProviderView) => {
    setProviders(items);
    if (!saved) return;
    if (editingProvider?.id === saved.id) {
      setEditingProvider(saved);
    }
    if (detailProvider?.id === saved.id) {
      setDetailProvider(saved);
    }
  };

  const runTableMutation = (op: () => Promise<unknown>, noticeKey: string) => {
    setPending(true);
    setNotice(undefined);
    op()
      .then(async () => {
        const items = await refreshProviders();
        setProviders(items);
        // A mutation changes the complete set as well (create/delete) or the
        // rows a picker shows (rename, activate, credential), so the option set
        // is re-read in the same pass.
        await refreshProviderOptions();
        setNotice({ type: 'success', messageKey: noticeKey });
      })
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', messageKey: 'noticeOperationFailed' });
      })
      .finally(() => setPending(false));
  };

  const createProvider = async (input: CreateStorageProviderInput) => {
    const created = await service.createProvider(input);
    const items = await refreshProviders();
    syncProviderViews(items, created);
    await refreshProviderOptions();
    return created;
  };

  const updateProvider = async (id: string, input: UpdateStorageProviderInput) => {
    const updated = await service.updateProvider(id, input);
    const items = await refreshProviders();
    syncProviderViews(items, updated);
    await refreshProviderOptions();
    return updated;
  };

  const rotateCredential = async (id: string, ref: string) => {
    const updated = await service.rotateCredential(id, ref);
    const items = await refreshProviders();
    syncProviderViews(items, updated);
    await refreshProviderOptions();
    return updated;
  };

  const deleteProvider = (id: string) => runTableMutation(() => service.deleteProvider(id), 'noticeDeleted');
  const activateProvider = (id: string) => runTableMutation(() => service.activateProvider(id), 'noticeEnabled');
  const deactivateProvider = (id: string) => runTableMutation(() => service.deactivateProvider(id), 'noticeDisabled');
  const testProvider = (id: string) => runTableMutation(() => service.testProvider(id), 'noticeTested');
  const testProviders = async (providerIds: string[]) => {
    setPending(true);
    setNotice(undefined);
    let passed = 0;
    const reachable = new Set<string>();
    try {
      for (const id of providerIds) {
        try {
          const ok = await service.testProvider(id);
          if (ok) {
            passed += 1;
            reachable.add(id);
          }
        } catch {
          // continue batch
        }
      }
      const items = await refreshProviders();
      setProviders(items);
      // A reachable provider cannot still be carrying a placeholder, so this
      // sweep is also a legitimate way for the hint to clear.
      setReachableProviderIds((current) => {
        const next = new Set(current);
        for (const id of reachable) {
          next.add(id);
        }
        return next;
      });
      setPlaceholderProviderIds((current) =>
        pruneResolvedPlaceholders(current, items.map((item) => item.id), [...reachable]),
      );
      setNotice({
        type: 'success',
        messageKey: 'testAllSummary',
        params: { total: String(providerIds.length), passed: String(passed) },
      });
      return { passed, total: providerIds.length };
    } catch (err) {
      if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', messageKey: 'noticeOperationFailed' });
      return { passed, total: providerIds.length };
    } finally {
      setPending(false);
    }
  };
  const setDefaultBinding = (id: string, spaceId?: string) =>
    runTableMutation(() => service.setDefaultBinding({ providerId: id, spaceId }), 'noticeBindingUpdated');
  const deleteDefaultBinding = (spaceId?: string) =>
    runTableMutation(() => service.deleteDefaultBinding(spaceId), 'noticeBindingCleared');

  /**
   * 初始化后的全量连通性自检。
   *
   * 与 `initializeProviderAccounts` 分开写，是为了不改写初始化那条通知——初始化
   * 的计数（新建了几个、写了几条占位密钥）才是运维判断"幂等、没覆盖我填的密钥"
   * 的依据，不能被自检结果覆盖。自检结果单列一个 `selfCheck*` 通知槽。
   *
   * 只测 active 的 provider：一个被停用的 provider 测不通是预期内的，把它算进
   * "待配置"会把运维引向一个不需要动的地方。
   *
   * 自检的目标来自**完整**的 provider 集合（`providerOptions`），不是表格当前那一页。
   * 初始化刚建出 25 个 provider 时，第 1 页只有 20 个——用分页结果当全集，恰好会把
   * `tencent_cos`（默认绑定指向的那个）漏掉，于是"N 可用 / M 待配置"既少算一个，
   * 也让运维以为它没被初始化。
   */
  const runPostInitSelfCheck = async (
    allProviders: StorageProviderView[],
    providerIds: string[],
  ) => {
    const targets = allProviders
      .filter((p) => p.status === 'active' && providerIds.includes(p.id))
      .map((p) => p.id);
    if (targets.length === 0) {
      return;
    }
    let reachable = 0;
    const reachableIds = new Set<string>();
    for (const id of targets) {
      try {
        if (await service.testProvider(id)) {
          reachable += 1;
          reachableIds.add(id);
        }
      } catch {
        // A placeholder is expected to be unreachable; keep walking so the
        // summary counts every provider rather than stopping at the first miss.
      }
    }
    let items: StorageProviderView[] = [];
    try {
      items = await refreshProviders();
      setProviders(items);
    } catch {
      // The summary below is still worth showing even if the refresh misses;
      // the health column catches up on the next manual refresh.
    }
    setReachableProviderIds((current) => {
      const next = new Set(current);
      for (const id of reachableIds) {
        next.add(id);
      }
      return next;
    });
    if (items.length > 0) {
      setPlaceholderProviderIds((current) =>
        pruneResolvedPlaceholders(current, items.map((item) => item.id), [...reachableIds]),
      );
    }
    setSelfCheck({
      reachable,
      total: targets.length,
      pending: targets.length - reachable,
    });
    setNotice({
      type: reachable === targets.length ? 'success' : 'error',
      messageKey: 'selfCheckSummary',
      params: {
        total: String(targets.length),
        reachable: String(reachable),
        pending: String(targets.length - reachable),
      },
    });
  };

  /**
   * 一键铺齐内置服务商的账号中心账号、服务商配置与租户默认绑定。
   *
   * 提示语里的计数来自服务端逐行的 `providerCreated` / `accountCreated` /
   * `credentialSeeded`。重复点击时它们全部归零，这正是「没有覆盖运维已填真实密钥」的
   * 可见证据——比一句"操作成功"更能让运维放心继续填密钥。
   */
  const initializeProviderAccounts = () => {
    setPending(true);
    setNotice(undefined);
    service
      .initializeProviderAccountDefaults()
      .then(async (rows) => {
        const items = await refreshProviders();
        // The sweep needs the post-bootstrap option set, not the table page: the
        // providers this run just created may not all fit on page 1.
        const options = await refreshProviderOptions();
        const summary = summarizeProviderAccountDefaults(rows);
        setProviders(items);
        // Keep the vendor vocabulary the server just returned: it is the
        // authority for how the placeholder accounts it minted name their key
        // pair, and the editor renders it instead of the static catalog.
        setVendorCredentialFields((current) => {
          const next = { ...current };
          for (const row of rows) {
            if (row.credentialFields) {
              next[row.providerKind] = row.credentialFields;
            }
          }
          return next;
        });
        // The same response carries the vendor's encryption-mode / storage-class
        // table. Caching it here is what lets the editor offer the values the
        // server would accept instead of the console's own static list.
        setVendorCapabilities((current) => {
          const next = { ...current };
          for (const row of rows) {
            if (row.vendorCapabilities) {
              next[row.providerKind] = row.vendorCapabilities;
            }
          }
          return next;
        });
        // Remember which providers this run seeded with a placeholder, so the
        // list can mark them as pending configuration.
        setPlaceholderProviderIds((current) =>
          mergePlaceholderProviders(
            current,
            rows,
            items.map((item) => item.id),
          ),
        );
        setNotice({
          type: 'success',
          messageKey: 'noticeAccountsInitialized',
          params: {
            total: String(summary.total),
            providers: String(summary.providers),
            accounts: String(summary.accounts),
            credentials: String(summary.credentials),
          },
        });
        // Close the loop the same click opened: a bootstrapped row is only
        // "ready to edit" if it can actually be reached, so the run tests every
        // provider it just settled and reports the split. A placeholder is
        // expected to fail — that is the point: `M 待配置` is the operator's
        // worklist, and `N 可用` is the proof the wiring is right for a vendor
        // whose keys are already real.
        void runPostInitSelfCheck(options, rows.map((row) => row.providerId));
      })
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) {
          setNotice({ type: 'error', messageKey: 'noticeAccountsInitializeFailed' });
        }
      })
      .finally(() => setPending(false));
  };

  // Resolved once per render from the two sets, so the table stays a pure
  // function of its props rather than reaching into page state.
  const pendingPlaceholderIds = new Set(
    providers
      .filter((p) =>
        isPendingPlaceholder(
          p.id,
          placeholderProviderIds,
          reachableProviderIds.has(p.id),
          p.status,
        ),
      )
      .map((p) => p.id),
  );

  // The kind switch is a server filter (`kindQuery` travels with the page
  // request), so the table renders exactly what came back. There is deliberately
  // no second, client-side kind filter here: narrowing one page and presenting
  // that as the filtered result is the defect this page used to have.
  const issueCount = providers.filter(
    (p) =>
      p.status === 'active' &&
      (!p.credentialConfigured ||
        pendingPlaceholderIds.has(p.id) ||
        p.healthStatus === 'unreachable' ||
        p.healthStatus === 'degraded'),
  ).length;

  return (
    <main className="flex h-full flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <div aria-label={t('pageTitle')} className="px-4 pt-4 sm:px-6 sm:pt-6">
        <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex flex-wrap items-center gap-2">
          {!loading && <span className={`${BADGE_BASE_CLASS} bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300`}>{t('headerProviderCount', { count: providerOptions.items.length })}</span>}
          {selfCheck && (
            <span
              className={`${BADGE_BASE_CLASS} ${
                selfCheck.pending === 0
                  ? 'bg-emerald-100 text-emerald-800 dark:bg-emerald-950/40 dark:text-emerald-200'
                  : 'bg-amber-100 text-amber-800 dark:bg-amber-950/40 dark:text-amber-200'
              }`}
              title={t('selfCheckBadgeHint')}
            >
              {t('selfCheckBadge', { reachable: selfCheck.reachable, total: selfCheck.total })}
            </span>
          )}
          {issueCount > 0 && <span className={`${BADGE_BASE_CLASS} bg-amber-100 text-amber-800 dark:bg-amber-950/40 dark:text-amber-200`}>{t('issuesSummary', { count: issueCount })}</span>}
          <select
            aria-label={t('filterByProviderKind')}
            value={kindFilter}
            onChange={(event) => {
              // Changing the kind restarts the cursor chain: page 2 of the old
              // result set is meaningless in the new one, and reusing its cursor
              // would page through a window the operator never asked for.
              setKindFilter(event.target.value);
              setPage(1);
              setPageCursors({ 1: undefined });
            }}
            className={`${SELECT_CLASS} !w-auto !py-1.5 text-xs`}
          >
            <option value="all">{t('filterAllKinds')}</option>
          {/* The whole catalog, one option per kind — including
              `local_filesystem` (every plane has a local row) and the `custom`
              family, which the server matches as `custom:<vendor>`. The catalog
              already carries the `custom` entry, so it is not appended again. */}
          {getAllProviderKindMeta().map((meta) => (
            <option key={String(meta.value)} value={String(meta.value)}>
              {providerKindLabel(t, meta)}
            </option>
          ))}
          </select>
        </div>
          <div className="flex w-full shrink-0 items-center justify-end gap-2 sm:!w-auto">
            <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading} onClick={() => reload()}>
              <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
              {t('refresh')}
            </button>
            <button
              type="button"
              className={SECONDARY_BUTTON_CLASS}
              disabled={pending || loading}
              title={t('initializeAccountsHint')}
              onClick={initializeProviderAccounts}
            >
              <KeyRound aria-hidden="true" size={15} />
              {t('initializeAccounts')}
            </button>
            <button type="button" className={PRIMARY_BUTTON_CLASS} onClick={() => { setEditingProvider(undefined); setEditorOpen(true); }}>
              <Plus aria-hidden="true" size={16} />
              {t('newProvider')}
            </button>
          </div>
        </div>

        {notice && !editorOpen && (
          <div className={`mt-4 flex items-center gap-3 rounded-lg border px-4 py-3 text-sm ${
            notice.type === 'success'
              ? 'border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950/30 dark:text-emerald-200'
              : 'border-red-200 bg-red-50 text-red-800 dark:border-red-900 dark:bg-red-950/30 dark:text-red-200'
          }`}>
            {notice.type === 'success' ? <CheckCircle2 aria-hidden="true" className="shrink-0" size={16} /> : <CircleAlert aria-hidden="true" className="shrink-0" size={16} />}
            <span className="flex-1">{t(notice.messageKey, notice.params)}</span>
            <button type="button" className={ICON_BUTTON_CLASS} aria-label={t('dismiss')} title={t('dismiss')} onClick={() => setNotice(undefined)}><X aria-hidden="true" size={15} /></button>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-auto p-4 sm:p-6">
        {loading ? (
          <div className="flex min-h-[360px] items-center justify-center rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900">
            <div className="flex items-center gap-3 text-sm text-neutral-500">
              <LoaderCircle aria-hidden="true" className="animate-spin" size={19} />
              {t('loading')}
            </div>
          </div>
        ) : (
          <>
            <StorageProviderTable
              providers={providers}
              pendingPlaceholderIds={pendingPlaceholderIds}
              providerKindFilterActive={kindQuery !== undefined}
              onClearProviderKindFilter={() => {
                setKindFilter('all');
                setPage(1);
                setPageCursors({ 1: undefined });
              }}
              actionPending={pending}
              onNewProvider={() => { setEditingProvider(undefined); setEditorOpen(true); }}
              onEditProvider={(p) => { setEditingProvider(p); setEditorOpen(true); }}
              onViewDetail={(p) => { setDetailProvider(p); setDetailDrawerOpen(true); }}
              onActivateProvider={activateProvider}
              onDeactivateProvider={deactivateProvider}
              onTestProvider={testProvider}
              onTestProviders={testProviders}
              onDeleteProvider={deleteProvider}
            />
            <div className="mt-4 flex items-center justify-between gap-3 rounded-lg border border-neutral-200 bg-white px-4 py-3 dark:border-neutral-800 dark:bg-neutral-900">
              <span className="text-sm text-neutral-500">{t('pageLabel', { page })}</span>
              <div className="flex gap-2">
                <button
                  type="button"
                  className={SECONDARY_BUTTON_CLASS}
                  disabled={page <= 1 || loading}
                  onClick={() => setPage((current) => Math.max(1, current - 1))}
                >
                  <ChevronLeft aria-hidden="true" size={16} />
                  <span className="hidden sm:inline">{t('previousPage')}</span>
                </button>
                <button
                  type="button"
                  className={SECONDARY_BUTTON_CLASS}
                  disabled={!hasMore || !nextPageToken || loading}
                  onClick={() => setPage((current) => current + 1)}
                >
                  <span className="hidden sm:inline">{t('nextPage')}</span>
                  <ChevronRight aria-hidden="true" size={16} />
                </button>
              </div>
            </div>
          </>
        )}
      </div>

      {editorOpen && (
        <StorageProviderEditor
          provider={editingProvider}
          // The complete set, not the visible page: the guard exists to stop a
          // duplicate id before the server rejects it, and a page-scoped list
          // would wave through an id that lives on another page.
          existingProviderIds={providerOptions.items.map((item) => item.id)}
          vendorCredentialFields={vendorCredentialFields}
          vendorCapabilities={vendorCapabilities}
          onClose={() => { setEditorOpen(false); setEditingProvider(undefined); }}
          onCreateProvider={createProvider}
          onUpdateProvider={updateProvider}
          onRotateCredential={rotateCredential}
          onProviderSaved={(saved) => {
            setEditingProvider((current) => (current?.id === saved.id ? saved : current));
          }}
          onListProviderAccounts={(input) => service.listProviderAccounts(input)}
          onCreateProviderAccount={(input) => service.createProviderAccount(input)}
        />
      )}

      {detailDrawerOpen && detailProvider && (
        <StorageProviderDetailDrawer
          provider={detailProvider}
          // The drawer's default-binding picker is a provider *switch*: it has
          // to offer the option set, not the rows that happen to share the
          // table's current page and kind.
          providers={providerOptions.items}
          providerOptionsHasMore={providerOptions.hasMore}
          onLoadMoreProviderOptions={providerOptions.loadMore}
          adminStorageSdkClient={adminStorageSdkClient}
          service={service}
          pending={pending}
          onClose={() => { setDetailDrawerOpen(false); setDetailProvider(undefined); }}
          onTestProvider={testProvider}
          onActivateProvider={activateProvider}
          onDeactivateProvider={deactivateProvider}
          onSetDefaultBinding={setDefaultBinding}
          onDeleteDefaultBinding={deleteDefaultBinding}
          onRotateCredential={(id, ref) => {
            void rotateCredential(id, ref);
          }}
        />
      )}
    </main>
  );
}
