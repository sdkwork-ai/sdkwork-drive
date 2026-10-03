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

interface ListQuery {
  provider_kind?: string;
  page_size?: number;
  cursor?: string;
}

type RequestMock = ReturnType<typeof vi.fn>;

/**
 * The surface issues two different `storageProviders.list` reads.
 *
 * The table page is one cursor page (`page_size` 20) filtered by the provider
 * kind; the switches behind the drawer and the editor read the option set with
 * the maximum page size (200). Tests therefore key on the request shape, never
 * on call order — which is what these tests used to do, and what made them
 * sensitive to a second, legitimate read.
 */
function listCalls(request: RequestMock, pageSize: number) {
  return request.mock.calls.filter(
    ([arg]) =>
      (arg as { operationId?: string }).operationId === 'storageProviders.list'
      && (arg as { query?: ListQuery }).query?.page_size === pageSize,
  );
}

const ARCHIVE_PROVIDER = {
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
};

/**
 * Sorts after the first page of the unfiltered list (`id ASC`, 20 per page),
 * which is exactly why the kind switch has to be answered by the server.
 */
const COS_PROVIDER = {
  id: 'provider-z-tencent-cos',
  providerKind: 'tencent_cos',
  name: 'Tencent COS',
  endpointUrl: 'https://cos.ap-guangzhou.myqcloud.com',
  region: 'ap-guangzhou',
  bucket: 'drive-cos',
  pathStyle: false,
  credentialConfigured: false,
  status: 'active',
  version: 1,
  strictTls: true,
};

/** The query of the last table-page read. */
function lastPageQuery(request: RequestMock): ListQuery {
  const call = listCalls(request, 20).at(-1);
  if (!call) {
    throw new Error('Expected a provider table page request.');
  }
  return (call[0] as { query: ListQuery }).query;
}

