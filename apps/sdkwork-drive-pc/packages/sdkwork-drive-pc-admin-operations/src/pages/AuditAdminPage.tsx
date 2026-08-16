import React, { useEffect, useMemo, useState, type FormEvent } from 'react';
import { RefreshCw, RotateCcw, Search, ScrollText, SlidersHorizontal } from 'lucide-react';
import type { DriveBackendSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import { EmptyState, LoadingState, NoticeBanner, OperationsPageHeader, PaginationBar } from '../components/OperationsAdminPrimitives';
import { createDriveOperationsAdminService } from '../services/driveOperationsAdminService';
import type { AuditEventView } from '../types/driveOperationsAdminTypes';
import {
  CARD_CLASS,
  CARD_HEADER_CLASS,
  INPUT_CLASS,
  PRIMARY_BUTTON_CLASS,
  SECONDARY_BUTTON_CLASS,
  TABLE_CLASS,
  TABLE_HEAD_CLASS,
  TABLE_ROW_CLASS,
} from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

interface AuditAdminPageProps {
  backendSdkClient: DriveBackendSdkClient;
  getSession: () => SessionSnapshot;
}

type AuditFilters = {
  action: string;
  resourceId: string;
  resourceType: string;
};

const EMPTY_FILTERS: AuditFilters = { action: '', resourceId: '', resourceType: '' };

function formatTimestamp(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

export function AuditAdminPage({ backendSdkClient, getSession }: AuditAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo(
    () => createDriveOperationsAdminService({ backendSdkClient, getSession }),
    [backendSdkClient, getSession],
  );
  const [items, setItems] = useState<AuditEventView[]>([]);
  const [page, setPage] = useState(1);
  const [pageSize] = useState(25);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | undefined>();
  const [refreshKey, setRefreshKey] = useState(0);
  const [draftFilters, setDraftFilters] = useState<AuditFilters>(EMPTY_FILTERS);
  const [appliedFilters, setAppliedFilters] = useState<AuditFilters>(EMPTY_FILTERS);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError(undefined);
    service.listAuditEvents({
      action: appliedFilters.action.trim() || undefined,
      resourceType: appliedFilters.resourceType.trim() || undefined,
      resourceId: appliedFilters.resourceId.trim() || undefined,
      page,
      pageSize,
      signal: controller.signal,
    })
      .then((result) => {
        setItems(result.items);
        setTotal(result.total);
      })
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setError(t('noticeLoadFailed'));
      })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [appliedFilters, page, pageSize, refreshKey, service, t]);

  const applyFilters = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setPage(1);
    setAppliedFilters({ ...draftFilters });
  };

  const resetFilters = () => {
    setDraftFilters(EMPTY_FILTERS);
    setAppliedFilters(EMPTY_FILTERS);
    setPage(1);
  };

  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const filtersActive = Object.values(appliedFilters).some((value) => value.trim() !== '');

  return (
    <main className="flex h-full min-h-0 w-full min-w-0 flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <OperationsPageHeader
        icon={ScrollText}
        title={t('auditPageTitle')}
        description={t('auditPageDescription')}
        actions={(
          <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading} onClick={() => setRefreshKey((current) => current + 1)}>
            <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
            {t('refresh')}
          </button>
        )}
      />

      <div className="flex-1 space-y-4 overflow-auto p-4 sm:p-6">
        {error ? <NoticeBanner type="error" message={error} dismissLabel={t('dismiss')} onDismiss={() => setError(undefined)} /> : null}

        <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="audit-filters-title">
          <div className={`${CARD_HEADER_CLASS} flex items-center gap-2`}>
            <SlidersHorizontal aria-hidden="true" className="text-blue-600 dark:text-blue-400" size={16} />
            <h2 id="audit-filters-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">{t('filtersTitle')}</h2>
          </div>
          <form className="grid gap-4 p-5 lg:grid-cols-[1fr_1fr_1.2fr_auto] lg:items-end" onSubmit={applyFilters}>
            <label className="flex min-w-0 flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">
              {t('filterAction')}
              <input className={INPUT_CLASS} value={draftFilters.action} onChange={(event) => setDraftFilters((current) => ({ ...current, action: event.target.value }))} placeholder={t('filterActionPlaceholder')} />
            </label>
            <label className="flex min-w-0 flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">
              {t('filterResourceType')}
              <input className={INPUT_CLASS} value={draftFilters.resourceType} onChange={(event) => setDraftFilters((current) => ({ ...current, resourceType: event.target.value }))} placeholder={t('filterResourceTypePlaceholder')} />
            </label>
            <label className="flex min-w-0 flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">
              {t('filterResourceId')}
              <input className={`${INPUT_CLASS} font-mono text-xs`} value={draftFilters.resourceId} onChange={(event) => setDraftFilters((current) => ({ ...current, resourceId: event.target.value }))} placeholder={t('filterResourceIdPlaceholder')} />
            </label>
            <div className="flex items-center gap-2">
              <button type="submit" className={PRIMARY_BUTTON_CLASS}>
                <Search aria-hidden="true" size={15} />
                {t('applyFilters')}
              </button>
              <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={!filtersActive && Object.values(draftFilters).every((value) => value === '')} onClick={resetFilters}>
                <RotateCcw aria-hidden="true" size={15} />
                {t('resetFilters')}
              </button>
            </div>
          </form>
        </section>

        <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="audit-events-title">
          <div className={`${CARD_HEADER_CLASS} flex flex-wrap items-center justify-between gap-3`}>
            <div>
              <h2 id="audit-events-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">{t('auditEventsTitle')}</h2>
              <p className="mt-0.5 text-xs text-neutral-500 dark:text-neutral-400">{t('countOf', { filtered: items.length, total })}</p>
            </div>
            <span className="rounded-full bg-neutral-100 px-2.5 py-1 text-xs font-medium text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300">{t('pageOf', { page, totalPages })}</span>
          </div>
          {loading ? <LoadingState label={t('loading')} /> : items.length === 0 ? (
            <EmptyState title={t('auditEmpty')} description={t('auditPageDescription')} icon={ScrollText} />
          ) : (
            <div className="overflow-x-auto">
              <table className={`${TABLE_CLASS} min-w-[1080px]`}>
                <thead><tr className={TABLE_HEAD_CLASS}>
                  <th className="px-5 py-3">{t('colTime')}</th>
                  <th className="px-5 py-3">{t('colAction')}</th>
                  <th className="px-5 py-3">{t('colResourceType')}</th>
                  <th className="px-5 py-3">{t('colResourceId')}</th>
                  <th className="px-5 py-3">{t('colOperator')}</th>
                  <th className="px-5 py-3">{t('colTrace')}</th>
                </tr></thead>
                <tbody>
                  {items.map((item) => (
                    <tr key={item.id} className={TABLE_ROW_CLASS}>
                      <td className="whitespace-nowrap px-5 py-3 text-xs text-neutral-600 dark:text-neutral-300">{formatTimestamp(item.createdAt)}</td>
                      <td className="px-5 py-3"><code className="rounded bg-blue-50 px-1.5 py-1 text-xs font-medium text-blue-700 dark:bg-blue-950/40 dark:text-blue-300">{item.action}</code></td>
                      <td className="px-5 py-3 text-xs font-medium text-neutral-700 dark:text-neutral-200">{item.resourceType}</td>
                      <td className="max-w-64 truncate px-5 py-3 font-mono text-xs text-neutral-600 dark:text-neutral-300" title={item.resourceId}>{item.resourceId}</td>
                      <td className="px-5 py-3 font-mono text-xs text-neutral-600 dark:text-neutral-300">{item.operatorId}</td>
                      <td className="max-w-64 truncate px-5 py-3 font-mono text-xs text-neutral-500" title={item.traceId || item.correlationId}>{item.traceId || item.correlationId || '--'}</td>
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
