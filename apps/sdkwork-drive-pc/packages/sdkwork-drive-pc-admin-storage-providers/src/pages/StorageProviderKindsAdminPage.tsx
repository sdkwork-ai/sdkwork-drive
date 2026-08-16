import React, { useEffect, useMemo, useState } from 'react';
import {
  CheckCircle2,
  CircleAlert,
  LoaderCircle,
  Power,
  PowerOff,
  RefreshCw,
  RotateCcw,
  X,
} from 'lucide-react';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import {
  createStorageProviderAdminService,
  type StorageProviderAdminService,
} from '../services/storageProviderAdminService';
import type { StorageProviderKindView } from '../types/storageProviderAdminTypes';
import { getProviderKindMeta } from '../utils/providerKindConfig';
import { ConfirmDialog } from '../components/ConfirmDialog';
import {
  PRIMARY_BUTTON_CLASS,
  SECONDARY_BUTTON_CLASS,
  BADGE_BASE_CLASS,
  ICON_BUTTON_CLASS,
  CARD_CLASS,
} from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

interface StorageProviderKindsAdminPageProps {
  adminStorageSdkClient: DriveAdminStorageSdkClient;
  getSession: () => SessionSnapshot;
}

type PageNotice = { type: 'success' | 'error'; messageKey: string; params?: Record<string, string> } | undefined;

