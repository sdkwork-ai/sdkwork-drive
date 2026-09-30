/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type {
  CreateStorageProviderAccountInput,
  StorageProviderAccountView,
} from '../src/types/storageProviderAdminTypes';
import { StorageProviderCredentialFields } from '../src/components/StorageProviderCredentialFields';
import { StorageProviderEditor } from '../src/components/StorageProviderEditor';

afterEach(() => cleanup());

const CREDENTIAL_FIELDS = {
  accessKeyLabel: 'Access Key ID',
  secretKeyLabel: 'Secret Access Key',
  accessKeyPlaceholder: 'AKIA...',
  secretKeyPlaceholder: '...',
  defaultEnvAccessKey: 'AWS_ACCESS_KEY_ID',
  defaultEnvSecretKey: 'AWS_SECRET_ACCESS_KEY',
};

const PLATFORM_ACCOUNT: StorageProviderAccountView = {
  id: 'acct-1',
  scopeType: 'platform',
  isDefault: true,
  vendorCode: 'aliyun',
  accountCode: 'aliyun-main-0001',
  displayName: 'Aliyun main account',
  accountType: 'long_term_key',
  environment: 'production',
  capabilityCodes: ['object_storage'],
  status: 'active',
  credentialConfigured: true,
  credentialCount: 1,
  version: 1,
};

function renderCredentialFields(overrides: Partial<Parameters<typeof StorageProviderCredentialFields>[0]> = {}) {
  const onProviderAccountIdChange = vi.fn();
  const props: Parameters<typeof StorageProviderCredentialFields>[0] = {
    credentialFields: CREDENTIAL_FIELDS,
    credentialRef: '',
    isEditing: false,
    error: undefined,
    onCredentialRefChange: vi.fn(),
    accountSourceEnabled: true,
    providerAccountId: '',
    onProviderAccountIdChange,
    providerAccounts: undefined,
    accountsLoading: false,
    accountsError: undefined,
    onReloadProviderAccounts: vi.fn(),
    onCreateProviderAccount: vi.fn(),
    // The editor always derives this from the provider kind (kind ↔ vendor
    // linkage); the tests default to an aliyun provider.
    allowedVendorCodes: ['aliyun'],
    defaultVendorCode: 'aliyun',
    ...overrides,
  };
  render(<StorageProviderCredentialFields {...props} />);
  return { onProviderAccountIdChange, props };
}

