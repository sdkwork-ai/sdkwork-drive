import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Activity,
  Archive,
  CircleAlert,
  ChartNoAxesColumn,
  Database,
  HardDrive,
  Link2,
  LoaderCircle,
  RefreshCw,
  Server,
  ServerCog,
} from 'lucide-react';
import { formatDriveBytes } from 'sdkwork-drive-pc-commons';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import {
  createStorageProviderAdminService,
  type StorageProviderAdminService,
} from '../services/storageProviderAdminService';
import type {
  StorageOverviewProviderUsageView,
  StorageOverviewView,
} from '../types/storageProviderAdminTypes';
import { getProviderKindMeta } from '../utils/providerKindConfig';
import {
  BADGE_BASE_CLASS,
  CARD_CLASS,
  SECONDARY_BUTTON_CLASS,
  SELECT_CLASS,
} from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

interface StorageOverviewAdminPageProps {
  adminStorageSdkClient: DriveAdminStorageSdkClient;
  getSession: () => SessionSnapshot;
}

const TREND_MONTH_OPTIONS = [6, 12, 24];

export function StorageOverviewAdminPage({
  adminStorageSdkClient,
  getSession,
}: StorageOverviewAdminPageProps) {
  const { t, language } = useTranslation();
  const service = useMemo<StorageProviderAdminService>(
    () => createStorageProviderAdminService({ adminStorageSdkClient, getSession }),
    [adminStorageSdkClient, getSession],
  );
  const [overview, setOverview] = useState<StorageOverviewView | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);
  const [trendMonths, setTrendMonths] = useState(12);

  const load = useCallback(
    (signal?: AbortSignal) => {
      setLoading(true);
      setLoadError(false);
      service
        .getStorageOverview({ trendMonths, signal })
        .then((next) => {
          setOverview(next);
        })
        .catch((err) => {
          if (!isDriveRequestCancellationError(err)) {
            setLoadError(true);
          }
        })
        .finally(() => setLoading(false));
    },
    [service, trendMonths],
  );

  useEffect(() => {
    const controller = new AbortController();
    load(controller.signal);
    return () => controller.abort();
  }, [load]);

  const locale = language === 'zh' ? 'zh-CN' : 'en-US';
  const formatCount = (value: number) => value.toLocaleString(locale);

  return (
    <main
      aria-label={t('overviewPageTitle')}
      className="flex h-full min-h-0 flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100"
    >
      <div className="flex shrink-0 flex-wrap items-center justify-between gap-3 px-4 pt-4 sm:px-6 sm:pt-6">
        <div className="min-w-0">
          <h1 className="truncate text-base font-semibold">{t('overviewPageTitle')}</h1>
          <p className="mt-0.5 truncate text-xs text-neutral-500 dark:text-neutral-400">
            {t('overviewSubtitle')}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {overview ? (
            <span
              className={`${BADGE_BASE_CLASS} bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300`}
            >
              {t('overviewTenantLabel', { tenantId: overview.scopeTenantId })}
            </span>
          ) : null}
          <button
            type="button"
            className={SECONDARY_BUTTON_CLASS}
            disabled={loading}
            onClick={() => load()}
          >
            <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
            {t('refresh')}
          </button>
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-auto p-4 sm:p-6">
        {loadError ? (
          <div className="flex items-start gap-3 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/30 dark:text-red-200">
            <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
            <div className="min-w-0 flex-1">
              <p>{t('overviewNoticeLoadFailed')}</p>
              <button
                type="button"
                className="mt-2 text-xs font-medium underline"
                onClick={() => load()}
              >
                {t('overviewRetry')}
              </button>
            </div>
          </div>
        ) : loading && !overview ? (
          <OverviewSkeleton />
        ) : overview ? (
          <div className="grid content-start gap-4">
            {isNothingStored(overview) ? (
              <div className="flex items-start gap-3 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800 dark:border-amber-900 dark:bg-amber-950/30 dark:text-amber-200">
                <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
                <div className="min-w-0">
                  <p className="font-medium">{t('overviewEmptyTitle')}</p>
                  <p className="mt-0.5 text-xs">{t('overviewEmptyDesc')}</p>
                </div>
              </div>
            ) : null}

            <div className="grid grid-cols-2 gap-3 lg:grid-cols-3 xl:grid-cols-6">
              <MetricTile
                icon={<HardDrive aria-hidden="true" size={16} />}
                label={t('overviewUsedCapacity')}
                value={formatDriveBytes(overview.capacity.usedBytes)}
                detail={usedCapacityDetail(overview.capacity, t, formatDriveBytes)}
              />
              <MetricTile
                icon={<Database aria-hidden="true" size={16} />}
                label={t('overviewActiveObjects')}
                value={formatCount(overview.capacity.activeObjectCount)}
                detail={`${t('overviewColumnObjects')}: ${formatCount(overview.capacity.totalObjectCount)}`}
              />
              <MetricTile
                icon={<Archive aria-hidden="true" size={16} />}
                label={t('overviewDeletedObjects')}
                value={formatCount(overview.capacity.deletedObjectCount)}
                detail={t('overviewBucketsInUse') + ': ' + formatCount(overview.capacity.bucketCount)}
              />
              <MetricTile
                icon={<ServerCog aria-hidden="true" size={16} />}
                label={t('overviewProvidersInUse')}
                value={formatCount(overview.providers.totalCount)}
                detail={[
                  overview.providers.disabledCount > 0
                    ? t('overviewDisabledCount', { count: overview.providers.disabledCount })
                    : '',
                  overview.providers.deletedCount > 0
                    ? t('overviewDeletedCount', { count: overview.providers.deletedCount })
                    : '',
                ]
                  .filter(Boolean)
                  .join(' · ') || t('overviewStatusActive')}
              />
              <MetricTile
                icon={<Link2 aria-hidden="true" size={16} />}
                label={t('overviewActiveBindings')}
                value={formatCount(overview.bindings.activeCount)}
                detail={`${formatCount(overview.bindings.totalCount)} ${t('overviewBindingHealthTitle')}`}
              />
              <MetricTile
                icon={<Server aria-hidden="true" size={16} />}
                label={t('overviewCatalogEnabled')}
                value={formatCount(overview.catalog.enabledCount)}
                detail={`${formatCount(overview.catalog.totalCount)} ${t('overviewColumnProvider')}`}
              />
            </div>

            <div className="grid gap-4 xl:grid-cols-2">
              <section className={`${CARD_CLASS} p-5`}>
                <header className="mb-4 flex items-center justify-between gap-3">
                  <div className="flex items-center gap-2 text-sm font-semibold">
                    <HardDrive aria-hidden="true" size={16} className="text-blue-600 dark:text-blue-400" />
                    {t('overviewCapacityTitle')}
                  </div>
                  <span className="text-xs text-neutral-500 dark:text-neutral-400">
                    {t('overviewUpdatedAt', { time: formatGeneratedAt(overview.generatedAt, locale) })}
                  </span>
                </header>
                <QuotaBar overview={overview} />
                <dl className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-3">
                  <StatRow label={t('overviewAverageObject')} value={formatDriveBytes(overview.capacity.averageObjectBytes)} />
                  <StatRow
                    label={t('overviewLargestObject')}
                    value={
                      overview.capacity.largestObjectBytes === undefined
                        ? t('overviewNoneValue')
                        : formatDriveBytes(overview.capacity.largestObjectBytes)
                    }
                  />
                  <StatRow label={t('overviewBucketsInUse')} value={formatCount(overview.capacity.bucketCount)} />
                  <StatRow label={t('overviewActiveObjects')} value={formatCount(overview.capacity.activeObjectCount)} />
                  <StatRow label={t('overviewDeletedObjects')} value={formatCount(overview.capacity.deletedObjectCount)} />
                  <StatRow label={t('overviewColumnObjects')} value={formatCount(overview.capacity.totalObjectCount)} />
                </dl>
              </section>

              <section className={`${CARD_CLASS} p-5`}>
                <header className="mb-4 flex items-center gap-2 text-sm font-semibold">
                  <Link2 aria-hidden="true" size={16} className="text-blue-600 dark:text-blue-400" />
                  {t('overviewBindingHealthTitle')}
                </header>
                <div className="grid gap-3">
                  {overview.bindings.hasTenantDefault ? (
                    <div className="flex items-start gap-2 rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-xs text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950/30 dark:text-emerald-200">
                      <span className="font-medium">{t('overviewDefaultBinding')}:</span>
                      <span className="break-all font-mono">
                        {overview.bindings.tenantDefaultProviderId ?? overview.bindings.tenantDefaultBindingId}
                      </span>
                    </div>
                  ) : (
                    <div className="flex items-start gap-2 rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-800 dark:border-amber-900 dark:bg-amber-950/30 dark:text-amber-200">
                      <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={14} />
                      <span>{t('overviewDefaultBindingMissing')}</span>
                    </div>
                  )}
                  <div className="grid grid-cols-3 gap-3">
                    <StatRow label={t('overviewScopeTenant')} value={formatCount(overview.bindings.byScope.tenantCount)} />
                    <StatRow label={t('overviewScopeSpace')} value={formatCount(overview.bindings.byScope.spaceCount)} />
                    <StatRow
                      label={t('overviewScopeSpaceType')}
                      value={formatCount(overview.bindings.byScope.spaceTypeCount)}
                    />
                  </div>
                  <div className="grid grid-cols-2 gap-3">
                    <StatRow label={t('overviewStatusActive')} value={formatCount(overview.bindings.activeCount)} />
                    <StatRow label={t('overviewStatusDisabled')} value={formatCount(overview.bindings.inactiveCount)} />
                  </div>
                </div>
              </section>
            </div>

            <ProviderUsageCard
              usage={overview.providers.usage}
              emptyText={t('overviewNoProvidersInUse')}
              labels={{
                title: t('overviewProviderUsageTitle'),
                configuration: t('overviewColumnConfiguration'),
                provider: t('overviewColumnProvider'),
                bucket: t('overviewColumnBucket'),
                objects: t('overviewColumnObjects'),
                stored: t('overviewColumnStored'),
                share: t('overviewColumnShare'),
                defaultBadge: t('overviewDefaultBadge'),
                statusActive: t('overviewStatusActive'),
                statusDisabled: t('overviewStatusDisabled'),
                statusDeleted: t('overviewStatusDeleted'),
              }}
              formatCount={formatCount}
            />

            <TrendCard
              trend={overview.trend}
              months={trendMonths}
              onMonthsChange={setTrendMonths}
              labels={{
                title: t('overviewTrendTitle'),
                lastMonths: t('overviewTrendLastMonths', { count: trendMonths }),
                objects: t('overviewTrendObjects'),
                empty: t('overviewTrendEmpty'),
              }}
              formatCount={formatCount}
            />
          </div>
        ) : (
          <OverviewSkeleton />
        )}
      </div>
    </main>
  );
}

