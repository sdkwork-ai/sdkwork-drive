import React, { useEffect, useMemo, useState } from 'react';
import { Cloud, KeyRound, Plus, RefreshCw, Search } from 'lucide-react';
import {
  Modal,
  ModalBody,
  ModalContent,
  ModalDescription,
  ModalFooter,
  ModalHeader,
  ModalTitle,
} from '@sdkwork/ui-pc-react';
import type { ProviderCredentialFieldMeta } from '../utils/providerKindConfig';
import type {
  CreateStorageProviderAccountInput,
  StorageProviderAccountScope,
  StorageProviderAccountView,
} from '../types/storageProviderAdminTypes';
import { DriveAdminStorageSdkError } from 'sdkwork-drive-pc-admin-core';
import { INPUT_CLASS, PRIMARY_BUTTON_CLASS, SECONDARY_BUTTON_CLASS, SELECT_CLASS, CHECKBOX_CLASS } from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';
import { StorageProviderAccountBadges } from './StorageProviderAccountBadges';

/**
 * Vendor codes offered when registering a new account-center account.
 *
 * The account center accepts any `^[a-z][a-z0-9_]{1,31}$` vendor code, so this
 * is a convenience list rather than a restriction: it covers every catalogued
 * storage vendor plus a few adjacent clouds an operator may want to register
 * keys against. `providerVendorCodeForKind` supplies the pre-selection.
 */
const ACCOUNT_VENDOR_CODES = [
  'aws',
  'aliyun',
  'tencent',
  'huawei',
  'volcengine',
  'baidu',
  'kingsoft',
  'qiniu',
  'china_mobile',
  'china_telecom',
  'china_unicom',
  'google',
  'minio',
  'cloudflare',
  'backblaze',
  'wasabi',
  'digitalocean',
  'linode',
  'vultr',
  'scaleway',
  'oracle',
  'ibm',
  'custom',
] as const;

/**
 * Scope every account registered through the admin console gets.
 *
 * The storage admin plane publishes platform-wide credentials: one account,
 * every tenant reuses it, and rotation happens once in the account center. A
 * tenant operator that reaches this dialog lacks the platform-operator
 * identity, so the create call fails closed with a 403 the dialog renders as
 * `accountPlatformRequired` instead of offering a scope choice that would
 * produce the same failure one step later.
 */
const ACCOUNT_SCOPE: StorageProviderAccountScope = 'platform';

interface StorageProviderAccountPickerDialogProps {
  open: boolean;
  /** Platform-wide accounts loaded from the account center. */
  accounts: StorageProviderAccountView[] | undefined;
  loading: boolean;
  error?: string;
  /**
   * Whether the account center holds accounts past the loaded pages.
   *
   * The list is paginated, so a page that is full is not the same answer as a
   * complete set: without the server's own `hasMore` an operator cannot tell a
   * vendor whose accounts start on a later page from a vendor that has none,
   * and the empty state then reads as "register one" for a vendor that already
   * has an account.
   */
  hasMoreAccounts?: boolean;
  /** True while the next page is in flight. */
  loadingMoreAccounts?: boolean;
  /** Read the next page from the account center (server-issued cursor). */
  onLoadMoreAccounts?: () => void;
  /**
   * Vendor codes the provider's kind can bind, derived from the kind catalog.
   * The picker lists only these accounts and the create form is pinned to
   * them, so an Aliyun provider never sees a Tencent account. Empty/undefined
   * means unrestricted (no linkage known).
   */
  allowedVendorCodes?: readonly string[];
  /** Currently referenced account id (empty string = none). */
  selectedAccountId: string;
  onSelectedAccountChange: (accountId: string) => void;
  onClose: () => void;
  onReloadAccounts: () => void;
  onCreateProviderAccount: (
    input: CreateStorageProviderAccountInput,
  ) => Promise<StorageProviderAccountView>;
  /** Vendor vocabulary for the two key-pair labels in the create form. */
  credentialFields: ProviderCredentialFieldMeta;
  /** Vendor pre-selection for the create form, derived from the provider kind. */
  defaultVendorCode: string;
}

/**
 * The "select account" dialog behind the credential section.
 *
 * One dialog owns both answers to "which credentials?": pick an existing
 * platform-wide account from the list, or — when the list is empty, which is
 * the first-run state — register the account inline and bind it in the same
 * breath. The create form lives here rather than on the provider form so the
 * credential section itself stays a single button plus one summary row.
 */
