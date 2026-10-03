import React, { useEffect, useMemo, useState } from 'react';
import { Search } from 'lucide-react';
import type { StorageProviderView } from '../types/storageProviderAdminTypes';
import {
  getAllProviderKindMeta,
  getProviderKindMeta,
  providerDisplayName,
  providerKindLabel,
} from '../utils/providerKindConfig';
import { useTranslation } from '../hooks/useTranslation';

interface StorageProviderNavListProps {
  providers: StorageProviderView[];
  selectedProviderId: string;
  onSelect: (providerId: string) => void;
  /**
   * True when the option set continues past what is loaded.
   *
   * The rail reads the provider list with the maximum page size, so this is the
   * degenerate case (more configurations than one page holds). Rendering a
   * continuation keeps the rail honest: without it the missing providers would
   * be indistinguishable from providers that do not exist.
   */
  hasMore?: boolean;
  loadingMore?: boolean;
  onLoadMore?: () => void;
}

const SEARCH_INPUT_CLASS =
  'h-8 w-full rounded-md border border-neutral-300 bg-white pl-8 pr-2 text-[13px] text-neutral-900 outline-none transition-colors placeholder:text-neutral-400 focus:border-blue-500 focus:ring-1 focus:ring-blue-500/20 dark:border-neutral-600 dark:bg-neutral-900 dark:text-neutral-100 dark:placeholder:text-neutral-500 dark:focus:border-blue-400 dark:focus:ring-blue-400/20';

/**
 * Credential filter of the rail, in display order.
 *
 * The predicate is the server's `credentialConfigured` — the very flag the dot
 * at the end of each row already renders, so a tab and a dot can never disagree
 * about the same configuration.
 */
type StorageProviderCredentialFilter = 'configured' | 'missing' | 'all';

const CREDENTIAL_FILTERS: readonly StorageProviderCredentialFilter[] = [
  'configured',
  'missing',
  'all',
];

const CREDENTIAL_FILTER_LABEL_KEYS: Record<StorageProviderCredentialFilter, string> = {
  configured: 'bucketsProviderFilterConfigured',
  missing: 'bucketsProviderFilterMissing',
  all: 'bucketsProviderFilterAll',
};

/**
 * Tab the rail opens on.
 *
 * The list exists to answer "which configurations can I actually browse right
 * now", and on a tenant where some rows were never given keys that is the
 * configured set; `全部` stays one click away for the audit view.
 */
const DEFAULT_CREDENTIAL_FILTER: StorageProviderCredentialFilter = 'configured';

/** Whether a configuration belongs to a tab, independent of the search box. */
function matchesCredentialFilter(
  provider: StorageProviderView,
  filter: StorageProviderCredentialFilter,
): boolean {
  if (filter === 'configured') return provider.credentialConfigured;
  if (filter === 'missing') return !provider.credentialConfigured;
  return true;
}

/**
 * Left-hand provider rail for the bucket page. One calm, single-line row per
 * configuration: vendor chip as the recognizable icon, display name, and a
 * credential dot. Bucket and endpoint details stay in the detail pane; they
 * surface here only as the row hover title.
 *
 * Two filters narrow the same list and compose: the credential tabs answer
 * "which configurations still need keys", the search box answers "where is the
 * one I am thinking of". Both run client-side over what is loaded — the rail
 * reads one maximum-size page plus an explicit continuation, so a filter can
 * never pretend the page it holds is the whole tenant.
 */