function QuotaBar({ overview }: { overview: StorageOverviewView }) {
  const { t } = useTranslation();
  const { capacity } = overview;
  const percent = quotaPercent(capacity.quotaUsageRatio);
  const exceeded = (capacity.quotaUsageRatio ?? 0) > 1;

  return (
    <div>
      <div className="mb-1.5 flex flex-wrap items-baseline justify-between gap-2">
        <span className="text-2xl font-semibold tabular-nums">
          {formatDriveBytes(capacity.usedBytes)}
        </span>
        <span className="text-xs text-neutral-500 dark:text-neutral-400">
          {capacity.quotaConfigured && capacity.quotaBytes !== undefined
            ? exceeded
              ? t('overviewQuotaExceeded', {
                  percent,
                  limit: formatDriveBytes(capacity.quotaBytes),
                })
              : t('overviewQuotaUsage', {
                  percent,
                  limit: formatDriveBytes(capacity.quotaBytes),
                })
            : t('overviewQuotaUnlimited')}
        </span>
      </div>
      {capacity.quotaConfigured ? (
        <div
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={Math.min(percent, 100)}
          aria-label={t('overviewUsedCapacity')}
          className="h-2 w-full overflow-hidden rounded-full bg-neutral-200 dark:bg-neutral-800"
        >
          <div
            className={`h-full rounded-full ${exceeded ? 'bg-red-500' : 'bg-blue-500'}`}
            style={{ width: `${Math.min(Math.max(percent, capacity.usedBytes > 0 ? 2 : 0), 100)}%` }}
          />
        </div>
      ) : (
        <div className="h-2 w-full overflow-hidden rounded-full bg-neutral-200 dark:bg-neutral-800">
          <div className="h-full rounded-full bg-neutral-400/60" style={{ width: '100%' }} />
        </div>
      )}
    </div>
  );
}