export function StorageProviderAccountPickerDialog({
  open,
  accounts,
  loading,
  error,
  hasMoreAccounts,
  loadingMoreAccounts,
  onLoadMoreAccounts,
  allowedVendorCodes,
  selectedAccountId,
  onSelectedAccountChange,
  onClose,
  onReloadAccounts,
  onCreateProviderAccount,
  credentialFields,
  defaultVendorCode,
}: StorageProviderAccountPickerDialogProps) {
  const { t } = useTranslation();
  // `list` is the landing view; `create` is the inline registration form. The
  // dialog re-opens on `list` so a stale draft never survives a cancel.
  const [mode, setMode] = useState<'list' | 'create'>('list');
  const [draftId, setDraftId] = useState(selectedAccountId);
  const [search, setSearch] = useState('');

  const [displayName, setDisplayName] = useState('');
  const [vendorCode, setVendorCode] = useState(defaultVendorCode);
  const [accountCode, setAccountCode] = useState(() => generateAccountCode(defaultVendorCode));
  const [accessKeyId, setAccessKeyId] = useState('');
  const [secretAccessKey, setSecretAccessKey] = useState('');
  const [sessionToken, setSessionToken] = useState('');
  const [isDefault, setIsDefault] = useState(false);
  const [creating, setCreating] = useState(false);
  const [createError, setCreateError] = useState<string | undefined>();

  useEffect(() => {
    if (open) {
      setMode('list');
      setDraftId(selectedAccountId);
      setSearch('');
      setCreateError(undefined);
    }
  }, [open, selectedAccountId]);

  // Follow the provider kind while the create form is not being edited, so the
  // vendor pre-selection tracks the cloud being configured.
  useEffect(() => {
    if (mode !== 'create') {
      setVendorCode(defaultVendorCode);
      setAccountCode(generateAccountCode(defaultVendorCode));
    }
  }, [defaultVendorCode, mode]);

  // Vendor linkage first, then the free-text search: a provider kind only ever
  // binds its own vendor's accounts, so out-of-family rows stay invisible even
  // when their name matches the search.
  const scopedAccounts = useMemo(() => {
    const items = accounts ?? [];
    if (!allowedVendorCodes || allowedVendorCodes.length === 0) {
      return items;
    }
    const allowed = new Set(allowedVendorCodes.map((vendor) => vendor.toLowerCase()));
    return items.filter((account) => allowed.has(account.vendorCode.toLowerCase()));
  }, [accounts, allowedVendorCodes]);

  const visibleAccounts = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) {
      return scopedAccounts;
    }
    return scopedAccounts.filter((account) =>
      [account.displayName, account.vendorCode, account.accountCode]
        .join(' ')
        .toLowerCase()
        .includes(keyword),
    );
  }, [scopedAccounts, search]);

  // The editor links a provider kind to exactly one vendor and sends that
  // vendor to the server as the list's `vendorCode` window filter, so with a
  // single allowed vendor an empty result means "this vendor has no account
  // yet" rather than "the platform has none".
  const vendorScoped = Boolean(allowedVendorCodes && allowedVendorCodes.length === 1);

  const submitNewAccount = async () => {
    if (!displayName.trim() || !accessKeyId.trim() || !secretAccessKey.trim()) {
      setCreateError(t('required'));
      return;
    }
    setCreating(true);
    setCreateError(undefined);
    try {
      const created = await onCreateProviderAccount({
        displayName: displayName.trim(),
        vendorCode,
        accountCode: accountCode.trim() || generateAccountCode(vendorCode),
        environment: 'production',
        scopeType: ACCOUNT_SCOPE,
        isDefault,
        accessKeyId: accessKeyId.trim(),
        secretAccessKey: secretAccessKey.trim(),
        ...(sessionToken.trim() ? { sessionToken: sessionToken.trim() } : {}),
      });
      onSelectedAccountChange(created.id);
      onReloadAccounts();
      onClose();
    } catch (submitError) {
      setCreateError(
        submitError instanceof DriveAdminStorageSdkError && submitError.status === 403
          ? t('accountPlatformRequired')
          : submitError instanceof Error && submitError.message
            ? submitError.message
            : t('accountCreateFailed'),
      );
    } finally {
      setCreating(false);
    }
  };

  const resetCreateForm = () => {
    setDisplayName('');
    setAccessKeyId('');
    setSecretAccessKey('');
    setSessionToken('');
    setIsDefault(false);
    setCreateError(undefined);
    setVendorCode(defaultVendorCode);
    setAccountCode(generateAccountCode(defaultVendorCode));
  };

  return (
    <Modal
      open={open}
      onOpenChange={(next) => {
        if (!next && !creating) {
          onClose();
        }
      }}
    >
      <ModalContent size="lg">
        <ModalHeader>
          <ModalTitle>{mode === 'list' ? t('accountPickerTitle') : t('accountNewTitle')}</ModalTitle>
          <ModalDescription>
            {mode === 'list' ? t('accountPickerDescription') : t('accountScopeFixedHint')}
          </ModalDescription>
        </ModalHeader>
        <ModalBody>
          {mode === 'list' ? (
            <div className="flex flex-col gap-3">
              <div className="flex items-center gap-2">
                <div className="relative min-w-0 flex-1">
                  <Search
                    aria-hidden="true"
                    size={14}
                    className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400"
                  />
                  <input
                    value={search}
                    onChange={(event) => setSearch(event.target.value)}
                    className={`${INPUT_CLASS} pl-8 text-xs`}
                    placeholder={t('accountPickerSearchPlaceholder')}
                  />
                </div>
                {/* The empty state carries its own prominent create action, so
                    the header button only shows once there is a list beside it. */}
                {!loading && visibleAccounts.length > 0 && (
                  <button
                    type="button"
                    className={`${SECONDARY_BUTTON_CLASS} shrink-0`}
                    disabled={creating}
                    onClick={() => {
                      resetCreateForm();
                      setMode('create');
                    }}
                  >
                    <Plus aria-hidden="true" size={14} />
                    {t('accountNew')}
                  </button>
                )}
              </div>

              {error && (
                <p className="flex flex-wrap items-center gap-2 text-[11px] text-amber-600 dark:text-amber-400">
                  {error}
                  <button
                    type="button"
                    className="inline-flex items-center gap-1 font-semibold text-blue-600 hover:underline dark:text-blue-400"
                    onClick={onReloadAccounts}
                  >
                    <RefreshCw aria-hidden="true" size={11} />
                    {t('accountReload')}
                  </button>
                </p>
              )}

              {loading ? (
                <div className="flex min-h-40 items-center justify-center rounded-lg border border-neutral-200 text-xs text-neutral-500 dark:border-neutral-700">
                  {t('accountsLoading')}
                </div>
              ) : visibleAccounts.length === 0 ? (
                (() => {
                  const platformEmpty =
                    (accounts ?? []).length === 0 && !search.trim() && !vendorScoped;
                  const emptyTitle = search.trim()
                    ? t('accountPickerSearchEmpty')
                    : platformEmpty
                      ? t('accountPickerEmptyTitle')
                      : t('accountPickerVendorEmptyTitle');
                  const emptyDesc = search.trim()
                    ? undefined
                    : platformEmpty
                      ? t('accountPickerEmptyDesc')
                      : t('accountPickerVendorEmptyDesc');
                  return (
                    <div className="flex min-h-40 flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-neutral-300 px-6 py-8 text-center dark:border-neutral-700">
                      <div className="flex h-10 w-10 items-center justify-center rounded-full bg-blue-50 text-blue-500 dark:bg-blue-950/40 dark:text-blue-300">
                        <Cloud aria-hidden="true" size={18} />
                      </div>
                      <div>
                        <div className="text-sm font-medium text-neutral-800 dark:text-neutral-100">
                          {emptyTitle}
                        </div>
                        {emptyDesc && (
                          <p className="mt-1 text-[11px] text-neutral-500 dark:text-neutral-400">
                            {emptyDesc}
                          </p>
                        )}
                      </div>
                      {!search.trim() && (
                        <button
                          type="button"
                          className={PRIMARY_BUTTON_CLASS}
                          onClick={() => {
                            resetCreateForm();
                            setMode('create');
                          }}
                        >
                          <Plus aria-hidden="true" size={14} />
                          {t('accountNew')}
                        </button>
                      )}
                    </div>
                  );
                })()
              ) : (
                <ul className="flex max-h-80 flex-col gap-1.5 overflow-y-auto pr-0.5">
                  {visibleAccounts.map((account) => {
                    const disabled = account.status !== 'active';
                    const checked = draftId === account.id;
                    return (
                      <li key={account.id}>
                        <button
                          type="button"
                          disabled={disabled}
                          aria-pressed={checked}
                          onClick={() => setDraftId(account.id)}
                          className={`flex w-full items-start gap-3 rounded-lg border px-3 py-2.5 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
                            checked
                              ? 'border-blue-400 bg-blue-50 dark:border-blue-500/60 dark:bg-blue-950/40'
                              : 'border-neutral-200 bg-white hover:border-neutral-300 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-900 dark:hover:border-neutral-600 dark:hover:bg-neutral-800'
                          }`}
                        >
                          <span
                            aria-hidden="true"
                            className={`mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full border ${
                              checked
                                ? 'border-blue-600 bg-blue-600 dark:border-blue-400 dark:bg-blue-400'
                                : 'border-neutral-300 dark:border-neutral-600'
                            }`}
                          >
                            {checked && <span className="h-1.5 w-1.5 rounded-full bg-white" />}
                          </span>
                          <span className="min-w-0 flex-1">
                            <span className="flex flex-wrap items-center gap-1.5">
                              <span className="truncate text-xs font-semibold text-neutral-900 dark:text-neutral-100">
                                {account.displayName}
                              </span>
                              <StorageProviderAccountBadges account={account} />
                            </span>
                            <span className="mt-0.5 block truncate font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
                              {account.vendorCode}/{account.accountCode}
                            </span>
                          </span>
                        </button>
                      </li>
                    );
                  })}
                </ul>
              )}
              {/* The loaded rows are one page of a paginated list, so the
                  continuation is rendered from the server's own `hasMore`
                  (`PAGINATION_SPEC.md` §8) instead of being inferred from a
                  full page — an operator looking for an account that sorts
                  past the window has to be able to reach it. */}
              {!loading && hasMoreAccounts && onLoadMoreAccounts && (
                <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-neutral-200 bg-neutral-50 px-3 py-2 dark:border-neutral-700 dark:bg-neutral-800/60">
                  <span className="text-[11px] text-neutral-500 dark:text-neutral-400">
                    {t('accountLoadMoreHint')}
                  </span>
                  <button
                    type="button"
                    className={SECONDARY_BUTTON_CLASS}
                    disabled={loadingMoreAccounts}
                    onClick={onLoadMoreAccounts}
                  >
                    {loadingMoreAccounts ? t('accountLoadingMore') : t('accountLoadMore')}
                  </button>
                </div>
              )}
            </div>
          ) : (
            <div className="flex flex-col gap-3">
              <div className="flex items-center gap-2 rounded-md border border-blue-200 bg-blue-50 px-3 py-2 text-[11px] text-blue-700 dark:border-blue-900 dark:bg-blue-950/40 dark:text-blue-300">
                <KeyRound aria-hidden="true" size={13} className="shrink-0" />
                <span className="min-w-0 flex-1">{t('accountReuseHelp')}</span>
              </div>
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                <DialogField label={t('accountDisplayName')}>
                  <input
                    value={displayName}
                    onChange={(event) => setDisplayName(event.target.value)}
                    className={INPUT_CLASS}
                    placeholder={t('accountDisplayNamePlaceholder')}
                    autoComplete="off"
                  />
                </DialogField>
                {allowedVendorCodes && allowedVendorCodes.length === 1 ? (
                  // Vendor linkage pins the create form: an Aliyun provider
                  // registers Aliyun accounts, so the field reads back the
                  // pinned vendor instead of offering the catalog.
                  <DialogField label={t('accountVendorCode')} hint={t('accountVendorFixedHint')}>
                    <div className="flex h-9 items-center rounded-md border border-neutral-200 bg-neutral-50 px-3 font-mono text-xs text-neutral-600 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300">
                      {vendorCode}
                    </div>
                  </DialogField>
                ) : (
                  <DialogField label={t('accountVendorCode')}>
                    <select
                      value={vendorCode}
                      onChange={(event) => {
                        setVendorCode(event.target.value);
                        setAccountCode(generateAccountCode(event.target.value));
                      }}
                      className={SELECT_CLASS}
                    >
                      {(allowedVendorCodes && allowedVendorCodes.length > 0
                        ? ACCOUNT_VENDOR_CODES.filter((vendor) =>
                            allowedVendorCodes.some(
                              (allowed) => allowed.toLowerCase() === vendor.toLowerCase(),
                            ),
                          )
                        : ACCOUNT_VENDOR_CODES
                      ).map((vendor) => (
                        <option key={vendor} value={vendor}>{vendor}</option>
                      ))}
                    </select>
                  </DialogField>
                )}
                <DialogField label={t('accountCode')} hint={t('accountCodeHelp')}>
                  <input
                    value={accountCode}
                    onChange={(event) => setAccountCode(event.target.value)}
                    className={`${INPUT_CLASS} font-mono text-xs`}
                    placeholder="aliyun-main-a1b2c3d4"
                    autoComplete="off"
                  />
                </DialogField>
                <DialogField label={t('accountScopeLabel')} hint={t('accountScopeFixedHint')}>
                  <div className="flex h-9 items-center rounded-md border border-neutral-200 bg-neutral-50 px-3 text-xs text-neutral-600 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-300">
                    {t('accountScopePlatform')}
                  </div>
                </DialogField>
                <DialogField label={credentialFields.accessKeyLabel || t('accountAccessKeyId')}>
                  <input
                    value={accessKeyId}
                    onChange={(event) => setAccessKeyId(event.target.value)}
                    className={`${INPUT_CLASS} font-mono text-xs`}
                    autoComplete="off"
                  />
                </DialogField>
                <DialogField label={credentialFields.secretKeyLabel || t('accountSecretAccessKey')}>
                  <input
                    value={secretAccessKey}
                    onChange={(event) => setSecretAccessKey(event.target.value)}
                    type="password"
                    className={`${INPUT_CLASS} font-mono text-xs`}
                    autoComplete="new-password"
                  />
                </DialogField>
                <DialogField label={t('accountSessionToken')}>
                  <input
                    value={sessionToken}
                    onChange={(event) => setSessionToken(event.target.value)}
                    type="password"
                    className={INPUT_CLASS}
                    autoComplete="new-password"
                  />
                </DialogField>
              </div>
              <label className="flex items-start gap-2">
                <input
                  type="checkbox"
                  className={`mt-0.5 ${CHECKBOX_CLASS}`}
                  checked={isDefault}
                  onChange={(event) => setIsDefault(event.target.checked)}
                />
                <span className="flex flex-col">
                  <span className="text-xs font-medium text-neutral-600 dark:text-neutral-300">
                    {t('accountDefaultLabel')}
                  </span>
                  <span className="text-[10px] text-neutral-400">{t('accountDefaultHelp')}</span>
                </span>
              </label>
              {createError && <p className="text-[11px] text-red-500">{createError}</p>}
            </div>
          )}
        </ModalBody>
        <ModalFooter>
          {mode === 'list' ? (
            <>
              <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={creating} onClick={onClose}>
                {t('cancel')}
              </button>
              <button
                type="button"
                className={PRIMARY_BUTTON_CLASS}
                disabled={draftId === '' || draftId === selectedAccountId || creating}
                onClick={() => {
                  onSelectedAccountChange(draftId);
                  onClose();
                }}
              >
                {t('accountPickerUse')}
              </button>
            </>
          ) : (
            <>
              <button
                type="button"
                className={SECONDARY_BUTTON_CLASS}
                disabled={creating}
                onClick={() => {
                  setMode('list');
                  setCreateError(undefined);
                }}
              >
                {t('accountPickerBackToList')}
              </button>
              <button
                type="button"
                className={PRIMARY_BUTTON_CLASS}
                disabled={creating}
                onClick={() => {
                  void submitNewAccount();
                }}
              >
                {creating ? t('accountCreating') : t('accountCreate')}
              </button>
            </>
          )}
        </ModalFooter>
      </ModalContent>
    </Modal>
  );
}

function generateAccountCode(vendorCode: string): string {
  const random = Math.random().toString(36).slice(2, 10).padEnd(8, '0');
  return `${vendorCode}-main-${random}`;
}

function DialogField({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-xs font-medium text-neutral-600 dark:text-neutral-300">{label}</span>
      {children}
      {hint && <span className="text-[10px] text-neutral-400">{hint}</span>}
    </label>
  );
}
