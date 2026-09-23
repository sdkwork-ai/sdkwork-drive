/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { StorageProvidersAdminPage } from '../src/pages/StorageProvidersAdminPage';

afterEach(() => cleanup());

const OPERATOR_SESSION = () => ({
  context: {
    tenantId: 'tenant-001',
    userId: 'operator-001',
    actorId: 'operator-001',
  },
});

const LOADED_PROVIDER = {
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
};

function fakeClient(request: unknown) {
  return {
    metadata: {},
    operations: {},
    request,
    setTokenManager: () => undefined,
  } as unknown as DriveAdminStorageSdkClient;
}

describe('StorageProvidersAdminPage', () => {
  it('uses the server returned opaque cursor when loading the next provider page', async () => {
    const request = vi.fn(async ({ operationId }: { operationId: string }) => {
      if (operationId !== 'storageProviders.list') {
        throw new Error(`Unexpected operation ${operationId}`);
      }

      if (request.mock.calls.length === 1) {
        return {
          items: [LOADED_PROVIDER],
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

    render(
      <StorageProvidersAdminPage
        adminStorageSdkClient={fakeClient(request)}
        getSession={OPERATOR_SESSION}
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

  it('bootstraps built-in provider accounts from the header and reloads the table', async () => {
    const operationIds: string[] = [];
    const request = vi.fn(async ({ operationId }: { operationId: string }) => {
      operationIds.push(operationId);
      if (operationId === 'storageProviderAccountDefaults.create') {
        return {
          items: [
            {
              providerKind: 'local_filesystem',
              providerId: 'builtin-storage-provider-local-filesystem',
              providerCreated: true,
            },
            {
              providerKind: 'aliyun_oss',
              providerId: 'builtin-storage-provider-aliyun-oss',
              providerCreated: true,
              vendorCode: 'aliyun',
              providerAccountId: 'iampacct-018f-seeded',
              accountCode: 'builtin-aliyun-storage',
              accountCreated: true,
              credentialSeeded: true,
            },
          ],
        };
      }
      if (operationId === 'storageProviders.list') {
        return { items: [LOADED_PROVIDER], pageInfo: { mode: 'cursor', hasMore: false } };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });

    render(
      <StorageProvidersAdminPage
        adminStorageSdkClient={fakeClient(request)}
        getSession={OPERATOR_SESSION}
      />,
    );

    await screen.findByText('Primary Provider');
    expect(operationIds.filter((id) => id === 'storageProviders.list')).toHaveLength(1);

    fireEvent.click(screen.getByRole('button', { name: 'initializeAccounts' }));

    await waitFor(() =>
      expect(operationIds).toContain('storageProviderAccountDefaults.create'),
    );
    // 初始化必须带回一次列表重拉：新铺的服务商配置只有重拉才会出现在表格里，
    // 否则运维点了按钮却看不到任何变化。
    await waitFor(() =>
      expect(operationIds.filter((id) => id === 'storageProviders.list')).toHaveLength(2),
    );
    // 测试宿主没有挂 LanguageProvider，`t()` 回退到原始 key —— 这里断言的是
    // "走到哪个通知键"，即成功分支而非失败分支。
    await screen.findByText('noticeAccountsInitialized');
  });

  it('reports a failed bootstrap on its own notice key', async () => {
    const request = vi.fn(async ({ operationId }: { operationId: string }) => {
      if (operationId === 'storageProviders.list') {
        return { items: [LOADED_PROVIDER], pageInfo: { mode: 'cursor', hasMore: false } };
      }
      if (operationId === 'storageProviderAccountDefaults.create') {
        throw new Error('bootstrap rejected');
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });

    render(
      <StorageProvidersAdminPage
        adminStorageSdkClient={fakeClient(request)}
        getSession={OPERATOR_SESSION}
      />,
    );

    await screen.findByText('Primary Provider');
    fireEvent.click(screen.getByRole('button', { name: 'initializeAccounts' }));

    await screen.findByText('noticeAccountsInitializeFailed');
  });
});

