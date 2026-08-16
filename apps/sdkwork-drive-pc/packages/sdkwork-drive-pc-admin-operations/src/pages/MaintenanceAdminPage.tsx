import React, { useEffect, useMemo, useState } from 'react';
import {
  Filter,
  History,
  LoaderCircle,
  Play,
  RefreshCw,
  ShieldCheck,
  Wrench,
} from 'lucide-react';
import type { DriveBackendSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import { OperationsConfirmDialog } from '../components/OperationsConfirmDialog';
import { EmptyState, LoadingState, NoticeBanner, OperationsPageHeader, PaginationBar } from '../components/OperationsAdminPrimitives';
import { createDriveOperationsAdminService } from '../services/driveOperationsAdminService';
import type { MaintenanceJobType, MaintenanceJobView } from '../types/driveOperationsAdminTypes';
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

interface MaintenanceAdminPageProps {
  backendSdkClient: DriveBackendSdkClient;
  getSession: () => SessionSnapshot;
}

const SWEEP_JOB_TYPES: MaintenanceJobType[] = [
  'object_sweep',
  'upload_session_sweep',
  'expired_upload_content_sweep',
  'abandoned_upload_task_sweep',
];

function formatTimestamp(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

export function MaintenanceAdminPage({ backendSdkClient, getSession }: MaintenanceAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo(
    () => createDriveOperationsAdminService({ backendSdkClient, getSession }),
    [backendSdkClient, getSession],
  );
  const [jobs, setJobs] = useState<MaintenanceJobView[]>([]);
  const [page, setPage] = useState(1);
  const [pageSize] = useState(20);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [pendingJobType, setPendingJobType] = useState<MaintenanceJobType | null>(null);
  const [confirmJobType, setConfirmJobType] = useState<MaintenanceJobType | null>(null);
  const [dryRun, setDryRun] = useState(true);
  const [notice, setNotice] = useState<{ type: 'success' | 'error'; message: string } | undefined>();
  const [jobTypeFilter, setJobTypeFilter] = useState<MaintenanceJobType | ''>('');
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    service.listMaintenanceJobs({
      jobType: jobTypeFilter || undefined,
      page,
      pageSize,
      signal: controller.signal,
    })
      .then((result) => {
        setJobs(result.items);
        setTotal(result.total);
      })
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', message: t('noticeLoadFailed') });
      })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [jobTypeFilter, page, pageSize, refreshKey, service, t]);

  const runSweep = async (jobType: MaintenanceJobType) => {
    setPendingJobType(jobType);
    setConfirmJobType(null);
    setNotice(undefined);
    try {
      const result = await service.startMaintenanceSweep({ jobType, dryRun });
      setNotice({
        type: 'success',
        message: t('sweepSuccess', {
          scanned: result.scannedCount,
          affected: result.affectedCount,
          mode: dryRun ? t('dryRunLabel') : t('liveRunLabel'),
        }),
      });
      setRefreshKey((current) => current + 1);
    } catch (err) {
      if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', message: t('noticeOperationFailed') });
    } finally {
      setPendingJobType(null);
    }
  };

  const requestSweep = (jobType: MaintenanceJobType) => {
    if (dryRun) {
      void runSweep(jobType);
      return;
    }
    setConfirmJobType(jobType);
  };

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  return (
    <main className="flex h-full min-h-0 w-full min-w-0 flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <OperationsPageHeader
        icon={Wrench}
        title={t('maintenancePageTitle')}
        description={t('maintenancePageDescription')}
        toneClassName="bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-300"
        actions={(
          <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading} onClick={() => setRefreshKey((current) => current + 1)}>
            <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
            {t('refresh')}
          </button>
        )}
      />

      <div className="flex-1 space-y-4 overflow-auto p-4 sm:p-6">
        {notice ? <NoticeBanner type={notice.type} message={notice.message} dismissLabel={t('dismiss')} onDismiss={() => setNotice(undefined)} /> : null}

        <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="maintenance-run-title">
          <div className={`${CARD_HEADER_CLASS} flex flex-wrap items-center justify-between gap-3`}>
            <div className="flex items-center gap-2">
              <ShieldCheck aria-hidden="true" className="text-amber-600 dark:text-amber-400" size={17} />
              <h2 id="maintenance-run-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">{t('runSweepTitle')}</h2>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={dryRun}
              className="inline-flex h-8 items-center gap-2 rounded-md border border-neutral-200 bg-neutral-50 px-2.5 text-xs font-medium text-neutral-700 transition-colors hover:bg-neutral-100 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-200 dark:hover:bg-neutral-700"
              onClick={() => setDryRun((current) => !current)}
            >
              <span className={`relative h-4 w-7 rounded-full transition-colors ${dryRun ? 'bg-blue-600' : 'bg-amber-500'}`}>
                <span className={`absolute top-0.5 h-3 w-3 rounded-full bg-white shadow transition-transform ${dryRun ? 'translate-x-3.5' : 'translate-x-0.5'}`} />
              </span>
              {dryRun ? t('dryRunLabel') : t('liveRunLabel')}
            </button>
          </div>
          <div className="p-5">
            <p className="mb-4 text-xs leading-5 text-neutral-500 dark:text-neutral-400">{t('dryRunHint')}</p>
            <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
              {SWEEP_JOB_TYPES.map((jobType) => {
                const pending = pendingJobType === jobType;
                return (
                  <button
                    key={jobType}
                    type="button"
                    className="group flex min-h-20 items-center gap-3 rounded-lg border border-neutral-200 bg-white p-4 text-left transition-colors hover:border-blue-300 hover:bg-blue-50/50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-700 dark:bg-neutral-900 dark:hover:border-blue-800 dark:hover:bg-blue-950/20"
                    disabled={pendingJobType !== null}
                    onClick={() => requestSweep(jobType)}
                  >
                    <span className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-lg ${dryRun ? 'bg-blue-50 text-blue-600 dark:bg-blue-950/40 dark:text-blue-300' : 'bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-300'}`}>
                      {pending ? <LoaderCircle aria-hidden="true" className="animate-spin" size={17} /> : <Play aria-hidden="true" size={17} />}
                    </span>
                    <span className="min-w-0 text-sm font-medium text-neutral-800 group-hover:text-blue-700 dark:text-neutral-100 dark:group-hover:text-blue-300">{t(`jobType.${jobType}`)}</span>
                  </button>
                );
              })}
            </div>
          </div>
        </section>

        <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="maintenance-history-title">
          <div className={`${CARD_HEADER_CLASS} flex flex-wrap items-center justify-between gap-3`}>
            <div className="flex items-center gap-2">
              <History aria-hidden="true" className="text-blue-600 dark:text-blue-400" size={17} />
              <div>
                <h2 id="maintenance-history-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">{t('jobHistoryTitle')}</h2>
                <p className="mt-0.5 text-xs text-neutral-500 dark:text-neutral-400">{t('countOf', { filtered: jobs.length, total })}</p>
              </div>
            </div>
            <label className="flex items-center gap-2 text-xs font-medium text-neutral-500 dark:text-neutral-400">
              <Filter aria-hidden="true" size={14} />
              <select
                className={SELECT_CLASS}
                value={jobTypeFilter}
                onChange={(event) => {
                  setJobTypeFilter(event.target.value as MaintenanceJobType | '');
                  setPage(1);
                }}
              >
                <option value="">{t('allJobTypes')}</option>
                {SWEEP_JOB_TYPES.map((jobType) => <option key={jobType} value={jobType}>{t(`jobType.${jobType}`)}</option>)}
              </select>
            </label>
          </div>
          {loading ? <LoadingState label={t('loading')} /> : jobs.length === 0 ? (
            <EmptyState title={t('jobsEmpty')} description={t('maintenancePageDescription')} icon={History} />
          ) : (
            <div className="overflow-x-auto">
              <table className={`${TABLE_CLASS} min-w-[880px]`}>
                <thead><tr className={TABLE_HEAD_CLASS}>
                  <th className="px-5 py-3">{t('colStarted')}</th>
                  <th className="px-5 py-3">{t('colJobType')}</th>
                  <th className="px-5 py-3">{t('colStatus')}</th>
                  <th className="px-5 py-3 text-right">{t('colScanned')}</th>
                  <th className="px-5 py-3 text-right">{t('colAffected')}</th>
                  <th className="px-5 py-3">{t('colOperator')}</th>
                </tr></thead>
                <tbody>
                  {jobs.map((job) => (
                    <tr key={job.id} className={TABLE_ROW_CLASS}>
                      <td className="whitespace-nowrap px-5 py-3 text-xs text-neutral-600 dark:text-neutral-300">{formatTimestamp(job.startedAt)}</td>
                      <td className="px-5 py-3 text-sm font-medium text-neutral-900 dark:text-neutral-100">{t(`jobType.${job.jobType}`)}</td>
                      <td className="px-5 py-3">
                        <div className="flex flex-wrap items-center gap-2">
                          <span className={`${BADGE_BASE_CLASS} ${job.status === 'completed' ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/50 dark:text-emerald-300' : 'bg-red-100 text-red-700 dark:bg-red-950/50 dark:text-red-300'}`}>{t(`status.${job.status}`)}</span>
                          <span className="text-xs text-neutral-500 dark:text-neutral-400">{job.dryRun ? t('dryRunLabel') : t('liveRunLabel')}</span>
                        </div>
                      </td>
                      <td className="px-5 py-3 text-right font-mono text-xs text-neutral-700 dark:text-neutral-200">{job.scannedCount}</td>
                      <td className="px-5 py-3 text-right font-mono text-xs text-neutral-700 dark:text-neutral-200">{job.affectedCount}</td>
                      <td className="px-5 py-3 font-mono text-xs text-neutral-600 dark:text-neutral-300">{job.operatorId}</td>
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

      <OperationsConfirmDialog
        open={confirmJobType !== null}
        title={t('confirmLiveSweepTitle')}
        message={t('confirmLiveSweepMessage', { jobType: confirmJobType ? t(`jobType.${confirmJobType}`) : '' })}
        confirmLabel={t('confirmRunAction')}
        cancelLabel={t('cancel')}
        variant="danger"
        onCancel={() => setConfirmJobType(null)}
        onConfirm={() => { if (confirmJobType) void runSweep(confirmJobType); }}
      />
    </main>
  );
}
