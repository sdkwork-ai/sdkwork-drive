import React, { useEffect, useMemo, useState } from 'react';
import { CheckCircle2, Eye, EyeOff, ExternalLink, KeyRound, Replace, X } from 'lucide-react';
import type { ProviderCredentialFieldMeta } from '../utils/providerKindConfig';
import type {
  CreateStorageProviderAccountInput,
  StorageProviderAccountView,
} from '../types/storageProviderAdminTypes';
import {
  buildCredentialRef,
  isCredentialRefMasked,
  parseCredentialRef,
  type CredentialInputMode,
} from '../utils/credentialRefUtils';
import { INPUT_CLASS, SECONDARY_BUTTON_CLASS } from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';
import { StorageProviderAccountBadges } from './StorageProviderAccountBadges';
import { StorageProviderAccountPickerDialog } from './StorageProviderAccountPickerDialog';

export type CredentialSource = 'account' | 'manual';

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
  /** Reusable platform accounts loaded from the account center. */
  providerAccounts: StorageProviderAccountView[] | undefined;
  accountsLoading: boolean;
  accountsError?: string;
  onReloadProviderAccounts: () => void;
  onCreateProviderAccount: (
    input: CreateStorageProviderAccountInput,
  ) => Promise<StorageProviderAccountView>;
  /**
   * Vendor codes the provider's kind can bind; the picker dialog lists only
   * these accounts and pins its create form to them (kind ↔ vendor linkage).
   */
  allowedVendorCodes?: readonly string[];
  /** Vendor pre-selection for the "new account" form, derived from the kind. */
  defaultVendorCode: string;
}

