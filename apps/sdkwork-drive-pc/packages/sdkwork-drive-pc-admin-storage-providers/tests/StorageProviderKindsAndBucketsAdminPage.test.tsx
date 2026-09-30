/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach } from 'vitest';

afterEach(() => cleanup());
import { describe, expect, it, vi } from 'vitest';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { StorageBucketsAdminPage } from '../src/pages/StorageBucketsAdminPage';
import { StorageProviderKindsAdminPage } from '../src/pages/StorageProviderKindsAdminPage';

function createFakeClient(request: unknown) {
  return {
    metadata: {},
    operations: {},
    setTokenManager: () => undefined,
    request,
  } as unknown as DriveAdminStorageSdkClient;
}

const getSession = () => ({
  context: { tenantId: 'tenant-100', userId: 'user-100', actorId: 'operator-100' },
});

describe('StorageProviderKindsAdminPage', () => {
  it('renders the provider catalog with enable states and toggles a kind', async () => {
    const request = vi.fn(async ({ operationId, pathParams, body }: any) => {
      if (operationId === 'storageProviderKinds.list') {
        return {
          items: [
            {
              providerKind: 'aliyun_oss',
              displayName: 'Alibaba Cloud OSS',
              enabled: true,
              sortOrder: 4,
              version: 1,
              configCount: 2,
            },
            {
              providerKind: 'tencent_cos',
              displayName: 'Tencent Cloud COS',
              enabled: false,
              sortOrder: 5,
              version: 1,
              configCount: 0,
            },
          ],
        };
      }
      if (operationId === 'storageProviderKinds.update') {
        return {
          providerKind: pathParams.providerKind,
          displayName: 'Tencent Cloud COS',
          enabled: body.enabled,
          sortOrder: 5,
          version: 2,
          configCount: 0,
        };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const client = createFakeClient(request);

    render(
      <StorageProviderKindsAdminPage adminStorageSdkClient={client} getSession={getSession} />,
    );

    expect(await screen.findByText('Alibaba Cloud OSS')).toBeTruthy();
    expect(await screen.findByText('Tencent Cloud COS')).toBeTruthy();

    // The disabled kind offers an enable action.
    const enableButton = await screen.findByRole('button', { name: /enable/i });
    fireEvent.click(enableButton);

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviderKinds.update'
            && call[0].pathParams.providerKind === 'tencent_cos'
            && call[0].body.enabled === true,
        ),
      ).toBe(true);
    });
  });

  it('initializes the provider catalog from the catalog action', async () => {
    const request = vi.fn(async ({ operationId }: any) => {
      if (operationId === 'storageProviderKinds.list') {
        return { items: [] };
      }
      if (operationId === 'storageProviderKinds.create') {
        return {
          items: [
            {
              providerKind: 's3_compatible',
              displayName: 'Amazon S3 / S3 Compatible',
              enabled: true,
              sortOrder: 2,
              version: 1,
              configCount: 0,
            },
          ],
        };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const client = createFakeClient(request);

    render(
      <StorageProviderKindsAdminPage adminStorageSdkClient={client} getSession={getSession} />,
    );

    const initializeButton = await screen.findByRole('button', { name: 'kindsInitialize' });
    fireEvent.click(initializeButton);

    expect(await screen.findByText('Amazon S3 / S3 Compatible')).toBeTruthy();
    await waitFor(() => {
      expect(
        request.mock.calls.some((call) => call[0].operationId === 'storageProviderKinds.create'),
      ).toBe(true);
    });
  });
});

describe('StorageBucketsAdminPage', () => {
  const twoActiveProviders = [
    {
      id: 'provider-oss-a',
      providerKind: 'aliyun_oss',
      name: 'OSS Hangzhou',
      endpointUrl: 'https://oss-cn-hangzhou.aliyuncs.com',
      region: 'cn-hangzhou',
      bucket: 'drive-hangzhou',
      pathStyle: false,
      credentialConfigured: true,
      status: 'active',
      version: 1,
      strictTls: true,
    },
    {
      id: 'provider-s3-c',
      providerKind: 's3_compatible',
      name: 'S3 Tokyo',
      endpointUrl: 'https://s3.ap-northeast-1.amazonaws.com',
      region: 'ap-northeast-1',
      bucket: 'drive-tokyo',
      pathStyle: false,
      credentialConfigured: false,
      status: 'active',
      version: 1,
      strictTls: true,
    },
    {
      id: 'provider-cos-b',
      providerKind: 'tencent_cos',
      name: 'COS Shanghai',
      endpointUrl: 'https://cos.ap-shanghai.myqcloud.com',
      region: 'ap-shanghai',
      bucket: 'drive-shanghai',
      pathStyle: false,
      credentialConfigured: true,
      status: 'disabled',
      version: 1,
      strictTls: true,
    },
  ];

  it('lists active configurations in the provider nav and syncs buckets automatically', async () => {
    const request = vi.fn(async ({ operationId, pathParams, query }: any) => {
      if (operationId === 'storageProviders.list') {
        // The nav's active-only restriction is a server filter now, so the stub
        // has to answer it like the server does: a client-side filter over one
        // page is what hid every provider past row 20.
        const items = query?.status
          ? twoActiveProviders.filter((provider) => provider.status === query.status)
          : twoActiveProviders;
        return { items, pageInfo: { mode: 'cursor', hasMore: false } };
      }
      if (operationId === 'storageProviders.buckets.list') {
        return {
          items: [
            { bucket: 'drive-hangzhou', configured: true },
          ],
        };
      }
      if (operationId === 'storageProviders.objects.list') {
        return { items: [] };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const client = createFakeClient(request);

    render(<StorageBucketsAdminPage adminStorageSdkClient={client} getSession={getSession} />);

    // Only the active configuration appears in the left provider nav, and the
    // restriction travels to the server instead of narrowing a fetched page.
    expect(await screen.findByRole('button', { name: /OSS Hangzhou/ })).toBeTruthy();
    expect(screen.queryByText('COS Shanghai')).toBeNull();
    expect(request.mock.calls[0][0]).toMatchObject({
      operationId: 'storageProviders.list',
      query: { status: 'active', page_size: 200 },
    });

    // The summary badge stays in the header row.
    expect(screen.getByText('bucketsSummary')).toBeTruthy();

    // Bucket listing syncs without a manual trigger for the selected provider.
    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.buckets.list'
            && call[0].pathParams.providerId === 'provider-oss-a',
        ),
      ).toBe(true);
    });
    expect((await screen.findAllByText('drive-hangzhou')).length).toBeGreaterThan(0);
  });

  it('switches the synced bucket panel when another provider is selected in the nav', async () => {
    const request = vi.fn(async ({ operationId, pathParams }: any) => {
      if (operationId === 'storageProviders.list') {
        return { items: twoActiveProviders };
      }
      if (operationId === 'storageProviders.buckets.list') {
        return {
          items: [
            { bucket: pathParams.providerId === 'provider-s3-c' ? 'drive-tokyo' : 'drive-hangzhou', configured: true },
          ],
        };
      }
      if (operationId === 'storageProviders.objects.list') {
        return { items: [] };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const client = createFakeClient(request);

    render(<StorageBucketsAdminPage adminStorageSdkClient={client} getSession={getSession} />);

    // The first configuration is selected initially; its bucket list loads.
    expect(await screen.findByRole('button', { name: /S3 Tokyo/ })).toBeTruthy();
    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.buckets.list'
            && call[0].pathParams.providerId === 'provider-oss-a',
        ),
      ).toBe(true);
    });

    // Picking the second provider in the nav re-syncs buckets for it.
    fireEvent.click(screen.getByRole('button', { name: /S3 Tokyo/ }));
    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.buckets.list'
            && call[0].pathParams.providerId === 'provider-s3-c',
        ),
      ).toBe(true);
    });
    expect((await screen.findAllByText('drive-tokyo')).length).toBeGreaterThan(0);
  });

  it('initializes the configured bucket idempotently and reports the outcome', async () => {
    let initializeChanged = true;
    const request = vi.fn(async ({ operationId, pathParams }: any) => {
      if (operationId === 'storageProviders.list') {
        return { items: [twoActiveProviders[0]] };
      }
      if (operationId === 'storageProviders.buckets.list') {
        return {
          items: [
            { bucket: 'drive-hangzhou', configured: true },
          ],
        };
      }
      if (operationId === 'storageProviders.bucket.update') {
        return {
          providerId: pathParams.providerId,
          bucket: 'drive-hangzhou',
          changed: initializeChanged,
        };
      }
      if (operationId === 'storageProviders.objects.list') {
        return { items: [] };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const client = createFakeClient(request);

    render(<StorageBucketsAdminPage adminStorageSdkClient={client} getSession={getSession} />);

    expect(await screen.findByRole('button', { name: /OSS Hangzhou/ })).toBeTruthy();

    // First run creates the bucket on the vendor.
    fireEvent.click(await screen.findByRole('button', { name: 'initializeBucket' }));
    await waitFor(() => {
      expect(
        request.mock.calls.some((call) => call[0].operationId === 'storageProviders.bucket.update'),
      ).toBe(true);
    });
    expect(await screen.findByText('bucketInitializedCreated')).toBeTruthy();

    // Re-running after bucket changes stays safe and reports consistency.
    initializeChanged = false;
    fireEvent.click(screen.getByRole('button', { name: 'initializeBucket' }));
    expect(await screen.findByText('bucketInitializedConsistent')).toBeTruthy();
  });

  it('offers quick-configuration shortcuts from the header', async () => {
    const request = vi.fn(async ({ operationId }: any) => {
      if (operationId === 'storageProviders.list') {
        return { items: [] };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const client = createFakeClient(request);
    const onConfigureProviders = vi.fn();
    const onCreateProvider = vi.fn();

    render(
      <StorageBucketsAdminPage
        adminStorageSdkClient={client}
        getSession={getSession}
        onConfigureProviders={onConfigureProviders}
        onCreateProvider={onCreateProvider}
      />,
    );

    // Each label appears twice (header + empty state), so scope to the header.
    const header = screen.getByRole('banner');
    fireEvent.click(within(header).getByRole('button', { name: 'bucketsQuickCreate' }));
    expect(onCreateProvider).toHaveBeenCalledTimes(1);

    fireEvent.click(within(header).getByRole('button', { name: 'bucketsQuickConfigure' }));
    expect(onConfigureProviders).toHaveBeenCalledTimes(1);

    // The empty state must carry the same hand-off for an operator who lands
    // here with nothing configured. It only appears once loading settles.
    const emptyStateTitle = await screen.findByText('bucketsNoActiveProvidersTitle');
    const emptyState = emptyStateTitle.closest('div.rounded-lg') as HTMLElement;
    fireEvent.click(within(emptyState).getByRole('button', { name: 'bucketsQuickCreate' }));
    expect(onCreateProvider).toHaveBeenCalledTimes(2);
  });

  it('hides the provider hand-off shortcuts when the host does not mount them', async () => {
    const request = vi.fn(async ({ operationId }: any) => {
      if (operationId === 'storageProviders.list') {
        return { items: [] };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const client = createFakeClient(request);

    render(<StorageBucketsAdminPage adminStorageSdkClient={client} getSession={getSession} />);

    // The webserver host mounts neither provider surface, so the hand-off
    // buttons must not render as dead controls.
    const emptyStateTitle = await screen.findByText('bucketsNoActiveProvidersTitle');
    const emptyState = emptyStateTitle.closest('div.rounded-lg') as HTMLElement;
    expect(within(emptyState).queryByRole('button', { name: 'bucketsQuickCreate' })).toBeNull();
    expect(within(emptyState).queryByRole('button', { name: 'bucketsQuickConfigure' })).toBeNull();
    expect(within(emptyState).getByRole('button', { name: 'refresh' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'bucketsQuickCreate' })).toBeNull();
  });
});
