/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  DriveAdminStorageSdkError,
  type DriveAdminStorageSdkClient,
} from 'sdkwork-drive-pc-admin-core';
import { StorageBindingsAdminPage } from '../src/pages/StorageBindingsAdminPage';

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

const ACTIVE_PROVIDER = {
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
};

const KNOWLEDGE_BASE_PREFIX = 'sdkwork-drive/v1/tenants/tenant-100/space-types/knowledge_base';

/** A tenant configuration pointing at the same provider. */
const TENANT_BINDING = {
  id: 'default:tenant:tenant-100',
  tenantId: 'tenant-100',
  providerId: ACTIVE_PROVIDER.id,
  bindingScope: 'tenant',
  purpose: 'primary',
  lifecycleStatus: 'active',
  version: 1,
  storageRootPrefix: 'sdkwork-drive/v1/tenants/tenant-100',
  storageProvider: ACTIVE_PROVIDER,
};

/**
 * The reads the page issues on mount.
 *
 * `storageProviderBindings.default.retrieve` answers 404 when the tenant has no
 * tenant configuration — the shape the real backend uses and the only one the
 * service projects to `undefined` rather than to a load failure. The two list
 * calls carry `binding_scope`, so the stub answers per step exactly like the
 * server does.
 */
function createRequest(
  bindings: unknown[],
  options: {
    tenantBinding?: unknown;
    onListObjects?: (input: any) => unknown;
    onBucketHead?: (input: any) => unknown;
  } = {},
) {
  const tenantBinding = options.tenantBinding === undefined ? TENANT_BINDING : options.tenantBinding;
  return vi.fn(async ({ operationId, query, pathParams }: any) => {
    if (operationId === 'storageProviders.list') {
      return { items: [ACTIVE_PROVIDER], pageInfo: { mode: 'cursor', hasMore: false } };
    }
    if (operationId === 'storageProviderBindings.list') {
      return {
        items: bindings.filter(
          (binding) =>
            !query?.binding_scope
            || (binding as { bindingScope?: string }).bindingScope === query.binding_scope,
        ),
        pageInfo: { mode: 'cursor', hasMore: false },
      };
    }
    if (operationId === 'storageProviderBindings.default.retrieve') {
      if (!tenantBinding) {
        throw new DriveAdminStorageSdkError({
          operationId: 'storageProviderBindings.default.retrieve',
          status: 404,
        });
      }
      return { item: tenantBinding };
    }
    if (operationId === 'storageProviders.bucket.retrieve') {
      return options.onBucketHead
        ? options.onBucketHead({ operationId, pathParams })
        : { providerId: pathParams.providerId, bucket: ACTIVE_PROVIDER.bucket, exists: true };
    }
    if (operationId === 'storageProviders.bucket.update') {
      return { providerId: pathParams.providerId, bucket: ACTIVE_PROVIDER.bucket, changed: true };
    }
    if (operationId === 'storageProviders.objects.list') {
      return options.onListObjects ? options.onListObjects({ operationId, query }) : { items: [] };
    }
    throw new Error(`Unexpected operation ${operationId}`);
  });
}

