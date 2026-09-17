import React, { useEffect, useMemo, useState } from 'react';
import { ExternalLink, RefreshCw } from 'lucide-react';
import type { ProviderCredentialFieldMeta } from '../utils/providerKindConfig';
import type {
  CreateStorageProviderAccountInput,
  StorageProviderAccountScope,
  StorageProviderAccountView,
} from '../types/storageProviderAdminTypes';
import {
  buildCredentialRef,
  isCredentialRefMasked,
  parseCredentialRef,
  type CredentialInputMode,
} from '../utils/credentialRefUtils';
import { INPUT_CLASS, PRIMARY_BUTTON_CLASS, SECONDARY_BUTTON_CLASS, SELECT_CLASS } from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

export type CredentialSource = 'account' | 'manual';

/**
 * Which slice of the account centre the picker shows.
 *
 * `all` is the default because the whole point of a shared account centre is
 * that a tenant reuses a platform-wide account it did not create. `mine` maps
 * to the server's `mine=true`, which pins the query to the caller's personal
 * scope without the client having to know its own user id.
 */
export type AccountScopeFilter = 'all' | 'mine' | 'tenant' | 'platform';

const ACCOUNT_SCOPE_FILTERS: readonly AccountScopeFilter[] = [
  'all',
  'mine',
  'tenant',
  'platform',
] as const;

/**
 * Scopes a new account can be created in, narrowest first.
 *
 * Deliberately excludes `user`: a storage provider is a tenant-level resource
 * (see `ensure_bindable_provider_account` on the backend), so a personal account
 * could never be bound to it. Offering the choice here would produce a 409 on
 * submit. Personal accounts are still *listed* under the "mine" filter so an
 * operator can see their own accounts and why they are not bindable.
 */
const ACCOUNT_CREATE_SCOPES: readonly StorageProviderAccountScope[] = [
  'tenant',
  'platform',
] as const;

const ACCOUNT_SCOPE_LABEL_KEY: Record<StorageProviderAccountScope, string> = {
  user: 'accountScopeUser',
  tenant: 'accountScopeTenant',
  platform: 'accountScopePlatform',
};

const ACCOUNT_VENDOR_CODES = [
  'aliyun',
  'tencent',
  'huawei',
  'volcengine',
  'aws',
  'google',
  'azure',
  'minio',
  'custom',
] as const;

interface StorageProviderCredentialFieldsProps {
  credentialFields: ProviderCredentialFieldMeta;
  credentialRef: string;
  isEditing: boolean;
  credentialConfigured?: boolean;
  error?: string;
  onCredentialRefChange: (value: string) => void;
  /** Whether the host injected the account-center callbacks. */
  accountSourceEnabled: boolean;
  /** Currently referenced reusable account id (empty string = none). */
  providerAccountId: string;
  onProviderAccountIdChange: (value: string) => void;
  /** Reusable accounts loaded from the platform account center. */
  providerAccounts: StorageProviderAccountView[] | undefined;
  accountsLoading: boolean;
  accountsError?: string;
  onReloadProviderAccounts: () => void;
  /** Which scope slice the list currently shows. */
  accountScopeFilter: AccountScopeFilter;
  onAccountScopeFilterChange: (value: AccountScopeFilter) => void;
  onCreateProviderAccount: (
    input: CreateStorageProviderAccountInput,
  ) => Promise<StorageProviderAccountView>;
  /** Vendor pre-selection for the "new account" form, derived from the kind. */
  defaultVendorCode: string;
}

/**
 * Credential source picker for one storage provider.
 *
 * Two mutually exclusive sources exist, mirroring the database constraint:
 * - `account`: reference a reusable service-provider account held by the
 *   platform account center. The operator picks an existing account from the
 *   list or registers a new one (display name + access key pair) inline, so
 *   every credential becomes reusable across businesses.
 * - `manual`: the legacy direct / env / secret credential_ref forms.
 */
