import { useCallback, useEffect, useRef, useState } from 'react';
import { isDriveRequestCancellationError } from 'sdkwork-drive-pc-core';
import type { StorageProviderAdminService } from '../services/storageProviderAdminService';
import type { StorageProviderView } from '../types/storageProviderAdminTypes';

/**
 * Page size the provider switches read with: the operation's declared maximum
 * (`PAGINATION_SPEC.md` §3 caps `page_size` at 200).
 *
 * A picker has no page control of its own, so it asks for the largest page the
 * contract allows in one request; a tenant's provider configurations are a small
 * bounded set (one row per catalogued vendor plus whatever an operator adds), so
 * the common case is complete after that single request.
 */
export const PROVIDER_OPTIONS_PAGE_SIZE = 200;

export interface ProviderOptionsState {
  /** Configurations loaded so far, in server order. */
  items: StorageProviderView[];
  /**
   * True when the server reported another page.
   *
   * The switches render a continuation control for this instead of silently
   * presenting a partial list — a picker that hides rows is how "select Tencent
   * COS" came to be missing a provider the tenant actually had.
   */
  hasMore: boolean;
  loading: boolean;
  /** Append the server's next page of the option set. */
  loadMore: () => Promise<StorageProviderView[]>;
  /** Re-read the option set from its first page. */
  reload: (signal?: AbortSignal) => Promise<StorageProviderView[]>;
}

export interface UseProviderOptionsOptions {
  /**
   * Restrict the option set, e.g. to `active` for a switch that must not offer
   * a disabled configuration.
   */
  status?: string;
  /** Skip loading entirely (a host that does not render the switches). */
  enabled?: boolean;
}

/**
 * Paged option set behind the storage console's provider switches.
 *
 * This is deliberately *paged*, not an aggregation: the table (`listProvidersPage`)
 * shows one cursor page at a time, and the switches read one maximum-size page
 * plus an explicit continuation. Nothing here walks the cursor chain on its own,
 * so a picker can neither hide rows silently nor turn a dropdown into an
 * unbounded scan (`PAGINATION_SPEC.md` §7, §8).
 */
export function useProviderOptions(
  service: StorageProviderAdminService,
  options: UseProviderOptionsOptions = {},
): ProviderOptionsState {
  const { status, enabled = true } = options;
  const [items, setItems] = useState<StorageProviderView[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(false);
  // The continuation cursor lives in a ref: it is request plumbing, not render
  // state, and `loadMore` must read the newest value without re-creating itself.
  const nextPageTokenRef = useRef<string | undefined>(undefined);

  const reload = useCallback(
    async (signal?: AbortSignal) => {
      setLoading(true);
      try {
        const page = await service.listProvidersPage({
          status,
          signal,
          pageSize: PROVIDER_OPTIONS_PAGE_SIZE,
        });
        nextPageTokenRef.current = page.nextPageToken;
        setItems(page.items);
        setHasMore(Boolean(page.nextPageToken));
        return page.items;
      } catch (error) {
        if (!isDriveRequestCancellationError(error)) {
          // A failed option read keeps the switches usable with whatever was
          // loaded before; the table's own load notice reports the outage.
          setHasMore(false);
        }
        throw error;
      } finally {
        setLoading(false);
      }
    },
    [service, status],
  );

  const loadMore = useCallback(async () => {
    const pageToken = nextPageTokenRef.current;
    if (!pageToken) {
      return [];
    }
    setLoading(true);
    try {
      const page = await service.listProvidersPage({
        status,
        pageToken,
        pageSize: PROVIDER_OPTIONS_PAGE_SIZE,
      });
      nextPageTokenRef.current = page.nextPageToken;
      setItems((current) => [...current, ...page.items]);
      setHasMore(Boolean(page.nextPageToken));
      return page.items;
    } finally {
      setLoading(false);
    }
  }, [service, status]);

  useEffect(() => {
    if (!enabled) {
      return;
    }
    const controller = new AbortController();
    reload(controller.signal).catch(() => {
      // Reported through the surface that also owns the provider table.
    });
    return () => controller.abort();
  }, [enabled, reload]);

  return { items, hasMore, loading, loadMore, reload };
}
