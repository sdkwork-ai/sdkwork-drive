/* @vitest-environment jsdom */

import { act, cleanup, renderHook, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type {
  DriveAdminStorageSdkClient,
  DriveAdminStorageSdkRequest,
} from 'sdkwork-drive-pc-admin-core';
import { createStorageProviderAdminService } from '../src/services/storageProviderAdminService';
import {
  PROVIDER_OPTIONS_PAGE_SIZE,
  useProviderOptions,
} from '../src/hooks/useProviderOptions';

afterEach(() => cleanup());

function providerRow(id: string): unknown {
  return {
    id,
    providerKind: 'tencent_cos',
    name: id,
    endpointUrl: 'https://cos.ap-shanghai.myqcloud.com',
    bucket: 'drive-prod',
    pathStyle: false,
    status: 'active',
    version: 1,
    credentialConfigured: true,
  };
}

function createHarness(pages: unknown[]) {
  const calls: DriveAdminStorageSdkRequest[] = [];
  const client = {
    metadata: {},
    operations: {},
    setTokenManager: () => undefined,
    async request<T>(request: DriveAdminStorageSdkRequest): Promise<T> {
      calls.push(request);
      return (pages[calls.length - 1] ?? { items: [], pageInfo: { hasMore: false } }) as T;
    },
  } as unknown as DriveAdminStorageSdkClient;

  const service = createStorageProviderAdminService({
    adminStorageSdkClient: client,
    getSession: () => ({
      context: { tenantId: 'tenant-100', userId: 'user-100', actorId: 'operator-100' },
    }),
  });

  return { calls, service };
}

describe('useProviderOptions', () => {
  it('reads one maximum-size option page instead of walking the cursor chain', async () => {
    const { calls, service } = createHarness([
      {
        items: [providerRow('provider-a')],
        pageInfo: { mode: 'cursor', hasMore: true, nextCursor: 'options-2' },
      },
      {
        items: [providerRow('provider-z-tencent-cos')],
        pageInfo: { mode: 'cursor', hasMore: false },
      },
    ]);

    const { result } = renderHook(() => useProviderOptions(service, { status: 'active' }));

    await waitFor(() => expect(result.current.items).toHaveLength(1));

    // One request, at the contract's maximum page size, and no self-driven
    // paging: an aggregation helper in a UI facade is exactly what
    // PAGINATION_SPEC §7 reserves for export/batch.
    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      operationId: 'storageProviders.list',
      query: { status: 'active', page_size: PROVIDER_OPTIONS_PAGE_SIZE },
    });
    expect(result.current.hasMore).toBe(true);
  });

  it('appends the next option page on demand and clears the continuation', async () => {
    const { calls, service } = createHarness([
      {
        items: [providerRow('provider-a')],
        pageInfo: { mode: 'cursor', hasMore: true, nextCursor: 'options-2' },
      },
      {
        items: [providerRow('provider-z-tencent-cos')],
        pageInfo: { mode: 'cursor', hasMore: false },
      },
    ]);

    const { result } = renderHook(() => useProviderOptions(service));

    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => {
      await result.current.loadMore();
    });

    // The provider past the first page — the one the old page-scoped switch
    // could never show — is now selectable.
    expect(result.current.items.map((item) => item.id)).toEqual([
      'provider-a',
      'provider-z-tencent-cos',
    ]);
    expect(result.current.hasMore).toBe(false);
    expect(calls[1]).toMatchObject({
      operationId: 'storageProviders.list',
      query: { page_size: PROVIDER_OPTIONS_PAGE_SIZE, cursor: 'options-2' },
    });
  });

  it('re-reads the option set from its first page when a mutation invalidates it', async () => {
    const { calls, service } = createHarness([
      {
        items: [providerRow('provider-a')],
        pageInfo: { mode: 'cursor', hasMore: false },
      },
      {
        items: [providerRow('builtin-storage-provider-tencent-cos')],
        pageInfo: { mode: 'cursor', hasMore: false },
      },
    ]);

    const { result } = renderHook(() => useProviderOptions(service));
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => {
      await result.current.reload();
    });

    expect(result.current.items.map((item) => item.id)).toEqual([
      'builtin-storage-provider-tencent-cos',
    ]);
    expect(calls[1]).toMatchObject({
      operationId: 'storageProviders.list',
      query: { page_size: PROVIDER_OPTIONS_PAGE_SIZE, cursor: undefined },
    });
  });

  it('keeps the previously loaded options when a re-read fails', async () => {
    const calls: DriveAdminStorageSdkRequest[] = [];
    const client = {
      metadata: {},
      operations: {},
      setTokenManager: () => undefined,
      async request<T>(request: DriveAdminStorageSdkRequest): Promise<T> {
        calls.push(request);
        if (calls.length === 1) {
          return {
            items: [providerRow('provider-a')],
            pageInfo: { mode: 'cursor', hasMore: false },
          } as T;
        }
        throw new Error('option read failed');
      },
    } as unknown as DriveAdminStorageSdkClient;
    const service = createStorageProviderAdminService({
      adminStorageSdkClient: client,
      getSession: () => ({
        context: { tenantId: 'tenant-100', userId: 'user-100', actorId: 'operator-100' },
      }),
    });

    const { result } = renderHook(() => useProviderOptions(service));
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => {
      await expect(result.current.reload()).rejects.toThrow('option read failed');
    });

    // A switch that keeps its last known options is a better answer than an
    // empty dropdown; the surface's own load notice reports the outage.
    expect(result.current.items.map((item) => item.id)).toEqual(['provider-a']);
    expect(result.current.hasMore).toBe(false);
  });
});