export function StorageProviderNavList({
  providers,
  selectedProviderId,
  onSelect,
  hasMore,
  loadingMore,
  onLoadMore,
}: StorageProviderNavListProps) {
  const { t } = useTranslation();
  const [query, setQuery] = useState('');
  const [credentialFilter, setCredentialFilter] =
    useState<StorageProviderCredentialFilter>(DEFAULT_CREDENTIAL_FILTER);

  const kindOrder = useMemo(() => {
    const order = new Map<string, number>();
    getAllProviderKindMeta().forEach((meta, index) => order.set(meta.value, index));
    return order;
  }, []);

  const counts = useMemo(() => {
    let configured = 0;
    for (const provider of providers) {
      if (provider.credentialConfigured) configured += 1;
    }
    return {
      configured,
      missing: providers.length - configured,
      all: providers.length,
    };
  }, [providers]);

  const items = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    return providers
      .filter((provider) => {
        if (!matchesCredentialFilter(provider, credentialFilter)) return false;
        if (!normalizedQuery) return true;
        const meta = getProviderKindMeta(provider.providerKind);
        // The haystack carries both spellings of the name and of the vendor:
        // an operator who reads `内置 阿里云 OSS` in the list must be able to
        // find it by typing either that or the stored English name.
        const haystack = [
          providerDisplayName(t, provider),
          provider.displayName,
          provider.bucket,
          provider.providerKind,
          providerKindLabel(t, meta),
          meta.label,
          provider.endpointUrl,
          provider.region ?? '',
        ]
          .join(' ')
          .toLowerCase();
        return haystack.includes(normalizedQuery);
      })
      .map((provider) => ({
        provider,
        meta: getProviderKindMeta(provider.providerKind),
        displayName: providerDisplayName(t, provider),
      }))
      // Ordered by what the operator reads, not by what the row stores: after
      // localization the stored names are no longer the visible sort key.
      .sort(
        (a, b) =>
          (kindOrder.get(a.meta.value) ?? Number.MAX_SAFE_INTEGER)
            - (kindOrder.get(b.meta.value) ?? Number.MAX_SAFE_INTEGER)
          || a.displayName.localeCompare(b.displayName),
      );
  }, [credentialFilter, providers, query, kindOrder, t]);

  /*
   * Keep the detail pane inside the tab the operator is looking at.
   *
   * With `已配置` as the opening tab, a host that picked the first configuration
   * of the *unfiltered* option set would otherwise open on a row the rail does
   * not even show. The reconciliation deliberately ignores the search box: a
   * keystroke must not reload the bucket list of whichever row happens to match
   * first, so a search that hides the selection leaves the pane where it is.
   *
   * An empty tab changes nothing — hiding the pane would be worse than leaving
   * the last valid selection in place.
   */
  useEffect(() => {
    const inTab = providers.filter((provider) =>
      matchesCredentialFilter(provider, credentialFilter));
    if (inTab.length === 0 || inTab.some((provider) => provider.id === selectedProviderId)) {
      return;
    }
    onSelect(inTab[0].id);
  }, [credentialFilter, onSelect, providers, selectedProviderId]);

  const emptyMessage = query.trim()
    ? t('bucketsProviderNoMatch')
    : credentialFilter === 'configured'
      ? t('bucketsProviderEmptyConfigured')
      : credentialFilter === 'missing'
        ? t('bucketsProviderEmptyMissing')
        : t('bucketsProviderNoMatch');

  return (
    <aside
      aria-label={t('bucketsProviderListLabel')}
      className="flex max-h-[42vh] shrink-0 flex-col border-b border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900 lg:max-h-none lg:w-[320px] lg:border-b-0 lg:border-r"
    >
      <div className="shrink-0 border-b border-neutral-100 px-3 py-2.5 dark:border-neutral-800">
        <div
          role="group"
          aria-label={t('bucketsProviderFilterLabel')}
          className="mb-2 flex items-center gap-0.5 rounded-md bg-neutral-100 p-0.5 dark:bg-neutral-800"
        >
          {CREDENTIAL_FILTERS.map((value) => {
            const active = value === credentialFilter;
            return (
              <button
                key={value}
                type="button"
                aria-pressed={active}
                onClick={() => setCredentialFilter(value)}
                className={`flex min-w-0 flex-1 items-center justify-center gap-1 rounded px-1.5 py-1 text-[11px] font-medium transition-colors ${
                  active
                    ? 'bg-white text-neutral-900 shadow-sm dark:bg-neutral-700 dark:text-neutral-100'
                    : 'text-neutral-500 hover:text-neutral-800 dark:text-neutral-400 dark:hover:text-neutral-200'
                }`}
              >
                <span className="truncate">{t(CREDENTIAL_FILTER_LABEL_KEYS[value])}</span>
                <span className="shrink-0 tabular-nums text-neutral-400 dark:text-neutral-500">
                  {counts[value]}
                </span>
              </button>
            );
          })}
        </div>
        <div className="relative">
          <Search
            aria-hidden="true"
            className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-neutral-400"
            size={14}
          />
          <input
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t('bucketsProviderSearchPlaceholder')}
            aria-label={t('bucketsProviderSearchPlaceholder')}
            className={SEARCH_INPUT_CLASS}
          />
        </div>
        {hasMore ? (
          // The tab counts cover what is loaded. Saying so is what keeps a
          // partial page from reading as the whole tenant.
          <p className="mt-1.5 text-[11px] leading-relaxed text-neutral-400 dark:text-neutral-500">
            {t('providerOptionsPartial', { count: providers.length })}
          </p>
        ) : null}
      </div>

      <nav className="min-h-0 flex-1 overflow-y-auto px-2 py-2">
        <div className="flex flex-col gap-0.5">
          {items.map(({ provider, meta, displayName }) => {
            const selected = provider.id === selectedProviderId;
            return (
              <button
                key={provider.id}
                type="button"
                onClick={() => onSelect(provider.id)}
                aria-current={selected ? 'true' : undefined}
                title={`${displayName} · ${provider.bucket}${provider.region ? ` · ${provider.region}` : ''}`}
                className={`flex w-full items-center gap-2.5 rounded-md border-l-2 px-2.5 py-2 text-left transition-colors ${
                  selected
                    ? 'border-blue-600 bg-blue-50 dark:border-blue-400 dark:bg-blue-950/40'
                    : 'border-transparent hover:bg-neutral-100 dark:hover:bg-neutral-800/60'
                }`}
              >
                <span
                  aria-hidden="true"
                  className={`flex h-[22px] w-10 shrink-0 items-center justify-center rounded-md text-[10px] font-bold tracking-wide ${meta.bgClass} ${meta.textClass}`}
                >
                  {meta.icon}
                </span>
                <span
                  className={`min-w-0 flex-1 truncate text-[13px] font-medium ${
                    selected
                      ? 'text-blue-900 dark:text-blue-100'
                      : 'text-neutral-800 dark:text-neutral-200'
                  }`}
                >
                  {displayName}
                </span>
                <span
                  className={`h-1.5 w-1.5 shrink-0 rounded-full ${
                    provider.credentialConfigured ? 'bg-emerald-500' : 'bg-amber-500'
                  }`}
                  title={provider.credentialConfigured ? t('credentialSet') : t('credentialMissing')}
                />
              </button>
            );
          })}
        </div>
        {items.length === 0 ? (
          <p className="px-2 py-6 text-center text-xs text-neutral-400 dark:text-neutral-500">
            {emptyMessage}
          </p>
        ) : null}
        {hasMore && onLoadMore ? (
          <button
            type="button"
            className="mt-1 w-full rounded-md border border-neutral-200 px-2 py-1.5 text-xs font-medium text-neutral-600 transition-colors hover:bg-neutral-100 disabled:opacity-50 dark:border-neutral-700 dark:text-neutral-300 dark:hover:bg-neutral-800"
            disabled={loadingMore}
            onClick={onLoadMore}
          >
            {loadingMore ? t('loading') : t('providerOptionsLoadMore')}
          </button>
        ) : null}
      </nav>
    </aside>
  );
}
