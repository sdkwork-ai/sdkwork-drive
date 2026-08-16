/* @vitest-environment jsdom */

import React from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { StorageProvidersAdminPage } from '../src/pages/StorageProvidersAdminPage';

describe('StorageProvidersAdminPage', () => {
  it('uses the server returned opaque cursor when loading the next provider page', async () => {
    const request = vi.fn(async ({ operationId }: { operationId: string }) => {
      if (operationId !== 'storageProviders.list') {
        throw new Error(`Unexpected operation ${operationId}`);
      }

      if (request.mock.calls.length === 1) {
        return {
          items: [{
            id: 'provider-primary',
            providerKind: 's3_compatible',
            name: 'Primary Provider',
            endpointUrl: 'https://s3.example.com',
            bucket: 'drive-primary',
            pathStyle: false,
            credentialConfigured: true,
            status: 'active',
            version: 1,
            strictTls: true,
          }],
          pageInfo: {
            mode: 'cursor',
            hasMore: true,
            nextCursor: 'opaque-provider-next',
          },
        };
      }

      return {
        items: [{
          id: 'provider-archive',
          providerKind: 's3_compatible',
          name: 'Archive Provider',
          endpointUrl: 'https://archive.example.com',
          bucket: 'drive-archive',
          pathStyle: false,
          credentialConfigured: true,
          status: 'active',
          version: 1,
          strictTls: true,
        }],
        pageInfo: {
          mode: 'cursor',
          hasMore: false,
        },
      };
    });
    const adminStorageSdkClient = {
      metadata: {},
      operations: {},
      request,
      setTokenManager: () => undefined,
    } as unknown as DriveAdminStorageSdkClient;

    render(
      <StorageProvidersAdminPage
        adminStorageSdkClient={adminStorageSdkClient}
        getSession={() => ({
          context: {
            tenantId: 'tenant-001',
            userId: 'operator-001',
            actorId: 'operator-001',
          },
        })}
      />,
    );

    await screen.findByText('Primary Provider');

    fireEvent.click(screen.getByRole('button', { name: 'nextPage' }));

    await waitFor(() => expect(request).toHaveBeenCalledTimes(2));
    expect(request.mock.calls[1]?.[0]).toMatchObject({
      operationId: 'storageProviders.list',
      query: {
        page_size: 20,
        cursor: 'opaque-provider-next',
      },
    });
  });
});
