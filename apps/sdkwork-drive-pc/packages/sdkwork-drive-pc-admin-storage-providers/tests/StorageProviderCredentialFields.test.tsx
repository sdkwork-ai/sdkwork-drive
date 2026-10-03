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
    // No vendor linkage: the list is the whole platform slice, so an empty
    // result really does mean "the platform has no account yet".
    renderCredentialFields({ providerAccounts: [], allowedVendorCodes: undefined });

    fireEvent.click(screen.getByRole('button', { name: 'accountPickCta' }));

    expect(await screen.findByText('accountPickerTitle')).toBeDefined();
    expect(screen.getByText('accountPickerEmptyTitle')).toBeDefined();
    // The empty state carries the create action, so first-run operators never
    // leave the dialog to register their first account.
    fireEvent.click(screen.getByRole('button', { name: 'accountNew' }));
    expect(await screen.findByText('accountNewTitle')).toBeDefined();
  });

  it('reads an empty vendor-linked list as "no account for this vendor"', async () => {
    // The editor always sends the provider kind's vendor to the server as the
    // list's window filter, so with a linkage an empty page is this vendor's
    // answer — telling the operator the platform has no accounts at all would
    // be false while 24 other vendors' accounts sit in the account center.
    renderCredentialFields({ providerAccounts: [] });

    fireEvent.click(screen.getByRole('button', { name: 'accountPickCta' }));

    expect(await screen.findByText('accountPickerVendorEmptyTitle')).toBeDefined();
    expect(screen.queryByText('accountPickerEmptyTitle')).toBeNull();
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
  // The editor opens on `s3_compatible`, whose vendor is `aws`, and the picker
  // is vendor-linked: these fixtures have to belong to that vendor or the guard
  // hides them before any of the assertions below can mean anything.
  const AWS_ACCOUNT: StorageProviderAccountView = {
    ...PLATFORM_ACCOUNT,
    id: 'acct-aws-1',
    vendorCode: 'aws',
    accountCode: 'aws-main-0001',
    displayName: 'AWS main account',
  };

  function renderEditor(
    onCreateProviderAccount: (input: CreateStorageProviderAccountInput) => Promise<StorageProviderAccountView>,
    onListProviderAccounts: (input?: unknown) => Promise<{
      items: StorageProviderAccountView[];
      hasMore: boolean;
      nextPageToken?: string;
    }>,
  ) {
    render(
      <StorageProviderEditor
        onClose={vi.fn()}
        onCreateProvider={vi.fn(async () => {
          throw new Error('not expected');
        })}
        onCreateProviderAccount={onCreateProviderAccount}
        // The prop is typed against the account-list input; the tests only ever
        // assert the page request shape they hand back to the editor.
        onListProviderAccounts={onListProviderAccounts as never}
        onRotateCredential={vi.fn(async () => {
          throw new Error('not expected');
        })}
        onUpdateProvider={vi.fn(async () => {
          throw new Error('not expected');
        })}
      />,
    );
  }

  it('lists the vendor-scoped platform slice with an explicit page window', async () => {
    const onListProviderAccounts = vi.fn(async () => ({
      items: [] as StorageProviderAccountView[],
      hasMore: false,
    }));
    renderEditor(
      vi.fn(async () => {
        throw new Error('not expected');
      }),
      onListProviderAccounts,
    );

    // The kind's vendor travels to the server so the window is selected from
    // that vendor's accounts: filtering the returned page instead answers
    // "this vendor has no accounts" whenever its rows sort past page 1 — which
    // is how a freshly registered account disappeared from the picker.
    await waitFor(() =>
      expect(onListProviderAccounts).toHaveBeenCalledWith({
        status: 'active',
        scopeType: 'platform',
        vendorCode: 'aws',
        pageSize: 20,
      }),
    );
  });

  it('keeps an account registered in the picker visible in the account list afterwards', async () => {
    const created: StorageProviderAccountView = {
      ...AWS_ACCOUNT,
      id: 'acct-aws-new',
      accountCode: 'aws-main-brandnew',
      displayName: 'AWS brand new account',
    };
    const onListProviderAccounts = vi.fn(async () => ({
      // The reloaded page does not carry the created row: the vendor already
      // fills a page, so the new account sorts past the window.
      items: [AWS_ACCOUNT],
      hasMore: false,
    }));
    renderEditor(vi.fn(async () => created), onListProviderAccounts);

    fireEvent.click(await screen.findByRole('button', { name: 'accountPickCta' }));
    fireEvent.click(await screen.findByRole('button', { name: 'accountNew' }));
    fireEvent.change(await screen.findByLabelText('accountDisplayName'), {
      target: { value: 'AWS brand new account' },
    });
    fireEvent.change(screen.getByLabelText('Access Key ID'), { target: { value: 'AKIA-KEY' } });
    fireEvent.change(screen.getByLabelText('Secret Access Key'), { target: { value: 'secret' } });
    fireEvent.click(screen.getByRole('button', { name: 'accountCreate' }));

    // The created account becomes the bound one immediately: the credential
    // section resolves its row from the list it was created into.
    expect(await screen.findByText('AWS brand new account')).toBeDefined();
    expect(screen.getByText('aws/aws-main-brandnew')).toBeDefined();

    // And re-opening the picker finds it as a pickable row rather than showing
    // the vendor-empty state that invites registering the same account again.
    fireEvent.click(await screen.findByRole('button', { name: 'accountChange' }));
    expect(
      await screen.findByRole('button', { name: /AWS brand new account/ }),
    ).toBeDefined();
    expect(screen.queryByText('accountPickerVendorEmptyTitle')).toBeNull();
  });

  it('renders the server continuation and appends the next account page', async () => {
    const secondPageAccount: StorageProviderAccountView = {
      ...AWS_ACCOUNT,
      id: 'acct-aws-page-2',
      accountCode: 'aws-main-page2',
      displayName: 'AWS second page account',
      isDefault: false,
    };
    const onListProviderAccounts = vi
      .fn()
      .mockResolvedValueOnce({ items: [AWS_ACCOUNT], hasMore: true, nextPageToken: 'cursor-1' })
      .mockResolvedValueOnce({ items: [secondPageAccount], hasMore: false });
    renderEditor(
      vi.fn(async () => {
        throw new Error('not expected');
      }),
      onListProviderAccounts,
    );

    fireEvent.click(await screen.findByRole('button', { name: 'accountPickCta' }));
    expect(await screen.findByText('AWS main account')).toBeDefined();

    // The continuation is the server's own `hasMore`/`nextCursor`, rendered
    // instead of quietly presenting one page as the whole vendor's accounts.
    fireEvent.click(screen.getByRole('button', { name: 'accountLoadMore' }));

    await waitFor(() =>
      expect(onListProviderAccounts).toHaveBeenLastCalledWith({
        status: 'active',
        scopeType: 'platform',
        vendorCode: 'aws',
        pageSize: 20,
        pageToken: 'cursor-1',
      }),
    );
    expect(await screen.findByText('AWS second page account')).toBeDefined();
    // Both pages stay listed once the continuation is read, and the exhausted
    // list stops offering more.
    expect(screen.getByText('AWS main account')).toBeDefined();
    expect(screen.queryByRole('button', { name: 'accountLoadMore' })).toBeNull();
  });
});
