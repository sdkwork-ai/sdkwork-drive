import React, { useEffect, useMemo, useState } from 'react';
import { Archive, Filter, RefreshCw } from 'lucide-react';
import type { DriveBackendSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import { formatDriveBytes } from 'sdkwork-drive-pc-commons';
import { EmptyState, LoadingState, NoticeBanner, OperationsPageHeader, PaginationBar } from '../components/OperationsAdminPrimitives';
import { createDriveOperationsAdminService } from '../services/driveOperationsAdminService';
import type { DownloadPackageView } from '../types/driveOperationsAdminTypes';
import {
  BADGE_BASE_CLASS,
  CARD_CLASS,
  CARD_HEADER_CLASS,
  SELECT_CLASS,
  SECONDARY_BUTTON_CLASS,
  TABLE_CLASS,
  TABLE_HEAD_CLASS,
  TABLE_ROW_CLASS,
} from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

interface DownloadPackagesAdminPageProps {
  backendSdkClient: DriveBackendSdkClient;
  getSession: () => SessionSnapshot;
}

const PACKAGE_STATES: DownloadPackageView['state'][] = ['creating', 'ready', 'failed', 'expired'];

const STATE_TONES: Record<DownloadPackageView['state'], string> = {
  creating: 'bg-blue-100 text-blue-700 dark:bg-blue-950/50 dark:text-blue-300',
  ready: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/50 dark:text-emerald-300',
  failed: 'bg-red-100 text-red-700 dark:bg-red-950/50 dark:text-red-300',
  expired: 'bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300',
};

function formatTimestamp(value: string | number): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? String(value) : date.toLocaleString();
}

export function DownloadPackagesAdminPage({ backendSdkClient, getSession }: DownloadPackagesAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo(
    () => createDriveOperationsAdminService({ backendSdkClient, getSession }),
    [backendSdkClient, getSession],
  );
  const [items, setItems] = useState<DownloadPackageView[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | undefined>();
  const [stateFilter, setStateFilter] = useState<DownloadPackageView['state'] | ''>('');
  const [page, setPage] = useState(1);
  const [pageSize] = useState(25);
  const [total, setTotal] = useState(0);
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError(undefined);
    service.listDownloadPackages({ state: stateFilter || undefined, page, pageSize, signal: controller.signal })
      .then((result) => {
        setItems(result.items);
        setTotal(result.total);
      })
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setError(t('noticeLoadFailed'));
      })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [page, pageSize, refreshKey, service, stateFilter, t]);

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  return (
    <main className="flex h-full min-h-0 w-full min-w-0 flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <OperationsPageHeader
        icon={Archive}
        title={t('downloadPackagesPageTitle')}
        description={t('downloadPackagesPageDescription')}
        toneClassName="bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300"
        actions={(
          <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading} onClick={() => setRefreshKey((current) => current + 1)}>
            <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
            {t('refresh')}
          </button>
        )}
      />

      <div className="flex-1 space-y-4 overflow-auto p-4 sm:p-6">
        {error ? <NoticeBanner type="error" message={error} dismissLabel={t('dismiss')} onDismiss={() => setError(undefined)} /> : null}

        <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="download-packages-title">
          <div className={`${CARD_HEADER_CLASS} flex flex-wrap items-center justify-between gap-3`}>
            <div>
              <h2 id="download-packages-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">{t('downloadPackagesListTitle')}</h2>
              <p className="mt-0.5 text-xs text-neutral-500 dark:text-neutral-400">{t('countOf', { filtered: items.length, total })}</p>
            </div>
            <label className="flex items-center gap-2 text-xs font-medium text-neutral-500 dark:text-neutral-400">
              <Filter aria-hidden="true" size={14} />
              <select className={SELECT_CLASS} value={stateFilter} onChange={(event) => { setStateFilter(event.target.value as DownloadPackageView['state'] | ''); setPage(1); }}>
                <option value="">{t('allStates')}</option>
                {PACKAGE_STATES.map((state) => <option key={state} value={state}>{t(`downloadState.${state}`)}</option>)}
              </select>
            </label>
          </div>
          {loading ? <LoadingState label={t('loading')} /> : items.length === 0 ? (
            <EmptyState title={t('downloadPackagesEmpty')} description={t('downloadPackagesPageDescription')} icon={Archive} />
          ) : (
            <div className="overflow-x-auto">
              <table className={`${TABLE_CLASS} min-w-[980px]`}>
                <thead><tr className={TABLE_HEAD_CLASS}>
                  <th className="px-5 py-3">{t('colPackageName')}</th>
                  <th className="px-5 py-3">{t('colStatus')}</th>
                  <th className="px-5 py-3">{t('colProvider')}</th>
                  <th className="px-5 py-3 text-right">{t('colFiles')}</th>
                  <th className="px-5 py-3 text-right">{t('colSize')}</th>
                  <th className="px-5 py-3">{t('colCreated')}</th>
                  <th className="px-5 py-3">{t('colExpires')}</th>
                </tr></thead>
                <tbody>
                  {items.map((item) => (
                    <tr key={item.id} className={TABLE_ROW_CLASS}>
                      <td className="px-5 py-3"><div className="text-sm font-medium text-neutral-900 dark:text-neutral-100">{item.packageName || item.id}</div><div className="mt-0.5 font-mono text-[10px] text-neutral-400">{item.id}</div></td>
                      <td className="px-5 py-3"><span className={`${BADGE_BASE_CLASS} ${STATE_TONES[item.state]}`}>{t(`downloadState.${item.state}`)}</span>{item.errorMessage ? <div className="mt-1 max-w-56 truncate text-[10px] text-red-600 dark:text-red-400" title={item.errorMessage}>{item.errorMessage}</div> : null}</td>
                      <td className="px-5 py-3"><div className="font-mono text-xs text-neutral-700 dark:text-neutral-200">{item.storageProviderId}</div><div className="mt-0.5 text-[10px] text-neutral-400">{item.bucket}</div></td>
                      <td className="px-5 py-3 text-right font-mono text-xs tabular-nums text-neutral-700 dark:text-neutral-200">{item.fileCount.toLocaleString()}</td>
                      <td className="px-5 py-3 text-right text-xs font-medium text-neutral-700 dark:text-neutral-200">{formatDriveBytes(item.totalBytes)}</td>
                      <td className="whitespace-nowrap px-5 py-3 text-xs text-neutral-600 dark:text-neutral-300">{formatTimestamp(item.createdAt)}</td>
                      <td className="whitespace-nowrap px-5 py-3 text-xs text-neutral-600 dark:text-neutral-300">{formatTimestamp(item.expiresAtEpochMs)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          <PaginationBar
            loading={loading}
            previousDisabled={page <= 1}
            nextDisabled={page >= totalPages}
            previousLabel={t('previousPage')}
            nextLabel={t('nextPage')}
            pageLabel={t('pageOf', { page, totalPages })}
            onPrevious={() => setPage((current) => Math.max(1, current - 1))}
            onNext={() => setPage((current) => current + 1)}
          />
        </section>
      </div>
    </main>
  );
}
