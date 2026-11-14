import { useEffect, useMemo, useState, type FormEvent } from 'react';
import {
  Archive,
  CircleAlert,
  ExternalLink,
  File as FileIcon,
  FileText,
  FolderOpen,
  Image as ImageIcon,
  Link2,
  Music,
  RefreshCw,
  Search,
  SlidersHorizontal,
  Video,
  X,
} from 'lucide-react';
import type { DriveBackendSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import { EmptyState, LoadingState, PaginationBar } from '../components/OperationsAdminPrimitives';
import { createDriveOperationsAdminService } from '../services/driveOperationsAdminService';
import type { DriveFileAdminView } from '../types/driveOperationsAdminTypes';
import {
  BADGE_BASE_CLASS,
  CARD_CLASS,
  CARD_HEADER_CLASS,
  GHOST_BUTTON_CLASS,
  INPUT_CLASS,
  PRIMARY_BUTTON_CLASS,
  SELECT_CLASS,
  TABLE_CLASS,
  TABLE_HEAD_CLASS,
  TABLE_ROW_CLASS,
} from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

interface FilesAdminPageProps {
  backendSdkClient: DriveBackendSdkClient;
  getSession: () => SessionSnapshot;
  /**
   * Space preselection used when an operator arrives from the spaces page.
   * The host remounts the page when this value changes, so the filter is
   * applied once instead of fighting the operator's later edits.
   */
  initialSpaceId?: string;
}

const PAGE_SIZE = 20;

const SPACE_TYPE_TONE: Record<string, string> = {
  personal: 'bg-blue-50 text-blue-700 dark:bg-blue-950/40 dark:text-blue-300',
  team: 'bg-violet-50 text-violet-700 dark:bg-violet-950/40 dark:text-violet-300',
  knowledge_base: 'bg-cyan-50 text-cyan-700 dark:bg-cyan-950/40 dark:text-cyan-300',
  ai_generated: 'bg-fuchsia-50 text-fuchsia-700 dark:bg-fuchsia-950/40 dark:text-fuchsia-300',
  git_repository: 'bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300',
  deployment: 'bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-300',
  app_upload: 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300',
  im: 'bg-sky-50 text-sky-700 dark:bg-sky-950/40 dark:text-sky-300',
  rtc: 'bg-teal-50 text-teal-700 dark:bg-teal-950/40 dark:text-teal-300',
  notary: 'bg-rose-50 text-rose-700 dark:bg-rose-950/40 dark:text-rose-300',
  website: 'bg-indigo-50 text-indigo-700 dark:bg-indigo-950/40 dark:text-indigo-300',
  documents: 'bg-orange-50 text-orange-700 dark:bg-orange-950/40 dark:text-orange-300',
};

const LIFECYCLE_TONE: Record<string, string> = {
  active: 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300',
  trashed: 'bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-300',
  deleted: 'bg-red-50 text-red-700 dark:bg-red-950/40 dark:text-red-300',
};

const SPACE_TYPES = [
  'personal',
  'team',
  'knowledge_base',
  'ai_generated',
  'git_repository',
  'deployment',
  'app_upload',
  'im',
  'rtc',
  'notary',
  'website',
  'documents',
] as const;

const NODE_TYPES = ['file', 'folder', 'shortcut', 'virtual_reference'] as const;
const LIFECYCLE_STATUSES = ['active', 'trashed', 'deleted'] as const;
const CONTENT_TYPE_GROUPS = ['image', 'video', 'audio', 'text', 'document', 'archive', 'binary'] as const;

interface FileFilters {
  keyword: string;
  spaceId: string;
  spaceType: string;
  nodeType: string;
  lifecycleStatus: string;
  contentTypeGroup: string;
}

const DEFAULT_FILTERS: FileFilters = {
  keyword: '',
  spaceId: '',
  spaceType: '',
  nodeType: '',
  lifecycleStatus: 'active',
  contentTypeGroup: '',
};

function resolveTone(map: Record<string, string>, value: string): string {
  return map[value.toLowerCase()] ?? 'bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300';
}

function formatSizeBytes(value: string): string {
  const bytes = Number(value);
  if (!Number.isFinite(bytes) || bytes < 0) {
    return '—';
  }
  if (bytes === 0) {
    return '0 B';
  }
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  let amount = bytes;
  let unit = 0;
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024;
    unit += 1;
  }
  const digits = unit === 0 ? 0 : amount >= 100 ? 0 : amount >= 10 ? 1 : 2;
  return `${amount.toFixed(digits)} ${units[unit]}`;
}

