/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
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
      if (operationId === 'storageProviderKinds.initialize') {
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
        request.mock.calls.some((call) => call[0].operationId === 'storageProviderKinds.initialize'),
      ).toBe(true);
    });
  });
});

describe('StorageBucketsAdminPage', () => {
  it('loads active configurations and selects one via the provider dropdown', async () => {
    const request = vi.fn(async ({ operationId, pathParams }: any) => {
      if (operationId === 'storageProviders.list') {
        return {
          items: [
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
          ],
        };
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

    // Only the active configuration appears in the dropdown options.
    const select = (await screen.findByLabelText('bucketsSelectProvider')) as HTMLSelectElement;
    expect(select.options).toHaveLength(1);
    expect(select.options[0].textContent).toContain('[OSS]');
    expect(select.options[0].textContent).toContain('OSS Hangzhou');

    // Bucket listing is triggered from the bucket panel action.
    fireEvent.click(await screen.findByRole('button', { name: 'listAll' }));

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.buckets.list'
            && call[0].pathParams.providerId === 'provider-oss-a',
        ),
      ).toBe(true);
    });
  });
});