export function StorageProviderCredentialFields({
  credentialFields,
  credentialRef,
  isEditing,
  credentialConfigured,
  error,
  onCredentialRefChange,
  accountSourceEnabled,
  providerAccountId,
  onProviderAccountIdChange,
  providerAccounts,
  accountsLoading,
  accountsError,
  onReloadProviderAccounts,
  accountScopeFilter,
  onAccountScopeFilterChange,
  onCreateProviderAccount,
  defaultVendorCode,
}: StorageProviderCredentialFieldsProps) {
  const { t } = useTranslation();
  const parsed = useMemo(() => parseCredentialRef(credentialRef), [credentialRef]);
  const masked = isCredentialRefMasked(credentialRef);
  const hasAccountReference = providerAccountId.trim() !== '';

  const [source, setSource] = useState<CredentialSource>(
    // The account center is the recommended default for new providers; an
    // existing provider opens on the source it is already bound to.
    accountSourceEnabled && (!isEditing || hasAccountReference) ? 'account' : 'manual',
  );
  const [mode, setMode] = useState<CredentialInputMode>(parsed?.mode ?? 'direct');
  const [accessKey, setAccessKey] = useState(parsed?.direct?.accessKey ?? '');
  const [secretKey, setSecretKey] = useState(parsed?.direct?.secretKey ?? '');
  const [accessKeyEnv, setAccessKeyEnv] = useState(
    parsed?.env?.accessKeyEnv ?? credentialFields.defaultEnvAccessKey,
  );
  const [secretKeyEnv, setSecretKeyEnv] = useState(
    parsed?.env?.secretKeyEnv ?? credentialFields.defaultEnvSecretKey,
  );
  const [secretRef, setSecretRef] = useState(parsed?.secret?.secretRef ?? '');
  const [showSecret, setShowSecret] = useState(false);
  const [replaceExisting, setReplaceExisting] = useState(!isEditing);

  // New-account inline form state.
  const [accountFormOpen, setAccountFormOpen] = useState(false);
  const [accountDisplayName, setAccountDisplayName] = useState('');
  const [accountVendorCode, setAccountVendorCode] = useState(defaultVendorCode);
  const [accountCode, setAccountCode] = useState(() => generateAccountCode(defaultVendorCode));
  const [accountAccessKeyId, setAccountAccessKeyId] = useState('');
  const [accountSecretAccessKey, setAccountSecretAccessKey] = useState('');
  const [accountSessionToken, setAccountSessionToken] = useState('');
  // A new account defaults to the tenant scope: it is the widest scope an
  // ordinary operator may publish, and it is what "the tenant's default cloud
  // account" means. Personal accounts are an explicit choice.
  const [accountScope, setAccountScope] = useState<StorageProviderAccountScope>('tenant');
  const [accountIsDefault, setAccountIsDefault] = useState(false);
  const [accountCreating, setAccountCreating] = useState(false);
  const [accountError, setAccountError] = useState<string | undefined>();

  useEffect(() => {
    if (parsed) {
      setMode(parsed.mode);
      if (parsed.direct) {
        setAccessKey(parsed.direct.accessKey);
        setSecretKey(parsed.direct.secretKey);
      }
      if (parsed.env) {
        setAccessKeyEnv(parsed.env.accessKeyEnv);
        setSecretKeyEnv(parsed.env.secretKeyEnv);
      }
      if (parsed.secret) {
        setSecretRef(parsed.secret.secretRef);
      }
    }
  }, [parsed]);

  // Follow the provider kind while the inline account form is not being
  // edited, so the vendor pre-selection tracks the selected cloud.
  useEffect(() => {
    if (!accountFormOpen) {
      setAccountVendorCode(defaultVendorCode);
      setAccountCode(generateAccountCode(defaultVendorCode));
    }
  }, [defaultVendorCode, accountFormOpen]);

  useEffect(() => {
    if (isEditing && credentialConfigured && !replaceExisting) {
      return;
    }

    const built =
      mode === 'direct'
        ? buildCredentialRef('direct', { accessKey, secretKey })
        : mode === 'env'
          ? buildCredentialRef('env', { accessKeyEnv, secretKeyEnv })
          : buildCredentialRef('secret', { secretRef });

    const nextRef = built ?? '';
    if (source === 'manual' && nextRef !== credentialRef) {
      onCredentialRefChange(nextRef);
    }
  }, [
    mode,
    accessKey,
    secretKey,
    accessKeyEnv,
    secretKeyEnv,
    secretRef,
    source,
    isEditing,
    credentialConfigured,
    replaceExisting,
    credentialRef,
    onCredentialRefChange,
  ]);

  const switchSource = (next: CredentialSource) => {
    setSource(next);
    if (next === 'account') {
      // Switching to the account center clears the local credential ref.
      if (credentialRef !== '') {
        onCredentialRefChange('');
      }
    } else {
      // Switching to a manual ref clears the account reference.
      if (providerAccountId !== '') {
        onProviderAccountIdChange('');
      }
    }
  };

  const submitNewAccount = async () => {
    if (!accountDisplayName.trim() || !accountAccessKeyId.trim() || !accountSecretAccessKey.trim()) {
      setAccountError(t('required'));
      return;
    }
    setAccountCreating(true);
    setAccountError(undefined);
    try {
      const created = await onCreateProviderAccount({
        displayName: accountDisplayName.trim(),
        vendorCode: accountVendorCode,
        accountCode: accountCode.trim() || generateAccountCode(accountVendorCode),
        environment: 'production',
        scopeType: accountScope,
        isDefault: accountIsDefault,
        accessKeyId: accountAccessKeyId.trim(),
        secretAccessKey: accountSecretAccessKey.trim(),
        ...(accountSessionToken.trim() ? { sessionToken: accountSessionToken.trim() } : {}),
      });
      onProviderAccountIdChange(created.id);
      // The new account may live outside the slice currently listed (a personal
      // account while "tenant default" is selected, say). Widen the filter so
      // the freshly bound account is actually visible in the select.
      if (
        (accountScopeFilter === 'platform' && created.scopeType !== 'platform') ||
        (accountScopeFilter === 'tenant' && created.scopeType !== 'tenant') ||
        (accountScopeFilter === 'mine' && created.scopeType !== 'user')
      ) {
        onAccountScopeFilterChange('all');
      }
      setAccountFormOpen(false);
      setAccountAccessKeyId('');
      setAccountSecretAccessKey('');
      setAccountSessionToken('');
      setAccountDisplayName('');
      setAccountIsDefault(false);
      setAccountCode(generateAccountCode(accountVendorCode));
    } catch (submitError) {
      setAccountError(
        submitError instanceof Error && submitError.message
          ? submitError.message
          : t('accountCreateFailed'),
      );
    } finally {
      setAccountCreating(false);
    }
  };

  const sourceButtonClass = (value: CredentialSource) =>
    `rounded-md px-2.5 py-1 text-[11px] font-medium transition-colors ${
      source === value
        ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300'
        : 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800'
    }`;

  const modeButtonClass = (value: CredentialInputMode) =>
    `rounded-md px-2.5 py-1 text-[11px] font-medium transition-colors ${
      mode === value
        ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300'
        : 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800'
    }`;

  const scopeFilterButtonClass = (value: AccountScopeFilter) =>
    `rounded-md px-2.5 py-1 text-[11px] font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-60 ${
      accountScopeFilter === value
        ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300'
        : 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800'
    }`;

  const scopeFilterLabel = (value: AccountScopeFilter): string =>
    value === 'all'
      ? t('accountScopeFilterAll')
      : value === 'mine'
        ? t('accountScopeFilterMine')
        : value === 'tenant'
          ? t('accountScopeFilterTenant')
          : t('accountScopeFilterPlatform');

  return (
    <div className="rounded-lg border border-neutral-200 bg-neutral-50/80 p-4 dark:border-neutral-700 dark:bg-neutral-900/40">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <div className="text-xs font-semibold text-neutral-800 dark:text-neutral-100">
            {t('credentialSectionTitle')}
          </div>
          <p className="mt-0.5 text-[11px] text-neutral-500 dark:text-neutral-400">
            {t('credentialSectionDesc')}
          </p>
        </div>
        {credentialFields.consoleUrl && (
          <a
            href={credentialFields.consoleUrl}
            target="_blank"
            rel="noreferrer"
            className="text-[11px] font-medium text-blue-600 hover:underline dark:text-blue-400"
          >
            <span className="inline-flex items-center gap-1">
              {t('openClawConsole')}
              <ExternalLink aria-hidden="true" size={12} />
            </span>
          </a>
        )}
      </div>

      <div className="mt-3 flex flex-wrap gap-1 rounded-md border border-neutral-200 bg-white p-1 dark:border-neutral-700 dark:bg-neutral-900">
        {accountSourceEnabled && (
          <button type="button" className={sourceButtonClass('account')} onClick={() => switchSource('account')}>
            {t('credentialSourceAccount')}
          </button>
        )}
        <button type="button" className={sourceButtonClass('manual')} onClick={() => switchSource('manual')}>
          {t('credentialSourceManual')}
        </button>
      </div>

      {error && <p className="mt-2 text-[11px] text-red-500">{error}</p>}
      {accountsError && (
        <p className="mt-2 flex flex-wrap items-center gap-2 text-[11px] text-amber-600 dark:text-amber-400">
          {accountsError}
          <button
            type="button"
            className="inline-flex items-center gap-1 font-semibold text-blue-600 hover:underline dark:text-blue-400"
            onClick={onReloadProviderAccounts}
          >
            <RefreshCw aria-hidden="true" size={11} />
            {t('accountReload')}
          </button>
        </p>
      )}

      {source === 'account' ? (
        <div className="mt-3 space-y-3">
          <div className="flex flex-wrap items-center gap-1 rounded-md border border-neutral-200 bg-white p-1 dark:border-neutral-700 dark:bg-neutral-900">
            <span className="px-1.5 text-[11px] font-medium text-neutral-500 dark:text-neutral-400">
              {t('accountScopeLabel')}
            </span>
            {ACCOUNT_SCOPE_FILTERS.map((value) => (
              <button
                key={value}
                type="button"
                className={scopeFilterButtonClass(value)}
                disabled={accountsLoading || accountFormOpen}
                onClick={() => onAccountScopeFilterChange(value)}
              >
                {scopeFilterLabel(value)}
              </button>
            ))}
          </div>
          <p className="text-[11px] text-neutral-500 dark:text-neutral-400">
            {accountScopeFilter === 'mine' ? t('accountScopeMineHint') : t('accountScopeHelp')}
          </p>

          <div className="grid grid-cols-1 gap-3 sm:grid-cols-[1fr_auto] sm:items-end">
            <label className="flex flex-col gap-1">
              <span className="text-xs font-medium text-neutral-600 dark:text-neutral-300">
                {t('accountSelectLabel')}
              </span>
              <select
                value={providerAccountId}
                onChange={(event) => onProviderAccountIdChange(event.target.value)}
                className={SELECT_CLASS}
                disabled={accountsLoading || accountFormOpen}
              >
                <option value="">{providerAccounts?.length ? t('accountSelectPlaceholder') : accountsLoading ? t('accountsLoading') : accountScopeFilter === 'all' ? t('accountNoAccounts') : t('accountScopeEmpty')}</option>
                {(providerAccounts ?? []).map((account) => {
                  // A personal account cannot back a tenant-level provider; the
                  // server rejects the bind with 409, so the option is disabled
                  // here instead of letting the submit fail.
                  const scopeBindable = account.scopeType !== 'user';
                  return (
                    <option
                      key={account.id}
                      value={account.id}
                      disabled={account.status !== 'active' || !scopeBindable}
                    >
                      {account.isDefault ? `★ ${t('accountDefaultBadge')} · ` : ''}
                      {account.displayName} · {account.vendorCode}/{account.accountCode}
                      {` · ${t(ACCOUNT_SCOPE_LABEL_KEY[account.scopeType])}`}
                      {account.environment !== 'production' ? ` (${account.environment})` : ''}
                      {account.status !== 'active' ? ` · ${account.status === 'disabled' ? t('accountStatusDisabled') : account.status}` : ''}
                      {!scopeBindable ? ` · ${t('accountScopeUser')}` : ''}
                    </option>
                  );
                })}
              </select>
            </label>
            <button
              type="button"
              className={`${SECONDARY_BUTTON_CLASS} shrink-0`}
              disabled={accountsLoading}
              onClick={onReloadProviderAccounts}
            >
              <RefreshCw aria-hidden="true" size={13} className={accountsLoading ? 'animate-spin' : undefined} />
              {t('accountReload')}
            </button>
          </div>
          <p className="text-[11px] text-neutral-500 dark:text-neutral-400">{t('accountReuseHelp')}</p>
          <p className="text-[11px] text-neutral-500 dark:text-neutral-400">{t('accountRotationHint')}</p>

          {accountFormOpen ? (
            <div className="space-y-3 rounded-md border border-neutral-200 bg-white p-3 dark:border-neutral-700 dark:bg-neutral-900">
              <div className="text-xs font-semibold text-neutral-800 dark:text-neutral-100">
                {t('accountNewTitle')}
              </div>
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                <AccountField label={t('accountDisplayName')}>
                  <input
                    value={accountDisplayName}
                    onChange={(event) => setAccountDisplayName(event.target.value)}
                    className={INPUT_CLASS}
                    placeholder={t('accountDisplayNamePlaceholder')}
                    autoComplete="off"
                  />
                </AccountField>
                <AccountField label={t('accountVendorCode')}>
                  <select
                    value={accountVendorCode}
                    onChange={(event) => {
                      setAccountVendorCode(event.target.value);
                      setAccountCode(generateAccountCode(event.target.value));
                    }}
                    className={SELECT_CLASS}
                  >
                    {ACCOUNT_VENDOR_CODES.map((vendor) => (
                      <option key={vendor} value={vendor}>{vendor}</option>
                    ))}
                  </select>
                </AccountField>
                <AccountField label={t('accountCode')} hint={t('accountCodeHelp')}>
                  <input
                    value={accountCode}
                    onChange={(event) => setAccountCode(event.target.value)}
                    className={`${INPUT_CLASS} font-mono text-xs`}
                    placeholder="aliyun-main-a1b2c3d4"
                    autoComplete="off"
                  />
                </AccountField>
                <AccountField
                  label={t('accountScopeLabel')}
                  hint={accountScope === 'platform' ? t('accountScopePlatformHint') : undefined}
                >
                  <select
                    value={accountScope}
                    onChange={(event) =>
                      setAccountScope(event.target.value as StorageProviderAccountScope)
                    }
                    className={SELECT_CLASS}
                  >
                    {ACCOUNT_CREATE_SCOPES.map((scope) => (
                      <option key={scope} value={scope}>
                        {t(ACCOUNT_SCOPE_LABEL_KEY[scope])}
                      </option>
                    ))}
                  </select>
                </AccountField>
                <AccountField label={t('accountSessionToken')}>
                  <input
                    value={accountSessionToken}
                    onChange={(event) => setAccountSessionToken(event.target.value)}
                    type="password"
                    className={INPUT_CLASS}
                    autoComplete="new-password"
                  />
                </AccountField>
                <AccountField label={credentialFields.accessKeyLabel || t('accountAccessKeyId')}>
                  <input
                    value={accountAccessKeyId}
                    onChange={(event) => setAccountAccessKeyId(event.target.value)}
                    className={`${INPUT_CLASS} font-mono text-xs`}
                    autoComplete="off"
                  />
                </AccountField>
                <AccountField label={credentialFields.secretKeyLabel || t('accountSecretAccessKey')}>
                  <input
                    value={accountSecretAccessKey}
                    onChange={(event) => setAccountSecretAccessKey(event.target.value)}
                    type="password"
                    className={`${INPUT_CLASS} font-mono text-xs`}
                    autoComplete="new-password"
                  />
                </AccountField>
              </div>
              <label className="flex items-start gap-2">
                <input
                  type="checkbox"
                  className="mt-0.5"
                  checked={accountIsDefault}
                  onChange={(event) => setAccountIsDefault(event.target.checked)}
                />
                <span className="flex flex-col">
                  <span className="text-xs font-medium text-neutral-600 dark:text-neutral-300">
                    {t('accountDefaultLabel')}
                  </span>
                  <span className="text-[10px] text-neutral-400">{t('accountDefaultHelp')}</span>
                </span>
              </label>
              {accountError && <p className="text-[11px] text-red-500">{accountError}</p>}
              <div className="flex flex-wrap gap-2">
                <button
                  type="button"
                  className={PRIMARY_BUTTON_CLASS}
                  disabled={accountCreating}
                  onClick={() => {
                    void submitNewAccount();
                  }}
                >
                  {accountCreating ? t('accountCreating') : t('accountCreate')}
                </button>
                <button
                  type="button"
                  className={SECONDARY_BUTTON_CLASS}
                  disabled={accountCreating}
                  onClick={() => {
                    setAccountFormOpen(false);
                    setAccountError(undefined);
                  }}
                >
                  {t('cancel')}
                </button>
              </div>
            </div>
          ) : (
            <button
              type="button"
              className={SECONDARY_BUTTON_CLASS}
              disabled={accountCreating}
              onClick={() => setAccountFormOpen(true)}
            >
              + {t('accountNew')}
            </button>
          )}
        </div>
      ) : isEditing && credentialConfigured && masked && !replaceExisting ? (
        <div className="mt-3 rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2.5 dark:border-emerald-900 dark:bg-emerald-950/30">
          <div className="text-xs font-medium text-emerald-800 dark:text-emerald-200">
            {t('credentialAlreadyConfigured')}
          </div>
          <p className="mt-0.5 text-[11px] text-emerald-700 dark:text-emerald-300">
            {t('credentialAlreadyConfiguredDesc')}
          </p>
          <button
            type="button"
            className="mt-2 text-[11px] font-semibold text-blue-600 hover:underline dark:text-blue-400"
            onClick={() => setReplaceExisting(true)}
          >
            {t('replaceCredential')}
          </button>
        </div>
      ) : (
        <>
          <div className="mt-3 flex flex-wrap gap-1 rounded-md border border-neutral-200 bg-white p-1 dark:border-neutral-700 dark:bg-neutral-900">
            <button type="button" className={modeButtonClass('direct')} onClick={() => setMode('direct')}>
              {t('credentialModeDirect')}
            </button>
            <button type="button" className={modeButtonClass('env')} onClick={() => setMode('env')}>
              {t('credentialModeEnv')}
            </button>
            <button type="button" className={modeButtonClass('secret')} onClick={() => setMode('secret')}>
              {t('credentialModeSecret')}
            </button>
          </div>

          {mode === 'direct' && (
            <div className="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-2">
              <CredentialField
                label={credentialFields.accessKeyLabel}
                value={accessKey}
                onChange={setAccessKey}
                placeholder={credentialFields.accessKeyPlaceholder}
              />
              <CredentialField
                label={credentialFields.secretKeyLabel}
                value={secretKey}
                onChange={setSecretKey}
                placeholder={credentialFields.secretKeyPlaceholder}
                secret
                showSecret={showSecret}
                onToggleSecret={() => setShowSecret(!showSecret)}
              />
            </div>
          )}

          {mode === 'env' && (
            <div className="mt-3 space-y-3">
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                <CredentialField
                  label={t('credentialEnvAccessKey')}
                  value={accessKeyEnv}
                  onChange={setAccessKeyEnv}
                  placeholder={credentialFields.defaultEnvAccessKey}
                />
                <CredentialField
                  label={t('credentialEnvSecretKey')}
                  value={secretKeyEnv}
                  onChange={setSecretKeyEnv}
                  placeholder={credentialFields.defaultEnvSecretKey}
                />
              </div>
              <p className="text-[11px] text-neutral-500 dark:text-neutral-400">{t('credentialEnvHelp')}</p>
            </div>
          )}

          {mode === 'secret' && (
            <div className="mt-3 space-y-2">
              <CredentialField
                label={t('credentialSecretRef')}
                value={secretRef}
                onChange={setSecretRef}
                placeholder="production/cos-main"
              />
              <p className="text-[11px] text-neutral-500 dark:text-neutral-400">{t('credentialSecretHelp')}</p>
            </div>
          )}
        </>
      )}
    </div>
  );
}

