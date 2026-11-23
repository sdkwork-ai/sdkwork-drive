/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { StorageProviderEditor } from '../src/components/StorageProviderEditor';
import type {
  CreateStorageProviderInput,
  StorageProviderView,
  UpdateStorageProviderInput,
} from '../src/types/storageProviderAdminTypes';
import {
  buildProviderEndpointUrl,
  getAllProviderKindMeta,
} from '../src/utils/providerKindConfig';

/**
 * Region → endpoint cascade, on the screen an operator actually uses.
 *
 * The defect these tests pin: editing an existing provider moved the region
 * select while the endpoint kept the old host, so the dialog showed
 * `ap-beijing` next to a `guangzhou` URL and saving persisted the
 * disagreement. Every assertion below is about the two fields staying one
 * answer, for every vendor the catalog offers, in both create and edit mode.
 */

afterEach(() => cleanup());

/**
 * A row the server could have written: region-derived host, credential already
 * stored (masked), so the form is submittable without touching credentials.
 */
const BASE_PROVIDER: StorageProviderView = {
  id: 'provider-1',
  providerKind: 'tencent_cos',
  displayName: 'Tencent COS',
  endpointUrl: 'https://cos.ap-guangzhou.myqcloud.com',
  region: 'ap-guangzhou',
  bucket: 'sdkwork-drive-cos-1250000000',
  pathStyle: false,
  credentialRef: 'env:***',
  credentialConfigured: true,
  status: 'active',
  version: 1,
  strictTls: true,
};

function renderEditor(
  overrides: Partial<React.ComponentProps<typeof StorageProviderEditor>> = {},
) {
  const onCreateProvider = vi.fn(async (_input: CreateStorageProviderInput) => ({
    ...BASE_PROVIDER,
  }));
  const onUpdateProvider = vi.fn(async (_id: string, _input: UpdateStorageProviderInput) => ({
    ...BASE_PROVIDER,
  }));
  render(
    <StorageProviderEditor
      onClose={vi.fn()}
      onCreateProvider={onCreateProvider}
      onRotateCredential={vi.fn(async () => ({ ...BASE_PROVIDER }))}
      onUpdateProvider={onUpdateProvider}
      {...overrides}
    />,
  );
  return { onCreateProvider, onUpdateProvider };
}

function endpointInput(): HTMLInputElement {
  return screen.getByTestId('storage-provider-endpoint') as HTMLInputElement;
}

function regionControl(): HTMLSelectElement | HTMLInputElement {
  return screen.getByTestId('storage-provider-region') as HTMLSelectElement | HTMLInputElement;
}