describe('StorageBindingsAdminPage storage targets', () => {
  it('presents the tenant storage provider configuration instead of a clearable default', async () => {
    const request = createRequest([]);
    render(
      <StorageBindingsAdminPage
        adminStorageSdkClient={createFakeClient(request)}
        getSession={getSession}
      />,
    );

    const card = await waitFor(() => screen.getByTestId('tenant-storage-configuration'));
    // The configuration is described by what it writes to: provider, bucket,
    // endpoint, prefix — not by an opaque binding id. The provider cell also
    // carries the kind chip, so the name is matched inside its text.
    expect(within(card).getByText(/OSS Hangzhou/)).toBeTruthy();
    expect(within(card).getByText('drive-hangzhou')).toBeTruthy();
    expect(within(card).getByText('https://oss-cn-hangzhou.aliyuncs.com')).toBeTruthy();
    expect(within(card).getByText('sdkwork-drive/v1/tenants/tenant-100')).toBeTruthy();
    // Changing it is offered; deleting it is not — a tenant with no storage
    // configuration cannot write at all.
    expect(within(card).getByRole('button', { name: 'bindingsTenantConfigChange' })).toBeTruthy();
    expect(within(card).queryByRole('button', { name: 'bindingsTenantDefaultClear' })).toBeNull();
  });

  it('reports the resolution chain and lists one section per step', async () => {
    const request = createRequest([
      {
        id: 'binding-kb',
        tenantId: 'tenant-100',
        providerId: ACTIVE_PROVIDER.id,
        bindingScope: 'space_type',
        purpose: 'knowledge_base',
        lifecycleStatus: 'active',
        version: 1,
        storageRootPrefix: KNOWLEDGE_BASE_PREFIX,
        storageProvider: ACTIVE_PROVIDER,
      },
      {
        id: 'default:space:tenant-100:space-team-a',
        tenantId: 'tenant-100',
        spaceId: 'space-team-a',
        providerId: ACTIVE_PROVIDER.id,
        bindingScope: 'space',
        purpose: 'primary',
        lifecycleStatus: 'active',
        version: 1,
        storageRootPrefix: 'sdkwork-drive/v1/tenants/tenant-100/spaces/space-team-a',
        storageProvider: ACTIVE_PROVIDER,
      },
    ]);
    render(
      <StorageBindingsAdminPage
        adminStorageSdkClient={createFakeClient(request)}
        getSession={getSession}
      />,
    );

    expect(await screen.findByText('bindingsResolutionHint')).toBeTruthy();

    // Each step is read through its own `binding_scope` query: the unfiltered
    // list is one page of every step, so a shared read cannot tell "unbound"
    // from "on another page".
    const scopes = request.mock.calls
      .filter((call) => call[0].operationId === 'storageProviderBindings.list')
      .map((call) => call[0].query.binding_scope);
    expect(scopes).toContain('space_type');
    expect(scopes).toContain('space');

    // The space-level section is its own inventory.
    const spaceSection = screen.getByTestId('space-scope-bindings');
    expect(within(spaceSection).getByText('space-team-a')).toBeTruthy();
  });

  it('shows the bucket each space type writes into, and flags a missing bucket', async () => {
    const request = createRequest(
      [
        {
          id: 'binding-kb',
          tenantId: 'tenant-100',
          providerId: ACTIVE_PROVIDER.id,
          bindingScope: 'space_type',
          purpose: 'knowledge_base',
          lifecycleStatus: 'active',
          version: 1,
          storageRootPrefix: KNOWLEDGE_BASE_PREFIX,
          storageProvider: ACTIVE_PROVIDER,
        },
      ],
      { onBucketHead: () => ({ providerId: ACTIVE_PROVIDER.id, bucket: 'drive-hangzhou', exists: false }) },
    );
    render(
      <StorageBindingsAdminPage
        adminStorageSdkClient={createFakeClient(request)}
        getSession={getSession}
      />,
    );

    // The bound row names its bucket outright.
    await waitFor(() => expect(screen.getAllByText('drive-hangzhou').length).toBeGreaterThan(0));

    fireEvent.click(screen.getAllByRole('button', { name: 'bindingsChange' })[0]);

    // Binding to a bucket that was never created is the failure this control
    // exists for, and the way out is offered right there.
    await screen.findByText('bindingsBucketMissing');
    const readiness = screen.getByTestId('binding-bucket-readiness');
    expect(within(readiness).getByRole('button', { name: 'bindingsBucketInitialize' })).toBeTruthy();

    fireEvent.click(within(readiness).getByRole('button', { name: 'bindingsBucketInitialize' }));
    await waitFor(() =>
      expect(
        request.mock.calls.some((call) => call[0].operationId === 'storageProviders.bucket.update'),
      ).toBe(true),
    );
  });

  it('binds the website space type to its own bucket', async () => {
    const request = createRequest([]);
    render(
      <StorageBindingsAdminPage
        adminStorageSdkClient={createFakeClient(request)}
        getSession={getSession}
      />,
    );

    // `website` is a first-class space type on the platform; without its row the
    // published-site content could never be given a bucket of its own.
    const websiteRow = (await screen.findByText('spaceTypeWebsiteLabel')).closest('tr');
    expect(websiteRow).not.toBeNull();
    fireEvent.click(within(websiteRow as HTMLElement).getByRole('button', { name: 'bindingsAssign' }));

    // The editor needs the provider option set, which loads alongside the
    // bindings; saving before it arrives is impossible (the action is disabled)
    // rather than silently wrong.
    const saveButton = await screen.findByRole('button', { name: 'save' });
    await waitFor(() => expect((saveButton as HTMLButtonElement).disabled).toBe(false));
    fireEvent.click(saveButton);

    await waitFor(() =>
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviderBindings.default.update'
            && call[0].body?.spaceType === 'website'
            && call[0].body?.providerId === ACTIVE_PROVIDER.id,
        ),
      ).toBe(true),
    );
  });

  it('saves a space-level binding for a named space and its default prefix', async () => {
    const request = createRequest([]);
    render(
      <StorageBindingsAdminPage
        adminStorageSdkClient={createFakeClient(request)}
        getSession={getSession}
      />,
    );

    fireEvent.click(await screen.findByRole('button', { name: 'bindingsSpaceScopeAdd' }));
    fireEvent.change(screen.getByPlaceholderText('bindingsSpaceIdPlaceholder'), {
      target: { value: 'space-team-b' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'save' }));

    await waitFor(() =>
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviderBindings.default.update'
            && call[0].body?.spaceId === 'space-team-b'
            && call[0].body?.providerId === ACTIVE_PROVIDER.id,
        ),
      ).toBe(true),
    );
    // The prefix is left to the backend default unless the operator opts into a
    // custom one, so the request must not invent a value.
    const saveCall = request.mock.calls.find(
      (call) => call[0].operationId === 'storageProviderBindings.default.update',
    );
    expect(saveCall?.[0].body?.storageRootPrefix).toBeUndefined();
  });

  it('warns when the tenant has no storage provider configuration at all', async () => {
    const request = createRequest([], { tenantBinding: null });
    render(
      <StorageBindingsAdminPage
        adminStorageSdkClient={createFakeClient(request)}
        getSession={getSession}
      />,
    );

    const card = await waitFor(() => screen.getByTestId('tenant-storage-configuration'));
    expect(within(card).getByText('bindingsTenantConfigUnset')).toBeTruthy();
    expect(within(card).getByText('bindingsTenantConfigUnsetDesc')).toBeTruthy();
    expect(within(card).getByRole('button', { name: 'bindingsTenantConfigSet' })).toBeTruthy();
  });
});