describe('StorageProviderCredentialFields', () => {
  it('opens the picker dialog from the select-account CTA and offers inline creation when the platform list is empty', async () => {
    renderCredentialFields({ providerAccounts: [] });

    fireEvent.click(screen.getByRole('button', { name: 'accountPickCta' }));

    expect(await screen.findByText('accountPickerTitle')).toBeDefined();
    expect(screen.getByText('accountPickerEmptyTitle')).toBeDefined();
    // The empty state carries the create action, so first-run operators never
    // leave the dialog to register their first account.
    fireEvent.click(screen.getByRole('button', { name: 'accountNew' }));
    expect(await screen.findByText('accountNewTitle')).toBeDefined();
  });

  it('registers the inline account in the platform scope, with the vendor pinned to the provider kind', async () => {
    const onCreateProviderAccount = vi.fn(
      async (input: CreateStorageProviderAccountInput) => ({
        ...PLATFORM_ACCOUNT,
        id: 'acct-new',
        displayName: input.displayName,
      }),
    );
    renderCredentialFields({
      providerAccounts: [],
      onCreateProviderAccount,
    });

    fireEvent.click(screen.getByRole('button', { name: 'accountPickCta' }));
    fireEvent.click(await screen.findByRole('button', { name: 'accountNew' }));

    fireEvent.change(await screen.findByLabelText('accountDisplayName'), {
      target: { value: 'Aliyun main account' },
    });
    fireEvent.change(screen.getByLabelText('Access Key ID'), { target: { value: 'LTAI-KEY' } });
    fireEvent.change(screen.getByLabelText('Secret Access Key'), { target: { value: 'secret' } });

    // Vendor linkage pins the create form: the vendor renders as a fixed value
    // (no select), so the created account always matches the provider kind.
    expect(screen.queryByRole('combobox')).toBeNull();
    expect(screen.getByText('aliyun')).toBeDefined();

    fireEvent.click(screen.getByRole('button', { name: 'accountCreate' }));

    await waitFor(() =>
      expect(onCreateProviderAccount).toHaveBeenCalledWith(
        expect.objectContaining({ scopeType: 'platform', vendorCode: 'aliyun' }),
      ),
    );
    await waitFor(() =>
      expect(onCreateProviderAccount.mock.results[0]?.value).resolves.toMatchObject({ id: 'acct-new' }),
    );
  });

  it('lists only accounts of the provider kind\u2019s vendor and binds the picked one', async () => {
    const otherVendor: StorageProviderAccountView = {
      ...PLATFORM_ACCOUNT,
      id: 'acct-aws',
      displayName: 'AWS org account',
      vendorCode: 'aws',
      accountCode: 'aws-main-0002',
      isDefault: false,
    };
    const { onProviderAccountIdChange } = renderCredentialFields({
      providerAccounts: [PLATFORM_ACCOUNT, otherVendor],
    });

    fireEvent.click(screen.getByRole('button', { name: 'accountPickCta' }));

    // Kind ↔ vendor linkage: an aliyun provider never sees the aws account.
    expect(await screen.findByText('Aliyun main account')).toBeDefined();
    expect(screen.queryByText('AWS org account')).toBeNull();

    fireEvent.click(screen.getByRole('button', { name: /Aliyun main account/ }));
    fireEvent.click(screen.getByRole('button', { name: 'accountPickerUse' }));

    await waitFor(() => expect(onProviderAccountIdChange).toHaveBeenCalledWith('acct-1'));
  });

  it('offers inline creation when the platform list holds no account for the linked vendor', async () => {
    renderCredentialFields({
      providerAccounts: [PLATFORM_ACCOUNT],
      allowedVendorCodes: ['tencent'],
      defaultVendorCode: 'tencent',
    });

    fireEvent.click(screen.getByRole('button', { name: 'accountPickCta' }));

    // The aliyun account is hidden by the linkage, so the tencent provider
    // reads the vendor-scoped empty state and is offered the create action.
    expect(await screen.findByText('accountPickerVendorEmptyTitle')).toBeDefined();
    expect(screen.queryByText('Aliyun main account')).toBeNull();
    expect(screen.getByRole('button', { name: 'accountNew' })).toBeDefined();
  });

  it('reads a bound account back as its own summary row and clears it on demand', () => {
    const { onProviderAccountIdChange } = renderCredentialFields({
      providerAccountId: 'acct-1',
      providerAccounts: [PLATFORM_ACCOUNT],
    });

    // The summary row is the only place the bound identity shows: display name,
    // badges, and the vendor/account code line.
    expect(screen.getByText('Aliyun main account')).toBeDefined();
    expect(screen.getByText('aliyun/aliyun-main-0001')).toBeDefined();

    fireEvent.click(screen.getByRole('button', { name: 'clear' }));

    expect(onProviderAccountIdChange).toHaveBeenCalledWith('');
  });
});

describe('StorageProviderEditor account source', () => {
  it('lists platform-scope accounts only, matching the picker and its inline create form', async () => {
    const onListProviderAccounts = vi.fn(async () => []);
    render(
      <StorageProviderEditor
        onClose={vi.fn()}
        onCreateProvider={vi.fn(async () => {
          throw new Error('not expected');
        })}
        onCreateProviderAccount={vi.fn(async () => {
          throw new Error('not expected');
        })}
        onListProviderAccounts={onListProviderAccounts}
        onRotateCredential={vi.fn(async () => {
          throw new Error('not expected');
        })}
        onUpdateProvider={vi.fn(async () => {
          throw new Error('not expected');
        })}
      />,
    );

    await waitFor(() =>
      expect(onListProviderAccounts).toHaveBeenCalledWith({ status: 'active', scopeType: 'platform' }),
    );
  });
});