describe('StorageProvidersAdminPage', () => {
  it('uses the server returned opaque cursor when loading the next provider page', async () => {
    const request = vi.fn(async (arg: { operationId: string; query?: ListQuery }) => {
      if (arg.operationId !== 'storageProviders.list') {
        throw new Error(`Unexpected operation ${arg.operationId}`);
      }
      if (arg.query?.page_size === 200) {
        // The switch's option walk: same single provider, nothing after it.
        return { items: [LOADED_PROVIDER], pageInfo: { mode: 'cursor', hasMore: false } };
      }
      if (arg.query?.cursor === 'opaque-provider-next') {
        return { items: [ARCHIVE_PROVIDER], pageInfo: { mode: 'cursor', hasMore: false } };
      }
      return {
        items: [LOADED_PROVIDER],
        pageInfo: { mode: 'cursor', hasMore: true, nextCursor: 'opaque-provider-next' },
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

    await screen.findByText('Archive Provider');
    expect(listCalls(request, 20).at(-1)?.[0]).toMatchObject({
      operationId: 'storageProviders.list',
      query: {
        page_size: 20,
        cursor: 'opaque-provider-next',
      },
    });
  });

  it('re-reads the table at the chosen page size and restarts the cursor chain', async () => {
    const request = vi.fn(async (arg: { operationId: string; query?: ListQuery }) => {
      if (arg.operationId !== 'storageProviders.list') {
        throw new Error(`Unexpected operation ${arg.operationId}`);
      }
      if (arg.query?.page_size === 200) {
        // The switch's option walk; unrelated to the table's window size.
        return { items: [LOADED_PROVIDER], pageInfo: { mode: 'cursor', hasMore: false } };
      }
      if (arg.query?.page_size === 50) {
        return {
          items: [LOADED_PROVIDER, ARCHIVE_PROVIDER],
          pageInfo: { mode: 'cursor', hasMore: false },
        };
      }
      if (arg.query?.cursor === 'opaque-provider-next') {
        return { items: [ARCHIVE_PROVIDER], pageInfo: { mode: 'cursor', hasMore: false } };
      }
      return {
        items: [LOADED_PROVIDER],
        pageInfo: { mode: 'cursor', hasMore: true, nextCursor: 'opaque-provider-next' },
      };
    });

    render(
      <StorageProvidersAdminPage
        adminStorageSdkClient={fakeClient(request)}
        getSession={OPERATOR_SESSION}
      />,
    );

    await screen.findByText('Primary Provider');
    // Walk to page 2 first: the window size then changes while a cursor minted
    // for the 20-row window is live, which is the state the reset has to handle.
    fireEvent.click(screen.getByRole('button', { name: 'nextPage' }));
    await screen.findByText('Archive Provider');
    expect(lastPageQuery(request).cursor).toBe('opaque-provider-next');

    fireEvent.change(screen.getByLabelText('pageSizeLabel'), { target: { value: '50' } });

    await waitFor(() => expect(listCalls(request, 50)).toHaveLength(1));
    // The choice is a server window, not a client-side slice: it travels as
    // `page_size` on a fresh request.
    expect(listCalls(request, 50)[0][0]).toMatchObject({
      operationId: 'storageProviders.list',
      query: { page_size: 50 },
    });
    // And the 20-row cursor is not replayed into it — a keyset cursor describes
    // the window it was minted for, so the chain restarts at page 1.
    expect((listCalls(request, 50)[0][0] as { query: ListQuery }).query.cursor).toBeUndefined();
    // Page 1, not page 2 with its cursor dropped: "previous" is the control that
    // says so, and it is only disabled on the first page.
    expect((screen.getByRole('button', { name: 'previousPage' }) as HTMLButtonElement).disabled).toBe(true);
  });

  it('sends the provider-kind switch to the server instead of filtering one page', async () => {
    const request = vi.fn(async (arg: { operationId: string; query?: ListQuery }) => {
      if (arg.operationId !== 'storageProviders.list') {
        throw new Error(`Unexpected operation ${arg.operationId}`);
      }
      // `page_size` 200 is the option walk; 20 is the table page. Only the
      // filtered page answers with the COS row, which is what the server does
      // once the kind is a query parameter.
      const items = arg.query?.provider_kind === 'tencent_cos' ? [COS_PROVIDER] : [LOADED_PROVIDER];
      return { items, pageInfo: { mode: 'cursor', hasMore: false } };
    });

    render(
      <StorageProvidersAdminPage
        adminStorageSdkClient={fakeClient(request)}
        getSession={OPERATOR_SESSION}
      />,
    );

    await screen.findByText('Primary Provider');

    fireEvent.change(screen.getByLabelText('filterByProviderKind'), {
      target: { value: 'tencent_cos' },
    });

    // The row that the old client-side filter could never see: it lives past
    // the first page of the unfiltered list.
    await screen.findByText('Tencent COS');
    expect(listCalls(request, 20).at(-1)?.[0]).toMatchObject({
      operationId: 'storageProviders.list',
      query: {
        provider_kind: 'tencent_cos',
        page_size: 20,
      },
    });
    // The cursor chain restarts: a cursor from the previous result set must
    // never be replayed into a different filter.
    expect(lastPageQuery(request).cursor).toBeUndefined();
    expect(screen.queryByText('Primary Provider')).toBeNull();
  });

  it('reports a provider kind with no configuration as a filter result, not as an empty plane', async () => {
    const request = vi.fn(async (arg: { operationId: string; query?: ListQuery }) => {
      if (arg.operationId !== 'storageProviders.list') {
        throw new Error(`Unexpected operation ${arg.operationId}`);
      }
      const filtered = arg.query?.provider_kind === 'volcengine_tos';
      const items = arg.query?.page_size === 200
        ? [LOADED_PROVIDER]
        : filtered
          ? []
          : [LOADED_PROVIDER];
      return { items, pageInfo: { mode: 'cursor', hasMore: false } };
    });

    render(
      <StorageProvidersAdminPage
        adminStorageSdkClient={fakeClient(request)}
        getSession={OPERATOR_SESSION}
      />,
    );

    await screen.findByText('Primary Provider');
    fireEvent.change(screen.getByLabelText('filterByProviderKind'), {
      target: { value: 'volcengine_tos' },
    });

    // "Create your first provider" here would be a lie — the plane has one.
    await screen.findByText('filteredEmptyTitle');
    expect(screen.queryByText('emptyTitle')).toBeNull();

    fireEvent.click(screen.getByRole('button', { name: 'filteredEmptyClear' }));
    await screen.findByText('Primary Provider');
    expect(lastPageQuery(request).provider_kind).toBeUndefined();
    expect(lastPageQuery(request).page_size).toBe(20);
  });

  it('offers the option set past the table page to the binding switch', async () => {
    const request = vi.fn(async (arg: { operationId: string; query?: ListQuery }) => {
      if (arg.operationId !== 'storageProviders.list') {
        throw new Error(`Unexpected operation ${arg.operationId}`);
      }
      if (arg.query?.page_size === 200) {
        // The switch's option page: it carries the provider the table's first
        // page does not show, and reports no continuation.
        return { items: [LOADED_PROVIDER, COS_PROVIDER], pageInfo: { mode: 'cursor', hasMore: false } };
      }
      return { items: [LOADED_PROVIDER], pageInfo: { mode: 'cursor', hasMore: false } };
    });

    render(
      <StorageProvidersAdminPage
        adminStorageSdkClient={fakeClient(request)}
        getSession={OPERATOR_SESSION}
      />,
    );

    await screen.findByText('Primary Provider');
    await waitFor(() =>
      expect(listCalls(request, 200).at(-1)?.[0]).toMatchObject({
        query: { page_size: 200 },
      }),
    );
    // One option read, not a cursor walk.
    expect(listCalls(request, 200)).toHaveLength(1);

    // The row button carries the display name and the id, so match on the name.
    fireEvent.click(screen.getByRole('button', { name: /Primary Provider/ }));
    // The drawer's default-binding switch lists the provider that only the
    // option page ever loaded.
    await waitFor(() =>
      expect(
        Array.from(document.querySelectorAll('option')).some(
          (option) => option.textContent === 'Tencent COS',
        ),
      ).toBe(true),
    );
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
    // One table page plus one option read, both `storageProviders.list`.
    expect(listCalls(request, 20)).toHaveLength(1);
    expect(listCalls(request, 200)).toHaveLength(1);

    fireEvent.click(screen.getByRole('button', { name: 'initializeAccounts' }));

    await waitFor(() =>
      expect(operationIds).toContain('storageProviderAccountDefaults.create'),
    );
    // 初始化必须带回一次列表重拉：新铺的服务商配置只有重拉才会出现在表格里\r
    // 否则运维点了按钮却看不到任何变化。开关用的选项集合同样要重拉——这次运行新建的
    // 服务商配置超出了表格第一页，只有选项集合重拉才会出现在选择控件里\r
    await waitFor(() => expect(listCalls(request, 20).length).toBeGreaterThanOrEqual(2));
    await waitFor(() => expect(listCalls(request, 200).length).toBeGreaterThanOrEqual(2));
    // 测试宿主没有挂 LanguageProvider，`t()` 回退到原始 key —— 这里断言的是
    // "走到哪个通知键"，即成功分支而非失败分支\r
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