export function StorageProviderKindsAdminPage({
  adminStorageSdkClient,
  getSession,
}: StorageProviderKindsAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo<StorageProviderAdminService>(
    () => createStorageProviderAdminService({ adminStorageSdkClient, getSession }),
    [adminStorageSdkClient, getSession],
  );
  const [kinds, setKinds] = useState<StorageProviderKindView[]>([]);
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [notice, setNotice] = useState<PageNotice>();
  const [disableTarget, setDisableTarget] = useState<StorageProviderKindView | null>(null);

  const reload = (signal?: AbortSignal) => {
    setLoading(true);
    setNotice(undefined);
    service
      .listKinds({ signal })
      .then(setKinds)
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', messageKey: 'kindsNoticeLoadFailed' });
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    const c = new AbortController();
    reload(c.signal);
    return () => c.abort();
  }, [service]);

  const enabledCount = kinds.filter((kind) => kind.enabled).length;

  const toggleKind = async (kind: StorageProviderKindView, enabled: boolean) => {
    setPending(true);
    setNotice(undefined);
    setDisableTarget(null);
    try {
      await service.setKindEnabled(kind.providerKind, enabled);
      setNotice({
        type: 'success',
        messageKey: enabled ? 'kindsNoticeEnabled' : 'kindsNoticeDisabled',
        params: { label: kind.displayName },
      });
      reload();
    } catch (err) {
      if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', messageKey: 'kindsNoticeToggleFailed' });
    } finally {
      setPending(false);
    }
  };

  const initializeKinds = () => {
    setPending(true);
    setNotice(undefined);
    service
      .initializeKinds()
      .then((items) => {
        setKinds(items);
        setNotice({ type: 'success', messageKey: 'kindsNoticeInitialized', params: { count: String(items.length) } });
      })
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', messageKey: 'kindsNoticeInitializeFailed' });
      })
      .finally(() => setPending(false));
  };

  return (
    <main className="flex h-full flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <div aria-label={t('kindsPageTitle')} className="px-4 pt-4 sm:px-6 sm:pt-6">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="flex flex-wrap items-center gap-2">
            <span className={`${BADGE_BASE_CLASS} bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300`}>{t('kindsSummary', { total: kinds.length, enabled: enabledCount })}</span>
          </div>
          <div className="flex w-full shrink-0 items-center justify-end gap-2 sm:!w-auto">
            <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading || pending} onClick={() => reload()}>
              <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
              {t('refresh')}
            </button>
            <button type="button" className={PRIMARY_BUTTON_CLASS} disabled={pending} onClick={initializeKinds}>
              <RotateCcw aria-hidden="true" size={16} />
              {t('kindsInitialize')}
            </button>
          </div>
        </div>

        {notice && (
          <div className={`mt-4 flex items-center gap-3 rounded-lg border px-4 py-3 text-sm ${
            notice.type === 'success'
              ? 'border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950/30 dark:text-emerald-200'
              : 'border-red-200 bg-red-50 text-red-800 dark:border-red-900 dark:bg-red-950/30 dark:text-red-200'
          }`}>
            {notice.type === 'success' ? <CheckCircle2 aria-hidden="true" className="shrink-0" size={16} /> : <CircleAlert aria-hidden="true" className="shrink-0" size={16} />}
            <span className="flex-1">{t(notice.messageKey, notice.params)}</span>
            <button type="button" className={ICON_BUTTON_CLASS} aria-label={t('dismiss')} title={t('dismiss')} onClick={() => setNotice(undefined)}><X aria-hidden="true" size={15} /></button>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-auto p-4 sm:p-6">
        {loading ? (
          <div className="flex min-h-[360px] items-center justify-center rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900">
            <div className="flex items-center gap-3 text-sm text-neutral-500">
              <LoaderCircle aria-hidden="true" className="animate-spin" size={19} />
              {t('kindsLoading')}
            </div>
          </div>
        ) : (
          <div className={`${CARD_CLASS} overflow-x-auto`}>
            <table className="w-full min-w-[720px] text-left text-sm">
              <thead className="border-b border-neutral-200 bg-neutral-50 text-xs font-medium text-neutral-500 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-400">
                <tr>
                  <th className="px-5 py-3 font-semibold">{t('kindsColProvider')}</th>
                  <th className="px-5 py-3 font-semibold">{t('kindsColKind')}</th>
                  <th className="px-5 py-3 font-semibold">{t('kindsColConfigs')}</th>
                  <th className="px-5 py-3 font-semibold">{t('kindsColStatus')}</th>
                  <th className="px-5 py-3 text-right font-semibold">{t('kindsColAction')}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-100 dark:divide-neutral-800">
                {kinds.map((kind) => {
                  const meta = getProviderKindMeta(kind.providerKind);
                  return (
                    <tr key={kind.providerKind} className="hover:bg-neutral-50 dark:hover:bg-neutral-800/50">
                      <td className="px-5 py-3">
                        <div className="flex items-center gap-2.5">
                          <div className={`flex h-9 w-9 items-center justify-center rounded-lg ${meta.bgClass} ${meta.textClass}`}>
                            <span className="text-[11px] font-bold">{meta.icon}</span>
                          </div>
                          <span className="text-sm font-semibold text-neutral-900 dark:text-neutral-100">{kind.displayName}</span>
                        </div>
                      </td>
                      <td className="px-5 py-3">
                        <span className="font-mono text-xs text-neutral-500">{kind.providerKind}</span>
                      </td>
                      <td className="px-5 py-3">
                        <span className="text-xs text-neutral-700 dark:text-neutral-300">{kind.configCount}</span>
                      </td>
                      <td className="px-5 py-3">
                        {kind.enabled ? (
                          <span className={`${BADGE_BASE_CLASS} bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300`}>
                            <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                            {t('kindsEnabled')}
                          </span>
                        ) : (
                          <span className={`${BADGE_BASE_CLASS} bg-neutral-100 text-neutral-500 dark:bg-neutral-800 dark:text-neutral-400`}>
                            <span className="h-1.5 w-1.5 rounded-full bg-neutral-400" />
                            {t('kindsDisabled')}
                          </span>
                        )}
                      </td>
                      <td className="px-5 py-3 text-right">
                        {kind.enabled ? (
                          <button
                            type="button"
                            className="rounded-md px-2.5 py-1 text-xs font-medium text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950/30"
                            disabled={pending}
                            onClick={() => setDisableTarget(kind)}
                          >
                            <span className="inline-flex items-center gap-1"><PowerOff aria-hidden="true" size={12} />{t('kindsDisable')}</span>
                          </button>
                        ) : (
                          <button
                            type="button"
                            className="rounded-md px-2.5 py-1 text-xs font-medium text-emerald-600 hover:bg-emerald-50 dark:text-emerald-400 dark:hover:bg-emerald-950/30"
                            disabled={pending}
                            onClick={() => void toggleKind(kind, true)}
                          >
                            <span className="inline-flex items-center gap-1"><Power aria-hidden="true" size={12} />{t('kindsEnable')}</span>
                          </button>
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <ConfirmDialog
        open={disableTarget !== null}
        title={t('kindsDisableTitle')}
        message={t('kindsDisableMessage', { label: disableTarget?.displayName ?? '' })}
        confirmLabel={t('kindsDisable')}
        variant="danger"
        busy={pending}
        onCancel={() => setDisableTarget(null)}
        onConfirm={() => {
          if (disableTarget) void toggleKind(disableTarget, false);
        }}
      />
    </main>
  );
}
