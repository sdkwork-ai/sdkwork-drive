import { DEFAULT_LIST_PAGE_SIZE, type PageInfo } from '@sdkwork/utils';

/** Wire shape from backend SDK before admin table normalization. */
export type BackendOffsetListPageInfoWire = {
  mode?: PageInfo['mode'];
  page?: number;
  pageSize?: number;
  totalItems?: PageInfo['totalItems'];
  totalPages?: number;
  hasMore?: boolean;
  nextCursor?: string | null;
};

export type BackendOffsetListWire<T> = {
  items: T[];
  pageInfo?: BackendOffsetListPageInfoWire;
  page?: number;
  pageSize?: number;
  total?: number;
};

export interface BackendOffsetListPage<T> {
  items: T[];
  page: number;
  pageSize: number;
  total: number;
  pageInfo?: BackendOffsetListPageInfoWire;
}

function parseTotalItems(totalItems: PageInfo['totalItems'], fallback?: number): number {
  if (totalItems != null && totalItems.trim() !== '') {
    const parsed = Number(totalItems);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }
  return fallback ?? 0;
}

/** Map SdkWorkApiResponse offset list payloads into admin table pagination state. */
export function normalizeBackendOffsetListPage<T>(
  payload: BackendOffsetListWire<T>,
): BackendOffsetListPage<T> {
  const page = payload.pageInfo?.page ?? payload.page ?? 1;
  const pageSize = payload.pageInfo?.pageSize ?? payload.pageSize ?? DEFAULT_LIST_PAGE_SIZE;
  const total = parseTotalItems(payload.pageInfo?.totalItems, payload.total ?? payload.items.length);

  return {
    items: payload.items,
    page,
    pageSize,
    total,
    pageInfo: payload.pageInfo,
  };
}