/**
 * Credential source picker for one storage provider.
 *
 * Two mutually exclusive sources exist, mirroring the database constraint:
 * - `account` (the primary path): the provider references a reusable
 *   platform-wide account held by the account center. The section renders as
 *   one "select account" affordance; picking one opens the picker dialog,
 *   which also registers a new account inline when none exists yet. The bound
 *   account then reads back as its own summary row.
 * - `manual`: the legacy direct / env / secret credential_ref forms, demoted
 *   behind a muted toggle for deployments that resolve credentials from the
 *   environment instead of the account center.
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
  onCreateProviderAccount,
  allowedVendorCodes,
  defaultVendorCode,
}: StorageProviderCredentialFieldsProps) {
  const { t } = useTranslation();
  const parsed = useMemo(() => parseCredentialRef(credentialRef), [credentialRef]);
  const masked = isCredentialRefMasked(credentialRef);
  const hasAccountReference = providerAccountId.trim() !== '';

  const [source, setSource] = useState<CredentialSource>(
    // The account center is the primary path for new providers; an existing
    // provider opens on the source it is already bound to.
    accountSourceEnabled && (!isEditing || hasAccountReference) ? 'account' : 'manual',
  );
  const [pickerOpen, setPickerOpen] = useState(false);
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

  const selectedAccount = (providerAccounts ?? []).find(
    (account) => account.id === providerAccountId.trim(),
  );
  const modeButtonClass = (value: CredentialInputMode) =>
    `rounded-md px-2.5 py-1 text-[11px] font-medium transition-colors ${
      mode === value
        ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300'
        : 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-800'
    }`;

  return (
    <div className="rounded-lg border border-neutral-200 bg-neutral-50/80 p-4 dark:border-neutral-700 dark:bg-neutral-900/40">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <div className="text-xs font-semibold text-neutral-800 dark:text-neutral-100">
            {t('credentialSectionTitle')}
          </div>
          <p className="mt-0.5 text-[11px] text-neutral-500 dark:text-neutral-400">
            {accountSourceEnabled ? t('credentialSectionAccountDesc') : t('credentialSectionDesc')}
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

      {error && <p className="mt-2 text-[11px] text-red-500">{error}</p>}
      {accountsError && source === 'account' && (
        <p className="mt-2 flex flex-wrap items-center gap-2 text-[11px] text-amber-600 dark:text-amber-400">
          {accountsError}
          <button
            type="button"
            className="inline-flex items-center gap-1 font-semibold text-blue-600 hover:underline dark:text-blue-400"
            onClick={onReloadProviderAccounts}
          >
            {t('accountReload')}
          </button>
        </p>
      )}

      {accountSourceEnabled && source === 'account' ? (
        <div className="mt-3">
          {hasAccountReference ? (
            // The bound account reads back as its own full-width row: identity,
            // scope badges, and the two actions that re-open or drop the binding.
            <div className="rounded-lg border border-blue-200 bg-white px-3.5 py-3 dark:border-blue-900 dark:bg-neutral-900">
              <div className="flex flex-wrap items-start justify-between gap-2">
                <div className="flex min-w-0 items-start gap-2.5">
                  <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-blue-100 text-blue-600 dark:bg-blue-900/40 dark:text-blue-300">
                    <KeyRound aria-hidden="true" size={15} />
                  </div>
                  <div className="min-w-0">
                    <div className="flex flex-wrap items-center gap-1.5">
                      <span className="truncate text-xs font-semibold text-neutral-900 dark:text-neutral-100">
                        {selectedAccount?.displayName ?? providerAccountId}
                      </span>
                      {selectedAccount && <StorageProviderAccountBadges account={selectedAccount} />}
                    </div>
                    <div className="mt-0.5 truncate font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
                      {selectedAccount
                        ? `${selectedAccount.vendorCode}/${selectedAccount.accountCode}`
                        : t('accountsLoading')}
                    </div>
                  </div>
                </div>
                <div className="flex shrink-0 gap-1.5">
                  <button
                    type="button"
                    className={SECONDARY_BUTTON_CLASS}
                    disabled={accountsLoading}
                    onClick={() => setPickerOpen(true)}
                  >
                    <Replace aria-hidden="true" size={13} />
                    {t('accountChange')}
                  </button>
                  <button
                    type="button"
                    className={SECONDARY_BUTTON_CLASS}
                    disabled={accountsLoading}
                    onClick={() => onProviderAccountIdChange('')}
                  >
                    <X aria-hidden="true" size={13} />
                    {t('clear')}
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <button
              type="button"
              className={`flex w-full items-center justify-center gap-2 rounded-lg border border-dashed border-neutral-300 bg-white px-3 py-3 text-xs font-medium text-neutral-600 transition-colors hover:border-blue-400 hover:bg-blue-50/50 hover:text-blue-600 disabled:cursor-not-allowed disabled:opacity-60 dark:border-neutral-600 dark:bg-neutral-900 dark:text-neutral-300 dark:hover:border-blue-500 dark:hover:bg-blue-950/30 dark:hover:text-blue-300 ${
                accountsLoading ? 'opacity-60' : ''
              }`}
              disabled={accountsLoading}
              onClick={() => setPickerOpen(true)}
            >
              <KeyRound aria-hidden="true" size={14} />
              {accountsLoading ? t('accountsLoading') : t('accountPickCta')}
            </button>
          )}
          <p className="mt-2 text-[11px] text-neutral-500 dark:text-neutral-400">
            {t('accountRotationHint')}
          </p>
          <div className="mt-1.5">
            <button
              type="button"
              className="text-[11px] text-neutral-400 underline-offset-2 transition-colors hover:text-neutral-600 hover:underline dark:text-neutral-500 dark:hover:text-neutral-300"
              onClick={() => switchSource('manual')}
            >
              {t('accountManualToggle')}
            </button>
          </div>
        </div>
      ) : (
        <>
          {accountSourceEnabled && (
            <div className="mt-2">
              <button
                type="button"
                className="text-[11px] font-medium text-blue-600 hover:underline dark:text-blue-400"
                onClick={() => switchSource('account')}
              >
                {t('accountManualBack')}
              </button>
            </div>
          )}
          {isEditing && credentialConfigured && masked && !replaceExisting ? (
            <div className="mt-3 rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2.5 dark:border-emerald-900 dark:bg-emerald-950/30">
              <div className="flex items-center gap-1.5 text-xs font-medium text-emerald-800 dark:text-emerald-200">
                <CheckCircle2 aria-hidden="true" size={13} />
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
        </>
      )}

      {accountSourceEnabled && (
        <StorageProviderAccountPickerDialog
          open={pickerOpen}
          accounts={providerAccounts}
          loading={accountsLoading}
          error={accountsError}
          selectedAccountId={providerAccountId.trim()}
          onSelectedAccountChange={onProviderAccountIdChange}
          onClose={() => setPickerOpen(false)}
          onReloadAccounts={onReloadProviderAccounts}
          onCreateProviderAccount={onCreateProviderAccount}
          allowedVendorCodes={allowedVendorCodes}
          credentialFields={credentialFields}
          defaultVendorCode={defaultVendorCode}
        />
      )}
    </div>
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
  label?: string;
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
            {showSecret ? <EyeOff aria-hidden="true" size={16} /> : <Eye aria-hidden="true" size={16} />}
          </button>
        )}
      </div>
    </label>
  );
}
