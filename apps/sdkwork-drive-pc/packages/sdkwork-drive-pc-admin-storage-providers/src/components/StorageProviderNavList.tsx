import React, { useMemo, useState } from 'react';
import { Search } from 'lucide-react';
import type { StorageProviderView } from '../types/storageProviderAdminTypes';
import { getAllProviderKindMeta, getProviderKindMeta } from '../utils/providerKindConfig';
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
 * Left-hand provider rail for the bucket page. One calm, single-line row per
 * configuration: vendor chip as the recognizable icon, display name, and a
 * credential dot. Bucket and endpoint details stay in the detail pane; they
 * surface here only as the row hover title.
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

  const kindOrder = useMemo(() => {
    const order = new Map<string, number>();
    getAllProviderKindMeta().forEach((meta, index) => order.set(meta.value, index));
    return order;
  }, []);

  const items = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    return providers
      .filter((provider) => {
        if (!normalizedQuery) return true;
        const haystack = [
          provider.displayName,
          provider.bucket,
          provider.providerKind,
          provider.endpointUrl,
          provider.region ?? '',
        ]
          .join(' ')
          .toLowerCase();
        return haystack.includes(normalizedQuery);
      })
      .map((provider) => ({ provider, meta: getProviderKindMeta(provider.providerKind) }))
      .sort(
        (a, b) =>
          (kindOrder.get(a.meta.value) ?? Number.MAX_SAFE_INTEGER)
            - (kindOrder.get(b.meta.value) ?? Number.MAX_SAFE_INTEGER)
          || a.provider.displayName.localeCompare(b.provider.displayName),
      );
  }, [providers, query, kindOrder]);

  return (
    <aside
      aria-label={t('bucketsProviderListLabel')}
      className="flex max-h-[42vh] shrink-0 flex-col border-b border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900 lg:max-h-none lg:w-[264px] lg:border-b-0 lg:border-r"
    >
      <div className="shrink-0 border-b border-neutral-100 px-3 py-2.5 dark:border-neutral-800">
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
      </div>

      <nav className="min-h-0 flex-1 overflow-y-auto px-2 py-2">
        <div className="flex flex-col gap-0.5">
          {items.map(({ provider, meta }) => {
            const selected = provider.id === selectedProviderId;
            return (
              <button
                key={provider.id}
                type="button"
                onClick={() => onSelect(provider.id)}
                aria-current={selected ? 'true' : undefined}
                title={`${provider.bucket}${provider.region ? ` · ${provider.region}` : ''}`}
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
                  {provider.displayName}
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
            {t('bucketsProviderNoMatch')}
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