function formatInstant(value: string | undefined, language: string): string {
  if (!value) {
    return '—';
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return '—';
  }
  try {
    return new Intl.DateTimeFormat(language, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  } catch {
    return date.toISOString();
  }
}

function FileTypeGlyph({ group }: { group: string }) {
  const className = 'shrink-0';
  switch (group) {
    case 'image':
      return <ImageIcon aria-hidden="true" className={`${className} text-fuchsia-500`} size={16} />;
    case 'video':
      return <Video aria-hidden="true" className={`${className} text-purple-500`} size={16} />;
    case 'audio':
      return <Music aria-hidden="true" className={`${className} text-cyan-500`} size={16} />;
    case 'text':
    case 'document':
      return <FileText aria-hidden="true" className={`${className} text-blue-500`} size={16} />;
    case 'archive':
      return <Archive aria-hidden="true" className={`${className} text-amber-500`} size={16} />;
    default:
      return <FileIcon aria-hidden="true" className={`${className} text-neutral-400`} size={16} />;
  }
}

/**
 * Tenant-scoped admin file list.
 *
 * The page is the operator-facing read model for the Drive node metadata plane:
 * it never exposes object keys, storage providers, or credentials, and it keeps
 * every int64 wire field (`contentLength`, `version`) as a string. Filters are
 * server-side and pagination uses the opaque continuation cursor returned by the
 * backend, so a large tenant never materializes in the browser.
 */
export function FilesAdminPage({ backendSdkClient, getSession, initialSpaceId }: FilesAdminPageProps) {
  const { t, language } = useTranslation();
  const service = useMemo(
    () => createDriveOperationsAdminService({ backendSdkClient, getSession }),
    [backendSdkClient, getSession],
  );

  const [draft, setDraft] = useState<FileFilters>({
    ...DEFAULT_FILTERS,
    spaceId: initialSpaceId ?? '',
  });
  const [applied, setApplied] = useState<FileFilters>({
    ...DEFAULT_FILTERS,
    spaceId: initialSpaceId ?? '',
  });
  const [files, setFiles] = useState<DriveFileAdminView[]>([]);
  const [page, setPage] = useState(1);
  const [pageCursors, setPageCursors] = useState<Record<number, string | undefined>>({ 1: undefined });
  const [nextPageToken, setNextPageToken] = useState<string | null>(null);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | undefined>();
  const [refreshKey, setRefreshKey] = useState(0);
  const [selectedFile, setSelectedFile] = useState<DriveFileAdminView | null>(null);

  const currentPageToken = pageCursors[page];

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError(undefined);
    service
      .listFiles({
        keyword: applied.keyword || undefined,
        spaceId: applied.spaceId || undefined,
        spaceType: applied.spaceType || undefined,
        nodeType: applied.nodeType || undefined,
        lifecycleStatus: applied.lifecycleStatus || undefined,
        contentTypeGroup: applied.contentTypeGroup || undefined,
        pageSize: PAGE_SIZE,
        pageToken: currentPageToken,
        signal: controller.signal,
      })
      .then((result) => {
        setFiles(result.items);
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
      .catch((loadError) => {
        if (!isDriveRequestCancellationError(loadError)) {
          setError(t('noticeLoadFailed'));
        }
      })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [applied, currentPageToken, page, refreshKey, service, t]);

  const totalPages = nextPageToken
    ? Math.max(page + 1, Math.ceil(total / PAGE_SIZE), 1)
    : Math.max(page, Math.ceil(total / PAGE_SIZE), 1);

  const refresh = () => setRefreshKey((current) => current + 1);

  const resetFilters = () => {
    setDraft(DEFAULT_FILTERS);
    setApplied(DEFAULT_FILTERS);
    setPage(1);
    setPageCursors({ 1: undefined });
    setNextPageToken(null);
  };

  const applyFilters = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setApplied({
      keyword: draft.keyword.trim(),
      spaceId: draft.spaceId.trim(),
      spaceType: draft.spaceType,
      nodeType: draft.nodeType,
      lifecycleStatus: draft.lifecycleStatus,
      contentTypeGroup: draft.contentTypeGroup,
    });
    setPage(1);
    setPageCursors({ 1: undefined });
    setNextPageToken(null);
  };

  return (
    <div className="flex h-full min-h-0 w-full min-w-0 flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <div aria-label={t('filesPageTitle')} className="flex shrink-0 justify-end px-4 pt-4 sm:px-6 sm:pt-6">
        <button type="button" className={GHOST_BUTTON_CLASS} disabled={loading} onClick={refresh}>
          <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
          {t('refresh')}
        </button>
      </div>
      <div className="flex-1 space-y-4 overflow-auto p-4 sm:p-6">
        {error ? (
          <div
            className="flex items-start gap-3 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900/50 dark:bg-red-950/40 dark:text-red-300"
            role="alert"
          >
            <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
            <span className="flex-1">{error}</span>
            <button type="button" className="font-medium underline underline-offset-2" onClick={refresh}>
              {t('refresh')}
            </button>
          </div>
        ) : null}

        <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="files-filters-title">
          <div className={`${CARD_HEADER_CLASS} flex items-center gap-2`}>
            <SlidersHorizontal aria-hidden="true" className="text-blue-600 dark:text-blue-400" size={16} />
            <h2 id="files-filters-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">
              {t('filtersTitle')}
            </h2>
          </div>
          <form className="grid grid-cols-1 gap-4 p-5 sm:grid-cols-2 xl:grid-cols-3" onSubmit={applyFilters}>
            <label className="flex flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">
              {t('filterKeyword')}
              <div className="relative">
                <Search aria-hidden="true" className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400" size={15} />
                <input
                  className={`${INPUT_CLASS} pl-9`}
                  onChange={(event) => setDraft((current) => ({ ...current, keyword: event.target.value }))}
                  placeholder={t('filterKeywordPlaceholder')}
                  value={draft.keyword}
                />
              </div>
            </label>
            <label className="flex flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">
              {t('filterSpaceId')}
              <input
                className={`${INPUT_CLASS} font-mono`}
                onChange={(event) => setDraft((current) => ({ ...current, spaceId: event.target.value }))}
                placeholder={t('filterSpaceIdPlaceholder')}
                value={draft.spaceId}
              />
            </label>
            <label className="flex flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">
              {t('filterSpaceType')}
              <select
                className={SELECT_CLASS}
                onChange={(event) => setDraft((current) => ({ ...current, spaceType: event.target.value }))}
                value={draft.spaceType}
              >
                <option value="">{t('allSpaceTypes')}</option>
                {SPACE_TYPES.map((value) => (
                  <option key={value} value={value}>{t(`spaceType.${value}`)}</option>
                ))}
              </select>
            </label>
            <label className="flex flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">
              {t('filterNodeType')}
              <select
                className={SELECT_CLASS}
                onChange={(event) => setDraft((current) => ({ ...current, nodeType: event.target.value }))}
                value={draft.nodeType}
              >
                <option value="">{t('allNodeTypes')}</option>
                {NODE_TYPES.map((value) => (
                  <option key={value} value={value}>{t(`nodeType.${value}`)}</option>
                ))}
              </select>
            </label>
            <label className="flex flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">
              {t('filterLifecycleStatus')}
              <select
                className={SELECT_CLASS}
                onChange={(event) => setDraft((current) => ({ ...current, lifecycleStatus: event.target.value }))}
                value={draft.lifecycleStatus}
              >
                {LIFECYCLE_STATUSES.map((value) => (
                  <option key={value} value={value}>{t(`fileStatus.${value}`)}</option>
                ))}
              </select>
            </label>
            <label className="flex flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">
              {t('filterContentTypeGroup')}
              <select
                className={SELECT_CLASS}
                onChange={(event) => setDraft((current) => ({ ...current, contentTypeGroup: event.target.value }))}
                value={draft.contentTypeGroup}
              >
                <option value="">{t('allContentTypes')}</option>
                {CONTENT_TYPE_GROUPS.map((value) => (
                  <option key={value} value={value}>{t(`contentTypeGroup.${value}`)}</option>
                ))}
              </select>
            </label>
            <div className="flex items-end gap-2 sm:col-span-2 xl:col-span-3">
              <button type="submit" className={PRIMARY_BUTTON_CLASS}>
                <Search aria-hidden="true" size={15} />
                {t('applyFilters')}
              </button>
              <button type="button" className={GHOST_BUTTON_CLASS} onClick={resetFilters}>
                {t('resetFilters')}
              </button>
            </div>
          </form>
        </section>

        <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="files-list-title">
          <div className={`${CARD_HEADER_CLASS} flex flex-wrap items-center justify-between gap-3`}>
            <div>
              <h2 id="files-list-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">
                {t('filesListTitle')}
              </h2>
              <p className="mt-0.5 text-xs text-neutral-500 dark:text-neutral-400">
                {t('countOf', { filtered: files.length, total })}
              </p>
            </div>
            <span className="rounded-full bg-neutral-100 px-2.5 py-1 text-xs font-medium text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300">
              {t('pageOf', { page, totalPages })}
            </span>
          </div>
          <div className="overflow-x-auto">
            <table className={`${TABLE_CLASS} min-w-[920px]`}>
              <thead className={TABLE_HEAD_CLASS}>
                <tr>
                  <th className="w-[34%] px-5 py-3 text-left">{t('colFileName')}</th>
                  <th className="w-[20%] px-5 py-3 text-left">{t('colSpace')}</th>
                  <th className="w-[10%] px-5 py-3 text-right">{t('colSize')}</th>
                  <th className="w-[14%] px-5 py-3 text-left">{t('colOwner')}</th>
                  <th className="w-[11%] px-5 py-3 text-left">{t('colStatus')}</th>
                  <th className="w-[15%] px-5 py-3 text-left">{t('colUpdated')}</th>
                  <th className="w-[6%] px-5 py-3 text-right">{t('colActions')}</th>
                </tr>
              </thead>
              <tbody>
                {loading ? (
                  <tr className={TABLE_ROW_CLASS}>
                    <td colSpan={7}>
                      <LoadingState label={t('loading')} />
                    </td>
                  </tr>
                ) : files.length === 0 ? (
                  <tr className={TABLE_ROW_CLASS}>
                    <td colSpan={7}>
                      <EmptyState
                        description={t('filesPageDescription')}
                        icon={FolderOpen}
                        title={t('filesEmpty')}
                      />
                    </td>
                  </tr>
                ) : (
                  files.map((file) => (
                    <tr key={file.id} className={TABLE_ROW_CLASS}>
                      <td className="px-5 py-3.5 align-top">
                        <div className="flex min-w-0 items-start gap-3">
                          <div className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-neutral-100 dark:bg-neutral-800">
                            {file.nodeType === 'folder' ? (
                              <FolderOpen aria-hidden="true" className="text-amber-500" size={16} />
                            ) : file.nodeType === 'shortcut' || file.nodeType === 'virtual_reference' ? (
                              <Link2 aria-hidden="true" className="text-neutral-400" size={16} />
                            ) : (
                              <FileTypeGlyph group={file.contentTypeGroup ?? 'binary'} />
                            )}
                          </div>
                          <div className="min-w-0">
                            <button
                              className="block max-w-full truncate text-left text-sm font-semibold text-neutral-900 hover:text-blue-600 dark:text-neutral-100 dark:hover:text-blue-400"
                              onClick={() => setSelectedFile(file)}
                              type="button"
                            >
                              {file.nodeName}
                            </button>
                            <div className="mt-0.5 truncate font-mono text-[11px] text-neutral-400 dark:text-neutral-500">
                              {file.filePath}
                            </div>
                          </div>
                        </div>
                      </td>
                      <td className="px-5 py-3.5 align-top">
                        <div className="min-w-0">
                          <div className="truncate text-sm text-neutral-800 dark:text-neutral-200">{file.spaceName}</div>
                          <div className="mt-1 flex flex-wrap items-center gap-1.5">
                            <span className={`${BADGE_BASE_CLASS} ${resolveTone(SPACE_TYPE_TONE, file.spaceType)}`}>
                              {t(`spaceType.${file.spaceType}`)}
                            </span>
                            <span className="truncate font-mono text-[10px] text-neutral-400 dark:text-neutral-500">{file.spaceId}</span>
                          </div>
                        </div>
                      </td>
                      <td className="px-5 py-3.5 text-right align-top font-mono text-xs text-neutral-600 dark:text-neutral-300">
                        {file.nodeType === 'file' ? formatSizeBytes(file.contentLength) : '—'}
                      </td>
                      <td className="px-5 py-3.5 align-top">
                        <div className="min-w-0">
                          <div className="truncate text-xs font-medium text-neutral-700 dark:text-neutral-200">
                            {t(`ownerType.${file.ownerSubjectType}`)}
                          </div>
                          <div className="mt-0.5 truncate font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
                            {file.ownerSubjectId}
                          </div>
                        </div>
                      </td>
                      <td className="px-5 py-3.5 align-top">
                        <span className={`${BADGE_BASE_CLASS} ${resolveTone(LIFECYCLE_TONE, file.lifecycleStatus)}`}>
                          <span aria-hidden="true" className="h-1.5 w-1.5 rounded-full bg-current" />
                          {t(`fileStatus.${file.lifecycleStatus}`)}
                        </span>
                      </td>
                      <td className="px-5 py-3.5 align-top text-xs text-neutral-600 dark:text-neutral-300">
                        <time dateTime={file.updatedAt}>{formatInstant(file.updatedAt, language)}</time>
                      </td>
                      <td className="px-5 py-3.5 text-right align-top">
                        <button
                          aria-label={t('fileDetailTitle')}
                          className={GHOST_BUTTON_CLASS}
                          onClick={() => setSelectedFile(file)}
                          title={t('fileDetailTitle')}
                          type="button"
                        >
                          <ExternalLink aria-hidden="true" size={14} />
                        </button>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
          <PaginationBar
            loading={loading}
            nextDisabled={!nextPageToken}
            nextLabel={t('nextPage')}
            onNext={() => setPage((current) => current + 1)}
            onPrevious={() => setPage((current) => Math.max(1, current - 1))}
            pageLabel={t('pageOf', { page, totalPages })}
            previousDisabled={page <= 1}
            previousLabel={t('previousPage')}
          />
        </section>
      </div>
      {selectedFile ? (
        <FileDetailDialog file={selectedFile} language={language} onClose={() => setSelectedFile(null)} />
      ) : null}
    </div>
  );
}

function FileDetailDialog({
  file,
  language,
  onClose,
}: {
  file: DriveFileAdminView;
  language: string;
  onClose: () => void;
}) {
  const { t } = useTranslation();

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [onClose]);

  const rows: ReadonlyArray<{ label: string; value: string }> = [
    { label: t('fieldId'), value: file.id },
    { label: t('fieldPath'), value: file.filePath },
    { label: t('fieldSpace'), value: `${file.spaceName} (${file.spaceId})` },
    { label: t('fieldSpaceType'), value: t(`spaceType.${file.spaceType}`) },
    { label: t('fieldNodeType'), value: t(`nodeType.${file.nodeType}`) },
    { label: t('fieldContentType'), value: file.contentType ?? t('fieldUnknown') },
    { label: t('fieldSize'), value: file.nodeType === 'file' ? formatSizeBytes(file.contentLength) : '—' },
    { label: t('fieldOwner'), value: `${t(`ownerType.${file.ownerSubjectType}`)} · ${file.ownerSubjectId}` },
    { label: t('fieldLifecycle'), value: t(`fileStatus.${file.lifecycleStatus}`) },
    { label: t('fieldVersion'), value: file.version },
    { label: t('fieldCreatedBy'), value: file.createdBy },
    { label: t('fieldUpdatedBy'), value: file.updatedBy },
    { label: t('fieldCreatedAt'), value: formatInstant(file.createdAt, language) },
    { label: t('fieldUpdatedAt'), value: formatInstant(file.updatedAt, language) },
  ];

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/55 p-4"
      onPointerDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
      role="presentation"
    >
      <div
        aria-labelledby="file-detail-dialog-title"
        aria-modal="true"
        className="flex w-full max-w-2xl flex-col overflow-hidden rounded-lg border border-neutral-200 bg-white shadow-2xl dark:border-neutral-700 dark:bg-neutral-900"
        role="dialog"
      >
        <div className="flex items-center justify-between border-b border-neutral-200 px-5 py-4 dark:border-neutral-700">
          <div className="min-w-0">
            <h2 className="truncate text-base font-semibold text-neutral-900 dark:text-white" id="file-detail-dialog-title">
              {t('fileDetailTitle')}
            </h2>
            <p className="mt-1 truncate text-sm text-neutral-500 dark:text-neutral-400">{file.nodeName}</p>
          </div>
          <button
            aria-label={t('close')}
            className={GHOST_BUTTON_CLASS}
            onClick={onClose}
            type="button"
          >
            <X aria-hidden="true" size={16} />
          </button>
        </div>
        <div className="grid max-h-[70vh] grid-cols-1 gap-x-6 gap-y-3 overflow-y-auto p-5 md:grid-cols-2">
          {rows.map((row) => (
            <div className="min-w-0" key={row.label}>
              <div className="text-xs font-medium uppercase tracking-wide text-neutral-400 dark:text-neutral-500">
                {row.label}
              </div>
              <div className="mt-0.5 break-words text-sm text-neutral-800 dark:text-neutral-100">{row.value}</div>
            </div>
          ))}
        </div>
        <div className="flex justify-end gap-3 border-t border-neutral-200 px-5 py-4 dark:border-neutral-700">
          <button className={GHOST_BUTTON_CLASS} onClick={onClose} type="button">{t('close')}</button>
        </div>
      </div>
    </div>
  );
}