describe('StorageBindingsAdminPage file browsing', () => {
  it('opens the objects behind a bound space type in a modal file browser', async () => {
    const request = createRequest(
      [
        {
          id: 'binding-kb',
          tenantId: 'tenant-100',
          providerId: ACTIVE_PROVIDER.id,
          bindingScope: 'space_type',
          purpose: 'knowledge_base',
          lifecycleStatus: 'active',
          version: 1,
          storageRootPrefix: KNOWLEDGE_BASE_PREFIX,
          storageProvider: ACTIVE_PROVIDER,
        },
      ],
      {
        onListObjects: () => ({
          items: [
            {
              objectKey: `${KNOWLEDGE_BASE_PREFIX}/readme.md`,
              objectKind: 'object',
              contentLength: 2048,
              lastModifiedEpochMs: Date.UTC(2026, 0, 2),
            },
            { objectKey: `${KNOWLEDGE_BASE_PREFIX}/archive/`, objectKind: 'prefix' },
          ],
        }),
      },
    );
    const client = createFakeClient(request);

    render(<StorageBindingsAdminPage adminStorageSdkClient={client} getSession={getSession} />);

    // Exactly the bound row carries the action: an unbound space type has no
    // provider to browse, so it must not offer one.
    const browse = await screen.findByRole('button', { name: 'bindingsBrowseFiles' });

    // The object plane is opened on demand, not by listing every binding.
    expect(
      request.mock.calls.some((call) => call[0].operationId === 'storageProviders.objects.list'),
    ).toBe(false);

    fireEvent.click(browse);

    // The shared browser mounts inside the dialog and loads the binding's own
    // root prefix rather than the bucket root.
    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.list'
            && call[0].pathParams.providerId === ACTIVE_PROVIDER.id
            && call[0].query.prefix === KNOWLEDGE_BASE_PREFIX,
        ),
      ).toBe(true);
    });
    expect(await screen.findByText('bindingsBrowserTitle')).toBeTruthy();
    expect(await screen.findByText('readme.md')).toBeTruthy();
    expect(await screen.findByText('archive/')).toBeTruthy();

    // Closing the dialog unmounts the browser with it.
    fireEvent.click(screen.getByRole('button', { name: 'bindingsBrowserClose' }));
    await waitFor(() => {
      expect(screen.queryByText('readme.md')).toBeNull();
    });
  });

  it('falls back to the backend default prefix when the binding stores none', async () => {
    const request = createRequest([
      {
        id: 'binding-personal',
        tenantId: 'tenant-100',
        providerId: ACTIVE_PROVIDER.id,
        bindingScope: 'space_type',
        purpose: 'personal',
        lifecycleStatus: 'active',
        version: 1,
        storageProvider: ACTIVE_PROVIDER,
      },
    ]);
    const client = createFakeClient(request);

    render(<StorageBindingsAdminPage adminStorageSdkClient={client} getSession={getSession} />);

    fireEvent.click(await screen.findByRole('button', { name: 'bindingsBrowseFiles' }));

    // `default_storage_root_prefix` on the backend derives the same path for a
    // space-type binding, so browsing starts on the sub-tree uploads land in.
    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.list'
            && call[0].query.prefix === 'sdkwork-drive/v1/tenants/tenant-100/space-types/personal',
        ),
      ).toBe(true);
    });
  });
});
