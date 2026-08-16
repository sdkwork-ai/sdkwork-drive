import React, { useEffect, useMemo, useState } from 'react';
import { Database, Files, Gauge, HardDrive, RefreshCw, Save, Trash2 } from 'lucide-react';
import {
  Drawer,
  DrawerBody,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
} from '@sdkwork/ui-pc-react';
import type { DriveBackendSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import { formatDriveBytes } from 'sdkwork-drive-pc-commons';
import { OperationsConfirmDialog } from '../components/OperationsConfirmDialog';
import { EmptyState, LoadingState, NoticeBanner, OperationsPageHeader } from '../components/OperationsAdminPrimitives';
import { createDriveOperationsAdminService } from '../services/driveOperationsAdminService';
import type { QuotaSummaryView } from '../types/driveOperationsAdminTypes';
import {
  CARD_CLASS,
  INPUT_CLASS,
  PRIMARY_BUTTON_CLASS,
  SECONDARY_BUTTON_CLASS,
} from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

interface QuotaAdminPageProps {
  backendSdkClient: DriveBackendSdkClient;
  getSession: () => SessionSnapshot;
}

function usagePercent(used: number, cap?: number | null): number | undefined {
  if (cap === undefined || cap === null || cap <= 0) return undefined;
  return Math.min(100, Math.round((used / cap) * 100));
}

function metricTone(percent?: number): string {
  if (percent === undefined || percent < 70) return 'bg-blue-600';
  if (percent < 90) return 'bg-amber-500';
  return 'bg-red-600';
}

export function QuotaAdminPage({ backendSdkClient, getSession }: QuotaAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo(
    () => createDriveOperationsAdminService({ backendSdkClient, getSession }),
    [backendSdkClient, getSession],
  );
  const [summary, setSummary] = useState<QuotaSummaryView | undefined>();
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [notice, setNotice] = useState<{ type: 'error' | 'success'; message: string } | undefined>();
  const [quotaInput, setQuotaInput] = useState('');
  const [refreshKey, setRefreshKey] = useState(0);
  const [policyEditorOpen, setPolicyEditorOpen] = useState(false);
  const [clearConfirmationOpen, setClearConfirmationOpen] = useState(false);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setNotice(undefined);
    service.getQuotaSummary(controller.signal)
      .then((result) => {
        setSummary(result);
        setQuotaInput(result.quotaBytes ? String(result.quotaBytes) : '');
      })
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', message: t('noticeLoadFailed') });
      })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [refreshKey, service, t]);

  const savePolicy = async () => {
    const quotaBytes = Number.parseInt(quotaInput, 10);
    if (!Number.isFinite(quotaBytes) || quotaBytes <= 0) {
      setNotice({ type: 'error', message: t('invalidQuotaBytes') });
      return;
    }
    setPending(true);
    setNotice(undefined);
    try {
      const updated = await service.updateQuotaPolicy({ quotaBytes });
      setSummary(updated);
      setQuotaInput(String(updated.quotaBytes ?? quotaBytes));
      setPolicyEditorOpen(false);
      setNotice({ type: 'success', message: t('quotaPolicySaved') });
    } catch (err) {
      if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', message: t('noticeOperationFailed') });
    } finally {
      setPending(false);
    }
  };

  const clearPolicy = async () => {
    setClearConfirmationOpen(false);
    setPending(true);
    setNotice(undefined);
    try {
      const updated = await service.updateQuotaPolicy({ clearTenantPolicy: true });
      setSummary(updated);
      setQuotaInput('');
      setNotice({ type: 'success', message: t('quotaPolicyCleared') });
    } catch (err) {
      if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', message: t('noticeOperationFailed') });
    } finally {
      setPending(false);
    }
  };

  const percent = summary ? usagePercent(summary.totalBytes, summary.quotaBytes) : undefined;
  const parsedQuotaInput = Number.parseInt(quotaInput, 10);
  const quotaPreview = Number.isFinite(parsedQuotaInput) && parsedQuotaInput > 0
    ? formatDriveBytes(parsedQuotaInput)
    : t('quotaUnlimited');

  return (
    <main className="flex h-full min-h-0 w-full min-w-0 flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <OperationsPageHeader
        icon={Gauge}
        title={t('quotaPageTitle')}
        description={t('quotaPageDescription')}
        toneClassName="bg-cyan-50 text-cyan-700 dark:bg-cyan-950/40 dark:text-cyan-300"
        actions={(
          <>
            <button type="button" className={PRIMARY_BUTTON_CLASS} disabled={loading || !summary} onClick={() => setPolicyEditorOpen(true)}>
              <Save aria-hidden="true" size={15} />
              {t('saveQuotaPolicy')}
            </button>
            <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading} onClick={() => setRefreshKey((current) => current + 1)}>
              <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
              {t('refresh')}
            </button>
          </>
        )}
      />

      <div className="flex-1 space-y-4 overflow-auto p-4 sm:p-6">
        {notice ? <NoticeBanner type={notice.type} message={notice.message} dismissLabel={t('dismiss')} onDismiss={() => setNotice(undefined)} /> : null}

        {loading ? (
          <div className={CARD_CLASS}><LoadingState label={t('loading')} /></div>
        ) : summary ? (
          <>
            <section className="grid gap-4 md:grid-cols-3" aria-label={t('quotaUsageTitle')}>
              <QuotaMetric icon={Database} label={t('totalBytesLabel')} value={formatDriveBytes(summary.totalBytes)} detail={summary.tenantId} tone="blue" />
              <QuotaMetric icon={Files} label={t('objectCountLabel')} value={summary.objectCount.toLocaleString()} detail={t('tenantIdLabel')} tone="violet" />
              <QuotaMetric icon={HardDrive} label={t('quotaCapLabel')} value={summary.quotaBytes ? formatDriveBytes(summary.quotaBytes) : t('quotaUnlimited')} detail={percent === undefined ? t('quotaUnlimited') : `${percent}%`} tone="cyan" />
            </section>

            <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="quota-usage-title">
              <div className="flex flex-wrap items-center justify-between gap-3 border-b border-neutral-100 px-5 py-4 dark:border-neutral-800">
                <div>
                  <h2 id="quota-usage-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">{t('usageLabel')}</h2>
                  <p className="mt-0.5 text-xs text-neutral-500 dark:text-neutral-400">{formatDriveBytes(summary.totalBytes)} / {summary.quotaBytes ? formatDriveBytes(summary.quotaBytes) : t('quotaUnlimited')}</p>
                </div>
                <span className="text-sm font-semibold tabular-nums text-neutral-700 dark:text-neutral-200">{percent === undefined ? '--' : `${percent}%`}</span>
              </div>
              <div className="p-5">
                <div className="h-2.5 overflow-hidden rounded-full bg-neutral-200 dark:bg-neutral-800" role="progressbar" aria-label={t('usageLabel')} aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent ?? 0}>
                  <div className={`h-full rounded-full transition-[width] ${metricTone(percent)}`} style={{ width: `${percent ?? 0}%` }} />
                </div>
              </div>
            </section>

            <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="quota-policy-title">
              <div className="border-b border-neutral-100 px-5 py-4 dark:border-neutral-800">
                <h2 id="quota-policy-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">{t('quotaPolicyTitle')}</h2>
                <p className="mt-1 text-xs leading-5 text-neutral-500 dark:text-neutral-400">{t('quotaPolicyHint')}</p>
              </div>
              <div className="flex flex-wrap items-center justify-between gap-4 p-5">
                <div><span className="text-xs text-neutral-500 dark:text-neutral-400">{t('quotaCapLabel')}</span><strong className="mt-1 block text-sm text-neutral-900 dark:text-white">{summary.quotaBytes ? formatDriveBytes(summary.quotaBytes) : t('quotaUnlimited')}</strong></div>
                <div className="flex items-center gap-2"><button type="button" className={PRIMARY_BUTTON_CLASS} disabled={pending} onClick={() => setPolicyEditorOpen(true)}><Save aria-hidden="true" size={15} />{t('saveQuotaPolicy')}</button><button type="button" className={SECONDARY_BUTTON_CLASS} disabled={pending || !summary.quotaBytes} onClick={() => setClearConfirmationOpen(true)}><Trash2 aria-hidden="true" size={15} />{t('clearQuotaPolicy')}</button></div>
              </div>
            </section>
          </>
        ) : (
          <div className={CARD_CLASS}><EmptyState title={t('quotaEmpty')} description={t('quotaPageDescription')} icon={Gauge} /></div>
        )}
      </div>

      <Drawer open={policyEditorOpen} onOpenChange={(open) => { if (!pending) setPolicyEditorOpen(open); }}>
        <DrawerContent size="sm">
          <DrawerHeader><DrawerTitle>{t('quotaPolicyTitle')}</DrawerTitle><DrawerDescription>{t('quotaPolicyHint')}</DrawerDescription></DrawerHeader>
          <DrawerBody>
            <label className="flex min-w-0 flex-col gap-2 text-xs font-medium text-neutral-600 dark:text-neutral-300">
              {t('quotaCapLabel')}
              <input autoFocus type="number" min="1" inputMode="numeric" className={`${INPUT_CLASS} font-mono`} value={quotaInput} onChange={(event) => setQuotaInput(event.target.value)} placeholder={t('quotaBytesPlaceholder')} />
              <span className="text-[11px] font-normal text-neutral-500 dark:text-neutral-400">{t('quotaInputPreview', { value: quotaPreview })}</span>
            </label>
          </DrawerBody>
          <DrawerFooter><button type="button" className={SECONDARY_BUTTON_CLASS} disabled={pending} onClick={() => setPolicyEditorOpen(false)}>{t('cancel')}</button><button type="button" className={PRIMARY_BUTTON_CLASS} disabled={pending} onClick={() => void savePolicy()}>{pending ? t('saving') : t('saveQuotaPolicy')}</button></DrawerFooter>
        </DrawerContent>
      </Drawer>

      <OperationsConfirmDialog
        open={clearConfirmationOpen}
        title={t('confirmClearQuotaTitle')}
        message={t('confirmClearQuotaMessage')}
        confirmLabel={t('confirmClearQuotaAction')}
        cancelLabel={t('cancel')}
        variant="danger"
        onCancel={() => setClearConfirmationOpen(false)}
        onConfirm={() => void clearPolicy()}
      />
    </main>
  );
}

function QuotaMetric({
  detail,
  icon: Icon,
  label,
  tone,
  value,
}: {
  detail: string;
  icon: typeof Database;
  label: string;
  tone: 'blue' | 'cyan' | 'violet';
  value: string;
}) {
  const tones = {
    blue: 'bg-blue-50 text-blue-600 dark:bg-blue-950/40 dark:text-blue-300',
    cyan: 'bg-cyan-50 text-cyan-700 dark:bg-cyan-950/40 dark:text-cyan-300',
    violet: 'bg-violet-50 text-violet-700 dark:bg-violet-950/40 dark:text-violet-300',
  };
  return (
    <div className={`${CARD_CLASS} flex min-w-0 items-center gap-4 p-5`}>
      <div className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-lg ${tones[tone]}`}><Icon aria-hidden="true" size={19} /></div>
      <div className="min-w-0">
        <p className="text-xs font-medium text-neutral-500 dark:text-neutral-400">{label}</p>
        <p className="mt-1 truncate text-xl font-semibold text-neutral-950 dark:text-white">{value}</p>
        <p className="mt-0.5 truncate font-mono text-[10px] text-neutral-400 dark:text-neutral-500">{detail}</p>
      </div>
    </div>
  );
}