describe('storage provider endpoint cascade', () => {
  it('rewrites the endpoint when the region changes on an edited provider', () => {
    renderEditor({ provider: BASE_PROVIDER });

    expect(regionControl().value).toBe('ap-guangzhou');
    expect(endpointInput().value).toBe('https://cos.ap-guangzhou.myqcloud.com');
    // Auto mode: the endpoint is derived, so it is not hand-editable.
    expect(endpointInput().readOnly).toBe(true);

    fireEvent.change(regionControl(), { target: { value: 'ap-beijing' } });

    expect(endpointInput().value).toBe('https://cos.ap-beijing.myqcloud.com');
  });

  it('cascades for every region-driven vendor in the catalog', () => {
    // A vendor added with a region list but no working derivation would be a
    // selector that silently does nothing; iterating the catalog is what turns
    // that into a red test instead of an operator's surprise.
    for (const meta of getAllProviderKindMeta()) {
      if (meta.features.isLocal || meta.regions.length < 2) {
        continue;
      }
      const kind = String(meta.value);
      const [first, second] = meta.regions;
      const firstEndpoint = buildProviderEndpointUrl(kind, first.value) as string;

      renderEditor({
        provider: {
          ...BASE_PROVIDER,
          providerKind: kind,
          region: first.value,
          endpointUrl: firstEndpoint,
        },
      });

      expect(endpointInput().value, `${kind} opened with the wrong endpoint`).toBe(firstEndpoint);

      fireEvent.change(regionControl(), { target: { value: second.value } });

      expect(
        endpointInput().value,
        `${kind} did not re-derive its endpoint for ${second.value}`,
      ).toBe(buildProviderEndpointUrl(kind, second.value));

      cleanup();
    }
  });

  it('flags a stored endpoint that disagrees with its region and aligns it on demand', () => {
    // The exact state in the bug report: region Beijing, host Guangzhou.
    renderEditor({
      provider: { ...BASE_PROVIDER, region: 'ap-beijing' },
    });

    // The stored value is preserved on open (no silent rewrite of stored data)...
    expect(endpointInput().value).toBe('https://cos.ap-guangzhou.myqcloud.com');
    // ...but the disagreement is stated, with the value the region derives.
    expect(screen.getByText('endpointMismatch')).toBeDefined();

    fireEvent.click(screen.getByRole('button', { name: 'endpointAlignRegion' }));

    expect(endpointInput().value).toBe('https://cos.ap-beijing.myqcloud.com');
    expect(screen.queryByText('endpointMismatch')).toBeNull();
    expect(screen.getByText('endpointAutoFilled')).toBeDefined();
  });

  it('keeps a manually typed endpoint when the region changes, and re-derives on auto', () => {
    renderEditor({ provider: BASE_PROVIDER });

    fireEvent.click(screen.getByRole('button', { name: 'endpointOverride' }));
    fireEvent.change(endpointInput(), {
      target: { value: 'https://cos.internal.example.com' },
    });
    expect(screen.getByText('endpointManualOverride')).toBeDefined();

    // An operator who brought their own gateway is not overruled by a region
    // click...
    fireEvent.change(regionControl(), { target: { value: 'ap-shanghai' } });
    expect(endpointInput().value).toBe('https://cos.internal.example.com');

    // ...but asking for the derived value gives it immediately, rather than
    // flipping a flag and leaving the stale host in the field.
    fireEvent.click(screen.getByRole('button', { name: 'endpointAuto' }));
    expect(endpointInput().value).toBe('https://cos.ap-shanghai.myqcloud.com');
    expect(endpointInput().readOnly).toBe(true);
  });

  it('cascades when a new provider switches to a region-derived vendor', () => {
    renderEditor();

    // The form opens on S3, whose host is regional too.
    expect(endpointInput().value).toBe('https://s3.us-east-1.amazonaws.com');

    // Pick Tencent COS in the vendor rail (the tile subtitle is the endpoint
    // hint without its scheme).
    const cosTile = screen.getByText('cos.ap-guangzhou.myqcloud.com').closest('button');
    expect(cosTile).not.toBeNull();
    fireEvent.click(cosTile as HTMLElement);

    expect(endpointInput().value).toBe('https://cos.ap-guangzhou.myqcloud.com');
    fireEvent.change(regionControl(), { target: { value: 'ap-shanghai' } });
    expect(endpointInput().value).toBe('https://cos.ap-shanghai.myqcloud.com');
  });

  it('leaves identifier-embedding hosts to the operator without an auto toggle', () => {
    const operatorEndpoint = 'https://ns1.compat.objectstorage.us-ashburn-1.oraclecloud.com';
    renderEditor({
      provider: {
        ...BASE_PROVIDER,
        providerKind: 'oracle_cloud_storage',
        region: 'us-ashburn-1',
        endpointUrl: operatorEndpoint,
      },
    });

    // No derivation exists for these vendors, so there is no "auto" claim to
    // make and no toggle that could produce a placeholder host.
    expect(screen.queryByRole('button', { name: 'endpointOverride' })).toBeNull();
    expect(endpointInput().readOnly).toBe(false);
    expect(screen.getByText('endpointOperatorHost')).toBeDefined();

    // The region is still editable — as a vendor region code, not a fixed list.
    const region = regionControl();
    expect(region.tagName).toBe('INPUT');
    fireEvent.change(region, { target: { value: 'eu-frankfurt-1' } });

    // Changing the region never rewrites a host the region cannot derive.
    expect(endpointInput().value).toBe(operatorEndpoint);
  });

  it('keeps an empty or retired stored region visible instead of showing the first option', () => {
    // A code the vendor's current list no longer carries: the selector must
    // still show the code the form will submit, not the list's first entry.
    renderEditor({
      provider: {
        ...BASE_PROVIDER,
        region: 'ap-retired-1',
        endpointUrl: 'https://cos.ap-retired-1.myqcloud.com',
      },
    });
    const retired = regionControl() as HTMLSelectElement;
    expect(retired.value).toBe('ap-retired-1');
    expect(Array.from(retired.options).map((option) => option.value)).toContain('ap-retired-1');
    cleanup();

    // No region at all: the selector says so, and the endpoint stays typeable
    // instead of being a read-only field with nothing to show.
    renderEditor({ provider: { ...BASE_PROVIDER, region: '', endpointUrl: '' } });
    const empty = regionControl() as HTMLSelectElement;
    expect(empty.value).toBe('');
    expect(endpointInput().readOnly).toBe(false);
  });

  it('refuses to save a row whose endpoint contradicts its region until it is aligned', async () => {
    const { onUpdateProvider } = renderEditor({
      provider: { ...BASE_PROVIDER, region: 'ap-beijing' },
    });
    const form = document.querySelector('form#storage-provider-editor-form') as HTMLFormElement;

    fireEvent.submit(form);

    // A provider whose region and host disagree sends every request to the
    // wrong endpoint, so the dialog does not persist one silently; both exits
    // (align, or declare the host manual) are one click away.
    expect(screen.getByText('endpointMismatchError')).toBeDefined();
    expect(onUpdateProvider).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'endpointAlignRegion' }));
    // The stale refusal goes away with the mismatch it described.
    expect(screen.queryByText('endpointMismatchError')).toBeNull();
    fireEvent.submit(form);

    await waitFor(() => expect(onUpdateProvider).toHaveBeenCalledTimes(1));
    const [, input] = onUpdateProvider.mock.calls[0];
    expect(input.region).toBe('ap-beijing');
    expect(input.endpointUrl).toBe('https://cos.ap-beijing.myqcloud.com');
  });

  it('lets a deliberately manual host be saved without a region match', async () => {
    const { onUpdateProvider } = renderEditor({
      provider: { ...BASE_PROVIDER, region: 'ap-beijing' },
    });

    // Declaring the field manual is the operator's statement that the host is
    // intentional — e.g. a private gateway in front of the vendor.
    fireEvent.click(screen.getByRole('button', { name: 'endpointOverride' }));
    fireEvent.submit(document.querySelector('form#storage-provider-editor-form') as HTMLFormElement);

    await waitFor(() => expect(onUpdateProvider).toHaveBeenCalledTimes(1));
    const [, input] = onUpdateProvider.mock.calls[0];
    expect(input.region).toBe('ap-beijing');
    expect(input.endpointUrl).toBe('https://cos.ap-guangzhou.myqcloud.com');
  });

  it('saves the cascaded endpoint together with the new region', async () => {
    const { onUpdateProvider } = renderEditor({ provider: BASE_PROVIDER });

    fireEvent.change(regionControl(), { target: { value: 'ap-beijing' } });
    expect(endpointInput().value).toBe('https://cos.ap-beijing.myqcloud.com');

    const form = document.querySelector('form#storage-provider-editor-form');
    expect(form).not.toBeNull();
    fireEvent.submit(form as HTMLFormElement);

    await waitFor(() => expect(onUpdateProvider).toHaveBeenCalledTimes(1));
    const [providerId, input] = onUpdateProvider.mock.calls[0];
    expect(providerId).toBe('provider-1');
    expect(input.region).toBe('ap-beijing');
    expect(input.endpointUrl).toBe('https://cos.ap-beijing.myqcloud.com');
  });
});