function generateAccountCode(vendorCode: string): string {
  const random = Math.random().toString(36).slice(2, 10).padEnd(8, '0');
  return `${vendorCode}-main-${random}`;
}

function AccountField({
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

function CredentialField({
  label,
  value,
  onChange,
  placeholder,
  secret,
  showSecret,
  onToggleSecret,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  secret?: boolean;
  showSecret?: boolean;
  onToggleSecret?: () => void;
}) {
  const { t } = useTranslation();
  return (
    <label className="flex flex-col gap-1">
      <span className="text-xs font-medium text-neutral-600 dark:text-neutral-300">{label}</span>
      <div className="relative">
        <input
          value={value}
          onChange={(e) => onChange(e.target.value)}
          type={secret && !showSecret ? 'password' : 'text'}
          className={`${INPUT_CLASS} ${secret ? 'pr-9 font-mono text-xs' : ''}`}
          placeholder={placeholder}
          autoComplete="off"
        />
        {secret && onToggleSecret && (
          <button
            type="button"
            className="absolute right-2 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300"
            aria-label={showSecret ? t('hideCredential') : t('showCredential')}
            title={showSecret ? t('hideCredential') : t('showCredential')}
            onClick={onToggleSecret}
          >
            <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d={
                  showSecret
                    ? 'M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242'
                    : 'M15 12a3 3 0 11-6 0 3 3 0 016 0zM2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z'
                }
              />
            </svg>
          </button>
        )}
      </div>
    </label>
  );
}
