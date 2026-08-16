import React, { useEffect, useMemo, useState } from 'react';
import { ExternalLink } from 'lucide-react';
import type { ProviderCredentialFieldMeta } from '../utils/providerKindConfig';
import {
  buildCredentialRef,
  isCredentialRefMasked,
  parseCredentialRef,
  type CredentialInputMode,
} from '../utils/credentialRefUtils';
import { INPUT_CLASS } from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

interface StorageProviderCredentialFieldsProps {
  credentialFields: ProviderCredentialFieldMeta;
  credentialRef: string;
  isEditing: boolean;
  credentialConfigured?: boolean;
  error?: string;
  onCredentialRefChange: (value: string) => void;
}

export function StorageProviderCredentialFields({
  credentialFields,
  credentialRef,
  isEditing,
  credentialConfigured,
  error,
  onCredentialRefChange,
}: StorageProviderCredentialFieldsProps) {
  const { t } = useTranslation();
  const parsed = useMemo(() => parseCredentialRef(credentialRef), [credentialRef]);
  const masked = isCredentialRefMasked(credentialRef);

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
    if (nextRef !== credentialRef) {
      onCredentialRefChange(nextRef);
    }
  }, [
    mode,
    accessKey,
    secretKey,
    accessKeyEnv,
    secretKeyEnv,
    secretRef,
    isEditing,
    credentialConfigured,
    replaceExisting,
    credentialRef,
    onCredentialRefChange,
  ]);

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

      {isEditing && credentialConfigured && masked && !replaceExisting ? (
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

      {error && <p className="mt-2 text-[11px] text-red-500">{error}</p>}
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
            className="absolute right-2 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600"
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
