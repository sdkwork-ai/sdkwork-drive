import React, { useEffect, useMemo, useState, type FormEvent } from 'react';
import {
  ChevronLeft,
  ChevronRight,
  CircleAlert,
  HardDrive,
  RefreshCw,
  SlidersHorizontal,
  UserRound,
} from 'lucide-react';
import type { DriveBackendSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import { createDriveOperationsAdminService } from '../services/driveOperationsAdminService';
import type { DriveSpaceAdminView } from '../types/driveOperationsAdminTypes';
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

interface SpacesAdminPageProps {
  backendSdkClient: DriveBackendSdkClient;
  getSession: () => SessionSnapshot;
}

const SPACE_TYPE_TONE: Record<string, string> = {
  personal: 'bg-blue-50 text-blue-700 dark:bg-blue-950/40 dark:text-blue-300',
  team: 'bg-violet-50 text-violet-700 dark:bg-violet-950/40 dark:text-violet-300',
  shared: 'bg-cyan-50 text-cyan-700 dark:bg-cyan-950/40 dark:text-cyan-300',
  system: 'bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300',
};

const SPACE_STATUS_TONE: Record<string, string> = {
  active: 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300',
  inactive: 'bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300',
  archived: 'bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-300',
  deleted: 'bg-red-50 text-red-700 dark:bg-red-950/40 dark:text-red-300',
};

function resolveTone(map: Record<string, string>, value: string): string {
  return map[value.toLowerCase()] ?? 'bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300';
}

export function SpacesAdminPage({ backendSdkClient, getSession }: SpacesAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo(
    () => createDriveOperationsAdminService({ backendSdkClient, getSession }),
    [backendSdkClient, getSession],
  );
  const [spaces, setSpaces] = useState<DriveSpaceAdminView[]>([]);
  const [page, setPage] = useState(1);
  const [pageSize] = useState(20);
  const [pageCursors, setPageCursors] = useState<Record<number, string | undefined>>({ 1: undefined });
  const [nextPageToken, setNextPageToken] = useState<string | null>(null);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | undefined>();
  const [ownerSubjectType, setOwnerSubjectType] = useState('');
  const [ownerSubjectId, setOwnerSubjectId] = useState('');
  const [appliedOwnerSubjectType, setAppliedOwnerSubjectType] = useState<string | undefined>();
  const [appliedOwnerSubjectId, setAppliedOwnerSubjectId] = useState<string | undefined>();
  const [refreshKey, setRefreshKey] = useState(0);

  const currentPageToken = pageCursors[page];

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError(undefined);
    service.listSpaces({
      ownerSubjectType: appliedOwnerSubjectType,
      ownerSubjectId: appliedOwnerSubjectId,
      pageSize,
      pageToken: currentPageToken,
      signal: controller.signal,
    })
      .then((result) => {
        setSpaces(result.items);
        setTotal(result.total);
        const serverNextCursor = result.pageInfo?.nextCursor ?? null;
        setNextPageToken(serverNextCursor);
        setPageCursors((current) => {
          const next = { ...current };
          Object.keys(next)
            .map(Number)
            .filter((cursorPage) => cursorPage > page + 1)
            .forEach((cursorPage) => {
              delete next[cursorPage];
            });
          if (serverNextCursor) {
            next[page + 1] = serverNextCursor;
          } else {
            delete next[page + 1];
          }
          return next;
        });
      })
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setError(t('noticeLoadFailed'));
      })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [
    service,
    appliedOwnerSubjectId,
    appliedOwnerSubjectType,
    currentPageToken,
    page,
    pageSize,
    refreshKey,
    t,
  ]);

  const totalPages = nextPageToken
    ? Math.max(page + 1, Math.ceil(total / pageSize), 1)
    : Math.max(page, Math.ceil(total / pageSize), 1);

  const applyFilters = () => {
    setPageCursors({ 1: undefined });
    setNextPageToken(null);
    setAppliedOwnerSubjectType(ownerSubjectType.trim() || undefined);
    setAppliedOwnerSubjectId(ownerSubjectId.trim() || undefined);
    setPage(1);
    setRefreshKey((current) => current + 1);
  };

  const refresh = () => setRefreshKey((current) => current + 1);

  const handleFilterSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    applyFilters();
  };

  return (
    <div className="flex h-full min-h-0 w-full min-w-0 flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <div aria-label={t('spacesPageTitle')} className="flex shrink-0 justify-end px-4 pt-4 sm:px-6 sm:pt-6">
        <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loading} onClick={refresh}>
          <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
          {t('refresh')}
        </button>
      </div>
      <div className="flex-1 space-y-4 overflow-auto p-4 sm:p-6">
        {error ? (
          <div className="flex items-start gap-3 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900/50 dark:bg-red-950/40 dark:text-red-300" role="alert">
            <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
            <span className="flex-1">{error}</span>
            <button type="button" className="font-medium underline underline-offset-2" onClick={refresh}>{t('refresh')}</button>
          </div>
        ) : null}

        <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="spaces-filters-title">
          <div className={`${CARD_HEADER_CLASS} flex items-center gap-2`}>
            <SlidersHorizontal aria-hidden="true" className="text-blue-600 dark:text-blue-400" size={16} />
            <h2 id="spaces-filters-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">{t('filtersTitle')}</h2>
          </div>
          <form className="flex flex-wrap items-end gap-4 p-5" onSubmit={handleFilterSubmit}>
            <label className="flex min-w-[220px] flex-1 flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">
              {t('filterOwnerType')}
              <div className="relative">
                <UserRound aria-hidden="true" className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400" size={15} />
                <input className={`${INPUT_CLASS} w-full pl-9`} value={ownerSubjectType} onChange={(e) => setOwnerSubjectType(e.target.value)} />
              </div>
            </label>
            <label className="flex min-w-[260px] flex-[1.35] flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">
              {t('filterOwnerId')}
              <input className={`${INPUT_CLASS} w-full font-mono`} value={ownerSubjectId} onChange={(e) => setOwnerSubjectId(e.target.value)} />
            </label>
            <button type="submit" className={PRIMARY_BUTTON_CLASS}>{t('applyFilters')}</button>
          </form>
        </section>

        <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="spaces-list-title">
          <div className={`${CARD_HEADER_CLASS} flex flex-wrap items-center justify-between gap-3`}>
            <div>
              <h2 id="spaces-list-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">{t('spacesListTitle')}</h2>
              <p className="mt-0.5 text-xs text-neutral-500 dark:text-neutral-400">{t('countOf', { filtered: spaces.length, total })}</p>
            </div>
            <span className="rounded-full bg-neutral-100 px-2.5 py-1 text-xs font-medium text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300">
              {t('pageOf', { page, totalPages })}
            </span>
          </div>
          <div className="overflow-x-auto">
            <table className={`${TABLE_CLASS} min-w-[760px]`}>
              <thead className={TABLE_HEAD_CLASS}>
                <tr>
                  <th className="w-[34%] px-5 py-3 text-left">{t('colSpaceName')}</th>
                  <th className="w-[18%] px-5 py-3 text-left">{t('colSpaceType')}</th>
                  <th className="w-[30%] px-5 py-3 text-left">{t('colOwner')}</th>
                  <th className="w-[18%] px-5 py-3 text-left">{t('colStatus')}</th>
                </tr>
              </thead>
              <tbody>
                {loading ? (
                  Array.from({ length: 4 }, (_, index) => (
                    <tr key={`space-skeleton-${index}`} className={TABLE_ROW_CLASS}>
                      <td colSpan={4} className="px-5 py-4">
                        <div className="flex animate-pulse items-center gap-3">
                          <div className="h-9 w-9 rounded-lg bg-neutral-200 dark:bg-neutral-800" />
                          <div className="flex-1 space-y-2">
                            <div className="h-3 w-2/5 rounded bg-neutral-200 dark:bg-neutral-800" />
                            <div className="h-2.5 w-1/4 rounded bg-neutral-100 dark:bg-neutral-800/70" />
                          </div>
                          <div className="h-6 w-20 rounded-full bg-neutral-100 dark:bg-neutral-800/70" />
                        </div>
                      </td>
                    </tr>
                  ))
                ) : spaces.length === 0 ? (
                  <tr className={TABLE_ROW_CLASS}>
                    <td colSpan={4} className="px-5 py-14 text-center">
                      <div className="mx-auto flex max-w-sm flex-col items-center">
                        <div className="flex h-11 w-11 items-center justify-center rounded-full bg-neutral-100 text-neutral-400 dark:bg-neutral-800 dark:text-neutral-500">
                          <HardDrive aria-hidden="true" size={20} strokeWidth={1.7} />
                        </div>
                        <p className="mt-3 text-sm font-medium text-neutral-700 dark:text-neutral-200">{t('spacesEmpty')}</p>
                        <p className="mt-1 text-xs text-neutral-500 dark:text-neutral-400">{t('spacesPageDescription')}</p>
                      </div>
                    </td>
                  </tr>
                ) : (
                  spaces.map((space) => (
                    <tr key={space.id} className={TABLE_ROW_CLASS}>
                      <td className="px-5 py-3.5 align-top">
                        <div className="flex min-w-0 items-center gap-3">
                          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-blue-50 text-blue-600 dark:bg-blue-950/40 dark:text-blue-300">
                            <HardDrive aria-hidden="true" size={17} strokeWidth={1.8} />
                          </div>
                          <div className="min-w-0">
                            <div className="truncate text-sm font-semibold text-neutral-900 dark:text-neutral-100">{space.displayName || '--'}</div>
                            <div className="mt-0.5 truncate font-mono text-[11px] text-neutral-400 dark:text-neutral-500">{space.id}</div>
                          </div>
                        </div>
                      </td>
                      <td className="px-5 py-3.5 align-top">
                        <span className={`inline-flex items-center rounded-full px-2.5 py-1 text-xs font-medium ${resolveTone(SPACE_TYPE_TONE, space.spaceType)}`}>
                          {t(`spaceType.${space.spaceType}`)}
                        </span>
                      </td>
                      <td className="px-5 py-3.5 align-top">
                        <div className="flex min-w-0 items-start gap-2">
                          <UserRound aria-hidden="true" className="mt-0.5 shrink-0 text-neutral-400" size={15} />
                          <div className="min-w-0">
                            <div className="text-xs font-medium text-neutral-700 dark:text-neutral-200">{t(`ownerType.${space.ownerSubjectType}`)}</div>
                            <div className="mt-0.5 truncate font-mono text-[11px] text-neutral-500 dark:text-neutral-400">{space.ownerSubjectId}</div>
                          </div>
                        </div>
                      </td>
                      <td className="px-5 py-3.5 align-top">
                        <span className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium ${resolveTone(SPACE_STATUS_TONE, space.lifecycleStatus)}`}>
                          <span className="h-1.5 w-1.5 rounded-full bg-current" aria-hidden="true" />
                          {t(`spaceStatus.${space.lifecycleStatus}`)}
                        </span>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
          <div className="flex flex-wrap items-center justify-between gap-3 border-t border-neutral-200 px-5 py-3 dark:border-neutral-800">
            <span className="text-xs text-neutral-500 dark:text-neutral-400">{t('pageOf', { page, totalPages })}</span>
            <div className="flex items-center gap-2">
              <button
                type="button"
                className={`${SECONDARY_BUTTON_CLASS} px-2.5`}
                aria-label={t('previousPage')}
                disabled={page <= 1 || loading}
                onClick={() => setPage((current) => Math.max(1, current - 1))}
              >
                <ChevronLeft aria-hidden="true" size={16} />
                <span className="hidden sm:inline">{t('previousPage')}</span>
              </button>
              <button
                type="button"
                className={`${SECONDARY_BUTTON_CLASS} px-2.5`}
                aria-label={t('nextPage')}
                disabled={!nextPageToken || loading}
                onClick={() => setPage((current) => current + 1)}
              >
                <span className="hidden sm:inline">{t('nextPage')}</span>
                <ChevronRight aria-hidden="true" size={16} />
              </button>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