function ProviderUsageCard({
  usage,
  emptyText,
  labels,
  formatCount,
}: {
  usage: StorageOverviewProviderUsageView[];
  emptyText: string;
  labels: {
    title: string;
    configuration: string;
    provider: string;
    bucket: string;
    objects: string;
    stored: string;
    share: string;
    defaultBadge: string;
    statusActive: string;
    statusDisabled: string;
    statusDeleted: string;
  };
  formatCount: (value: number) => string;
}) {
  return (
    <section className={CARD_CLASS}>
      <header className="border-b border-neutral-100 px-5 py-3 dark:border-neutral-800">
        <div className="flex items-center gap-2 text-sm font-semibold">
          <ChartNoAxesColumn aria-hidden="true" size={16} className="text-blue-600 dark:text-blue-400" />
          {labels.title}
        </div>
      </header>
      {usage.length === 0 ? (
        <p className="px-5 py-6 text-center text-sm text-neutral-500 dark:text-neutral-400">
          {emptyText}
        </p>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full min-w-[720px] border-collapse text-sm">
            <thead>
              <tr className="text-left text-xs uppercase tracking-wide text-neutral-500 dark:text-neutral-400">
                <th className="px-5 py-2 font-medium">{labels.configuration}</th>
                <th className="px-3 py-2 font-medium">{labels.provider}</th>
                <th className="px-3 py-2 font-medium">{labels.bucket}</th>
                <th className="px-3 py-2 text-right font-medium">{labels.objects}</th>
                <th className="px-3 py-2 text-right font-medium">{labels.stored}</th>
                <th className="px-5 py-2 font-medium">{labels.share}</th>
              </tr>
            </thead>
            <tbody>
              {usage.map((row) => {
                const meta = getProviderKindMeta(row.providerKind);
                const percent = Math.round(row.capacityShare * 100);
                return (
                  <tr
                    key={row.providerId}
                    className="border-t border-neutral-100 dark:border-neutral-800"
                  >
                    <td className="px-5 py-3">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-medium">{row.name || row.providerId}</span>
                        {row.isTenantDefault ? (
                          <span
                            className={`${BADGE_BASE_CLASS} bg-blue-50 text-blue-700 dark:bg-blue-950 dark:text-blue-300`}
                          >
                            {labels.defaultBadge}
                          </span>
                        ) : null}
                        {row.status !== 'active' ? (
                          <span
                            className={`${BADGE_BASE_CLASS} ${
                              row.status === 'deleted'
                                ? 'bg-red-50 text-red-700 dark:bg-red-950 dark:text-red-300'
                                : 'bg-amber-50 text-amber-700 dark:bg-amber-950 dark:text-amber-300'
                            }`}
                          >
                            {row.status === 'deleted' ? labels.statusDeleted : labels.statusDisabled}
                          </span>
                        ) : null}
                      </div>
                      <div className="mt-0.5 break-all font-mono text-xs text-neutral-500 dark:text-neutral-400">
                        {row.providerId}
                      </div>
                    </td>
                    <td className="px-3 py-3">
                      <span className={`inline-flex items-center gap-1 ${meta.textClass}`}>
                        {meta.icon}
                        <span className="text-xs">{meta.shortLabel}</span>
                      </span>
                    </td>
                    <td className="px-3 py-3 break-all font-mono text-xs text-neutral-600 dark:text-neutral-300">
                      {row.bucket}
                    </td>
                    <td className="px-3 py-3 text-right tabular-nums">{formatCount(row.objectCount)}</td>
                    <td className="px-3 py-3 text-right whitespace-nowrap tabular-nums">
                      {formatDriveBytes(row.usedBytes)}
                    </td>
                    <td className="px-5 py-3">
                      <div className="flex items-center gap-2">
                        <span className="w-9 shrink-0 text-right text-xs tabular-nums text-neutral-500 dark:text-neutral-400">
                          {percent}%
                        </span>
                        <span className="h-2 min-w-[60px] flex-1 overflow-hidden rounded-full bg-neutral-200 dark:bg-neutral-800">
                          <span
                            className="block h-full rounded-full bg-blue-500"
                            style={{ width: `${Math.min(Math.max(percent, row.usedBytes > 0 ? 2 : 0), 100)}%` }}
                          />
                        </span>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}

function TrendCard({
  trend,
  months,
  onMonthsChange,
  labels,
  formatCount,
}: {
  trend: StorageOverviewView['trend'];
  months: number;
  onMonthsChange: (months: number) => void;
  labels: { title: string; lastMonths: string; objects: string; empty: string };
  formatCount: (value: number) => string;
}) {
  const maxBytes = trend.reduce((max, point) => Math.max(max, point.bytes), 0);
  const hasData = trend.some((point) => point.bytes > 0 || point.objectCount > 0);

  return (
    <section className={CARD_CLASS}>
      <header className="flex flex-wrap items-center justify-between gap-3 border-b border-neutral-100 px-5 py-3 dark:border-neutral-800">
        <div className="flex items-center gap-2 text-sm font-semibold">
          <Activity aria-hidden="true" size={16} className="text-blue-600 dark:text-blue-400" />
          {labels.title}
          <span className="ml-1 text-xs font-normal text-neutral-500 dark:text-neutral-400">
            {labels.lastMonths}
          </span>
        </div>
        <select
          aria-label={labels.title}
          className={SELECT_CLASS}
          value={months}
          onChange={(event) => onMonthsChange(Number(event.target.value))}
        >
          {TREND_MONTH_OPTIONS.map((option) => (
            <option key={option} value={option}>
              {String(option)}
            </option>
          ))}
        </select>
      </header>
      <div className="px-5 py-4">
        {!hasData ? (
          <p className="py-6 text-center text-sm text-neutral-500 dark:text-neutral-400">
            {labels.empty}
          </p>
        ) : (
          <div className="flex h-40 items-end gap-1.5 sm:gap-2">
            {trend.map((point) => {
              const height = maxBytes > 0 && point.bytes > 0 ? Math.max((point.bytes / maxBytes) * 100, 3) : 2;
              return (
                <div key={point.periodLabel} className="flex min-w-0 flex-1 flex-col items-center gap-1.5">
                  <span className="max-w-full truncate text-[10px] text-neutral-500 dark:text-neutral-400">
                    {formatDriveBytes(point.bytes)}
                  </span>
                  <div className="flex w-full flex-1 items-end">
                    <div
                      title={`${point.periodLabel} · ${formatCount(point.objectCount)} ${labels.objects}`}
                      className="w-full rounded-t bg-blue-500/85"
                      style={{ height: `${height}%` }}
                    />
                  </div>
                  <span className="max-w-full truncate text-[10px] text-neutral-500 dark:text-neutral-400">
                    {point.periodLabel.slice(5)}
                  </span>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </section>
  );
}

function MetricTile({
  icon,
  label,
  value,
  detail,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <div className={`${CARD_CLASS} p-4`}>
      <div className="flex items-center gap-2 text-xs text-neutral-500 dark:text-neutral-400">
        <span className="text-blue-600 dark:text-blue-400">{icon}</span>
        <span className="truncate">{label}</span>
      </div>
      <p className="mt-2 truncate text-xl font-semibold tabular-nums">{value}</p>
      <p className="mt-1 truncate text-xs text-neutral-500 dark:text-neutral-400">{detail}</p>
    </div>
  );
}

function StatRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-neutral-100 px-3 py-2 dark:border-neutral-800">
      <dt className="truncate text-xs text-neutral-500 dark:text-neutral-400">{label}</dt>
      <dd className="mt-0.5 truncate text-sm font-medium tabular-nums">{value}</dd>
    </div>
  );
}

function OverviewSkeleton() {
  return (
    <div className="grid content-start gap-4" aria-hidden="true">
      <div className="grid grid-cols-2 gap-3 lg:grid-cols-3 xl:grid-cols-6">
        {Array.from({ length: 6 }).map((_, index) => (
          <div key={index} className={`${CARD_CLASS} h-[104px] animate-pulse`} />
        ))}
      </div>
      <div className="grid gap-4 xl:grid-cols-2">
        <div className={`${CARD_CLASS} h-[220px] animate-pulse`} />
        <div className={`${CARD_CLASS} h-[220px] animate-pulse`} />
      </div>
      <div className={`${CARD_CLASS} h-[280px] animate-pulse`} />
    </div>
  );
}

/// A tenant with no objects, no bindings and no providers has nothing to chart;
/// the frame stays mounted with zeroes, so this only drives the hint banner.
function isNothingStored(overview: StorageOverviewView): boolean {
  return (
    overview.capacity.usedBytes === 0
    && overview.capacity.totalObjectCount === 0
    && overview.providers.totalCount === 0
    && overview.bindings.totalCount === 0
  );
}

/// Quota line under the used-capacity tile. Mirrors `QuotaBar`, so both places
/// describe the same division — a drift here reads as "the numbers disagree".
function usedCapacityDetail(
  capacity: StorageOverviewView['capacity'],
  t: (key: string, params?: Record<string, string | number>) => string,
  formatBytes: (value?: number | null) => string,
): string {
  if (!capacity.quotaConfigured || capacity.quotaBytes === undefined) {
    return t('overviewQuotaUnlimited');
  }
  const percent = quotaPercent(capacity.quotaUsageRatio);
  const params = { percent, limit: formatBytes(capacity.quotaBytes) };
  return (capacity.quotaUsageRatio ?? 0) > 1
    ? t('overviewQuotaExceeded', params)
    : t('overviewQuotaUsage', params);
}

function quotaPercent(ratio: number | undefined): number {
  if (ratio === undefined || !Number.isFinite(ratio)) {
    return 0;
  }
  return Math.round(ratio * 100);
}

function formatGeneratedAt(value: string, locale: string): string {
  if (!value) {
    return '--';
  }
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  return parsed.toLocaleString(locale, { hour12: false });
}
