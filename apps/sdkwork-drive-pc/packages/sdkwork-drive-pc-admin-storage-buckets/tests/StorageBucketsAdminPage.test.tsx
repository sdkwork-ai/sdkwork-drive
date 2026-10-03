/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { StorageBucketsAdminPage } from '../src/pages/StorageBucketsAdminPage';

afterEach(() => cleanup());

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

const OSS_PROVIDER = {
  id: 'provider-oss-a',
  providerKind: 'aliyun_oss',
  displayName: 'OSS Hangzhou',
  endpointUrl: 'https://oss-cn-hangzhou.aliyuncs.com',
  region: 'cn-hangzhou',
  bucket: 'drive-hangzhou',
  pathStyle: false,
  credentialConfigured: true,
  status: 'active',
  version: 1,
  strictTls: true,
};

const COS_PROVIDER = {
  ...OSS_PROVIDER,
  id: 'provider-cos-b',
  providerKind: 'tencent_cos',
  displayName: 'COS Guangzhou',
  endpointUrl: 'https://cos.ap-guangzhou.myqcloud.com',
  region: 'ap-guangzhou',
  bucket: 'drive-guangzhou',
};

describe('StorageBucketsAdminPage', () => {
  it('lists the buckets of the selected provider account and browses a picked bucket', async () => {
    const request = vi.fn(async ({ operationId, pathParams, query }: any) => {
      if (operationId === 'storageProviders.list') {
        // 只读 active 的限制是服务端过滤，桩要像服务端那样回答。
        const items = query?.status
          ? [OSS_PROVIDER, COS_PROVIDER].filter((provider) => provider.status === query.status)
          : [OSS_PROVIDER, COS_PROVIDER];
        return { items, pageInfo: { mode: 'cursor', hasMore: false } };
      }
      if (operationId === 'storageProviders.buckets.list') {
        return {
          items:
            pathParams.providerId === OSS_PROVIDER.id
              ? [
                  {
                    bucket: 'archive-2024',
                    configured: false,
                    creationDateEpochMs: Date.UTC(2024, 0, 2),
                    // 账号级清单里每一行自带地域：跨地域的桶必须带着它去读文件。
                    region: 'cn-beijing',
                  },
                  {
                    bucket: 'drive-hangzhou',
                    configured: true,
                    creationDateEpochMs: Date.UTC(2023, 4, 1),
                    region: 'cn-hangzhou',
                  },
                ]
              : [{ bucket: 'drive-guangzhou', configured: true, region: 'ap-guangzhou' }],
        };
      }
      if (operationId === 'storageProviders.objects.list') {
        return {
          items: [
            { objectKey: 'docs/', objectKind: 'prefix' },
            { objectKey: 'readme.md', objectKind: 'object', contentLength: 2048 },
          ],
          pageInfo: { hasMore: false },
        };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const client = createFakeClient(request);

    render(<StorageBucketsAdminPage adminStorageSdkClient={client} getSession={getSession} />);

    // 左栏读的是选项集：请求本身就把 active 与最大页带出去。
    expect(await screen.findByRole('button', { name: /OSS Hangzhou/ })).toBeTruthy();
    expect(request.mock.calls[0][0]).toMatchObject({
      operationId: 'storageProviders.list',
      query: { status: 'active', page_size: 200 },
    });

    // 选中配置的桶列表来自厂商 ListBuckets，并且标出了配置桶。
    expect(await screen.findByText('archive-2024')).toBeTruthy();
    // 所属地域是清单里的字段，直接列在表里。
    expect(await screen.findByText('cn-beijing')).toBeTruthy();
    // 配置桶同时出现在配置条与列表行里，两处都必须读得到。
    expect((await screen.findAllByText('drive-hangzhou')).length).toBeGreaterThan(1);
    // 列表里的每一行都有用途：配置桶是写入目标，其余只能浏览（旧"状态"列只有一行有值）。
    expect(await screen.findByText('bucketsRoleWriteTarget')).toBeTruthy();
    expect(await screen.findByText('bucketsRoleBrowseOnly')).toBeTruthy();

    // 对象平面在打开弹窗之前不读。
    expect(
      request.mock.calls.some((call) => call[0].operationId === 'storageProviders.objects.list'),
    ).toBe(false);

    // 点开非配置桶：请求必须带上这个桶**和它的地域**，而不是回落到配置桶/配置地域。
    const browseButtons = await screen.findAllByRole('button', { name: 'bucketsBrowseFiles' });
    fireEvent.click(browseButtons[0]);

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.list'
            && call[0].pathParams.providerId === OSS_PROVIDER.id
            && call[0].query.bucket === 'archive-2024'
            && call[0].query.region === 'cn-beijing',
        ),
      ).toBe(true);
    });
    expect(await screen.findByText('readme.md')).toBeTruthy();
    expect(await screen.findByText('docs/')).toBeTruthy();

    // 文件列表点击文件即预览，点文件夹进入下一层：夹在列表里的行不再需要单独按钮。
    expect(screen.queryByText('bucketsBrowseClose')).toBeNull();

    // 关闭弹窗即卸载浏览器：头部不再有页脚按钮，关闭入口是对话框自带的关闭按钮。
    fireEvent.click(screen.getByRole('button', { name: 'Close' }));
    await waitFor(() => {
      expect(screen.queryByText('readme.md')).toBeNull();
    });
  });

  it('names the provider account the listed buckets come from', async () => {
    const boundProvider = { ...OSS_PROVIDER, providerAccountId: 'account-42' };
    const request = vi.fn(async ({ operationId }: any) => {
      if (operationId === 'storageProviders.list') {
        return { items: [boundProvider], pageInfo: { hasMore: false } };
      }
      if (operationId === 'storageProviders.buckets.list') {
        return { items: [{ bucket: 'drive-hangzhou', configured: true }] };
      }
      if (operationId === 'storageProviderAccounts.list') {
        return {
          items: [
            {
              id: 'account-42',
              scopeType: 'tenant',
              isDefault: true,
              vendorCode: 'aliyun',
              accountCode: 'ops-cn',
              displayName: '运维主账号',
              accountType: 'ak',
              environment: 'production',
              capabilityCodes: [],
              status: 'active',
            },
          ],
          pageInfo: { hasMore: false },
        };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });

    render(
      <StorageBucketsAdminPage
        adminStorageSdkClient={createFakeClient(request)}
        getSession={getSession}
      />,
    );

    // 同一个端点下可能挂多个账号：身份条必须说清这批桶属于哪个账号。
    expect(await screen.findByText(/运维主账号/)).toBeTruthy();
  });

  it('still lists buckets when the account lookup fails', async () => {
    const boundProvider = { ...OSS_PROVIDER, providerAccountId: 'account-42' };
    const request = vi.fn(async ({ operationId }: any) => {
      if (operationId === 'storageProviders.list') {
        return { items: [boundProvider], pageInfo: { hasMore: false } };
      }
      if (operationId === 'storageProviders.buckets.list') {
        return { items: [{ bucket: 'drive-hangzhou', configured: true }] };
      }
      // 账号名只是增强信息：它失败不能拖垮桶列表。
      throw new Error(`Unexpected operation ${operationId}`);
    });

    render(
      <StorageBucketsAdminPage
        adminStorageSdkClient={createFakeClient(request)}
        getSession={getSession}
      />,
    );

    expect((await screen.findAllByText('drive-hangzhou')).length).toBeGreaterThan(0);
  });

  it('switches the bucket list when another provider is selected in the rail', async () => {
    const request = vi.fn(async ({ operationId, pathParams }: any) => {
      if (operationId === 'storageProviders.list') {
        return { items: [OSS_PROVIDER, COS_PROVIDER], pageInfo: { hasMore: false } };
      }
      if (operationId === 'storageProviders.buckets.list') {
        return {
          items: [{ bucket: `bucket-${pathParams.providerId}`, configured: true }],
        };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const client = createFakeClient(request);

    render(<StorageBucketsAdminPage adminStorageSdkClient={client} getSession={getSession} />);

    expect(await screen.findByText(`bucket-${OSS_PROVIDER.id}`)).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: /COS Guangzhou/ }));

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.buckets.list'
            && call[0].pathParams.providerId === COS_PROVIDER.id,
        ),
      ).toBe(true);
    });
    expect(await screen.findByText(`bucket-${COS_PROVIDER.id}`)).toBeTruthy();
  });

  it('lists every bucket of the provider account, not just the first server window', async () => {
    // 账号里的桶多于一页时，第二页起必须也出现在表里：这一栏是"该配置下有哪些桶"的
    // 答案，数量徽标与搜索都建立在它是全集的前提上。
    const request = vi.fn(async ({ operationId, query }: any) => {
      if (operationId === 'storageProviders.list') {
        return { items: [OSS_PROVIDER], pageInfo: { mode: 'cursor', hasMore: false } };
      }
      if (operationId === 'storageProviders.buckets.list') {
        if (query?.cursor === 'opaque-bucket-next') {
          return {
            items: [{ bucket: 'zeta-media', configured: false }],
            pageInfo: { mode: 'cursor', hasMore: false },
          };
        }
        return {
          items: [{ bucket: 'drive-hangzhou', configured: true }],
          pageInfo: { mode: 'cursor', hasMore: true, nextCursor: 'opaque-bucket-next' },
        };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const client = createFakeClient(request);

    render(<StorageBucketsAdminPage adminStorageSdkClient={client} getSession={getSession} />);

    // 两页的桶都在同一个表里。
    expect(await screen.findByText('zeta-media')).toBeTruthy();
    expect((await screen.findAllByText('drive-hangzhou')).length).toBeGreaterThan(0);
    expect(
      request.mock.calls.filter((call) => call[0].operationId === 'storageProviders.buckets.list'),
    ).toHaveLength(2);
  });

  it('keeps the provider hand-off shortcuts optional', async () => {
    const request = vi.fn(async ({ operationId }: any) => {
      if (operationId === 'storageProviders.list') {
        return { items: [], pageInfo: { hasMore: false } };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const client = createFakeClient(request);

    render(<StorageBucketsAdminPage adminStorageSdkClient={client} getSession={getSession} />);

    // 宿主没有挂载服务商页面时不能渲染成死按钮。
    const emptyStateTitle = await screen.findByText('bucketsNoActiveProvidersTitle');
    const emptyState = emptyStateTitle.closest('div.rounded-lg') as HTMLElement;
    expect(emptyState).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'bucketsQuickCreate' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'bucketsQuickConfigure' })).toBeNull();
  });
});
