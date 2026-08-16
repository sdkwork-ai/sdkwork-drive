import { describe, expect, it } from 'vitest';
import { DEFAULT_LIST_PAGE_SIZE } from '@sdkwork/utils';
import { normalizeBackendOffsetListPage } from '../src/utils/normalizeBackendOffsetListPage';

describe('normalizeBackendOffsetListPage', () => {
  it('maps SdkWorkPageData pageInfo totals into admin offset list views', () => {
    const page = normalizeBackendOffsetListPage({
      items: [{ id: 1 }],
      pageInfo: {
        mode: 'offset',
        page: 2,
        pageSize: 25,
        totalItems: '42',
      },
    });

    expect(page).toEqual({
      items: [{ id: 1 }],
      page: 2,
      pageSize: 25,
      total: 42,
      pageInfo: {
        mode: 'offset',
        page: 2,
        pageSize: 25,
        totalItems: '42',
      },
    });
  });

  it('defaults page size from sdkwork utils when pageInfo is absent', () => {
    const page = normalizeBackendOffsetListPage({
      items: [{ id: 1 }],
    });

    expect(page.pageSize).toBe(DEFAULT_LIST_PAGE_SIZE);
    expect(page.total).toBe(1);
  });

  it('keeps legacy flat page fields when pageInfo is absent', () => {
    const page = normalizeBackendOffsetListPage({
      items: [{ id: 1 }],
      page: 1,
      pageSize: 10,
      total: 3,
    });

    expect(page.total).toBe(3);
    expect(page.pageSize).toBe(10);
  });

  it('preserves opaque cursor metadata returned by backend list responses', () => {
    const page = normalizeBackendOffsetListPage({
      items: [{ id: 1 }],
      pageInfo: {
        mode: 'cursor',
        pageSize: 20,
        hasMore: true,
        nextCursor: 'opaque-next',
      },
    });

    expect(page.pageInfo).toEqual({
      mode: 'cursor',
      pageSize: 20,
      hasMore: true,
      nextCursor: 'opaque-next',
    });
  });
});
