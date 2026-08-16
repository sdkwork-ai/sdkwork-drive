import React, { useCallback, useEffect, useMemo, useState, type FormEvent } from 'react';
import {
  Pencil,
  Plus,
  RefreshCw,
  Search,
  Tags,
  Trash2,
} from 'lucide-react';
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
import { OperationsConfirmDialog } from '../components/OperationsConfirmDialog';
import { EmptyState, LoadingState, NoticeBanner, OperationsPageHeader } from '../components/OperationsAdminPrimitives';
import { createDriveOperationsAdminService } from '../services/driveOperationsAdminService';
import type { LabelView } from '../types/driveOperationsAdminTypes';
import {
  CARD_CLASS,
  CARD_HEADER_CLASS,
  GHOST_BUTTON_CLASS,
  INPUT_CLASS,
  PRIMARY_BUTTON_CLASS,
  SECONDARY_BUTTON_CLASS,
  TABLE_CLASS,
  TABLE_HEAD_CLASS,
  TABLE_ROW_CLASS,
} from '../utils/uiPrimitives';
import { useTranslation } from '../hooks/useTranslation';

interface LabelsAdminPageProps {
  backendSdkClient: DriveBackendSdkClient;
  getSession: () => SessionSnapshot;
}

export function LabelsAdminPage({ backendSdkClient, getSession }: LabelsAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo(
    () => createDriveOperationsAdminService({ backendSdkClient, getSession }),
    [backendSdkClient, getSession],
  );
  const [labels, setLabels] = useState<LabelView[]>([]);
  const [nextPageToken, setNextPageToken] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [pending, setPending] = useState(false);
  const [notice, setNotice] = useState<{ type: 'error' | 'success'; message: string } | undefined>();
  const [search, setSearch] = useState('');
  const [id, setId] = useState('');
  const [labelKey, setLabelKey] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [color, setColor] = useState('#3366FF');
  const [description, setDescription] = useState('');
  const [createOpen, setCreateOpen] = useState(false);
  const [editingLabelId, setEditingLabelId] = useState<string | null>(null);
  const [editDisplayName, setEditDisplayName] = useState('');
  const [editColor, setEditColor] = useState('');
  const [editDescription, setEditDescription] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<LabelView | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  const loadPage = useCallback(async ({
    append = false,
    pageToken,
    signal,
  }: {
    append?: boolean;
    pageToken?: string;
    signal?: AbortSignal;
  } = {}) => {
    append ? setLoadingMore(true) : setLoading(true);
    try {
      const result = await service.listLabels({ lifecycleStatus: 'active', pageSize: 50, pageToken, signal });
      setLabels((current) => (append ? [...current, ...result.items] : result.items));
      setNextPageToken(result.pageInfo?.nextCursor ?? result.nextPageToken ?? null);
    } catch (err) {
      if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', message: t('noticeLoadFailed') });
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  }, [service, t]);

  useEffect(() => {
    const controller = new AbortController();
    void loadPage({ signal: controller.signal });
    return () => controller.abort();
  }, [loadPage, refreshKey]);

  const createLabel = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setPending(true);
    setNotice(undefined);
    try {
      await service.createLabel({
        id: id.trim(),
        labelKey: labelKey.trim(),
        displayName: displayName.trim(),
        color: color.trim() || undefined,
        description: description.trim() || undefined,
      });
      setId('');
      setLabelKey('');
      setDisplayName('');
      setColor('#3366FF');
      setDescription('');
      setCreateOpen(false);
      await loadPage();
      setNotice({ type: 'success', message: t('labelCreated') });
    } catch (err) {
      if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', message: t('noticeOperationFailed') });
    } finally {
      setPending(false);
    }
  };

  const beginEdit = (label: LabelView) => {
    setEditingLabelId(label.id);
    setEditDisplayName(label.displayName);
    setEditColor(label.color ?? '');
    setEditDescription(label.description ?? '');
  };

  const cancelEdit = () => {
    setEditingLabelId(null);
    setEditDisplayName('');
    setEditColor('');
    setEditDescription('');
  };

  const saveEdit = async () => {
    if (!editingLabelId) return;
    setPending(true);
    setNotice(undefined);
    try {
      await service.updateLabel(editingLabelId, {
        displayName: editDisplayName.trim() || undefined,
        color: editColor.trim() || undefined,
        description: editDescription.trim() || undefined,
      });
      cancelEdit();
      await loadPage();
      setNotice({ type: 'success', message: t('labelUpdated') });
    } catch (err) {
      if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', message: t('noticeOperationFailed') });
    } finally {
      setPending(false);
    }
  };

  const deleteLabel = async () => {
    if (!deleteTarget) return;
    const labelId = deleteTarget.id;
    setDeleteTarget(null);
    setPending(true);
    setNotice(undefined);
    try {
      await service.deleteLabel(labelId);
      if (editingLabelId === labelId) cancelEdit();
      await loadPage();
      setNotice({ type: 'success', message: t('labelDeleted') });
    } catch (err) {
      if (!isDriveRequestCancellationError(err)) setNotice({ type: 'error', message: t('noticeOperationFailed') });
    } finally {
      setPending(false);
    }
  };

  const normalizedSearch = search.trim().toLowerCase();
  const filteredLabels = normalizedSearch
    ? labels.filter((label) => [label.id, label.labelKey, label.displayName, label.description ?? ''].some((value) => value.toLowerCase().includes(normalizedSearch)))
    : labels;

  return (
    <main className="flex h-full min-h-0 w-full min-w-0 flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <OperationsPageHeader
        icon={Tags}
        title={t('labelsPageTitle')}
        description={t('labelsPageDescription')}
        toneClassName="bg-violet-50 text-violet-700 dark:bg-violet-950/40 dark:text-violet-300"
        actions={(
          <>
            <button type="button" className={PRIMARY_BUTTON_CLASS} disabled={pending} onClick={() => setCreateOpen(true)}>
              <Plus aria-hidden="true" size={15} />
              {t('createLabelAction')}
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

        <section className={`${CARD_CLASS} overflow-hidden`} aria-labelledby="labels-list-title">
          <div className={`${CARD_HEADER_CLASS} flex flex-wrap items-center justify-between gap-3`}>
            <div>
              <h2 id="labels-list-title" className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">{t('labelsListTitle')}</h2>
              <p className="mt-0.5 text-xs text-neutral-500 dark:text-neutral-400">{t('countOf', { filtered: filteredLabels.length, total: labels.length })}</p>
            </div>
            <label className="relative min-w-[220px] max-w-sm flex-1 sm:flex-none">
              <span className="sr-only">{t('searchLabel')}</span>
              <Search aria-hidden="true" className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400" size={15} />
              <input className={`${INPUT_CLASS} pl-9`} value={search} onChange={(event) => setSearch(event.target.value)} placeholder={t('labelsSearchPlaceholder')} />
            </label>
          </div>
          {loading ? <LoadingState label={t('loading')} /> : filteredLabels.length === 0 ? (
            <EmptyState title={t('labelsEmpty')} description={t('labelsPageDescription')} icon={Tags} />
          ) : (
            <div className="overflow-x-auto">
              <table className={`${TABLE_CLASS} min-w-[960px]`}>
                <thead><tr className={TABLE_HEAD_CLASS}>
                  <th className="px-5 py-3">{t('colLabelKey')}</th>
                  <th className="px-5 py-3">{t('colDisplayName')}</th>
                  <th className="px-5 py-3">{t('colColor')}</th>
                  <th className="px-5 py-3">{t('colDescription')}</th>
                  <th className="px-5 py-3 text-right">{t('colActions')}</th>
                </tr></thead>
                <tbody>
                  {filteredLabels.map((label) => {
                    return (
                      <tr key={label.id} className={TABLE_ROW_CLASS}>
                        <td className="px-5 py-3"><div className="font-mono text-xs font-medium text-neutral-800 dark:text-neutral-100">{label.labelKey}</div><div className="mt-0.5 font-mono text-[10px] text-neutral-400">{label.id}</div></td>
                        <td className="px-5 py-3"><span className="text-sm font-medium text-neutral-800 dark:text-neutral-100">{label.displayName}</span></td>
                        <td className="px-5 py-3"><span className="inline-flex items-center gap-2 font-mono text-xs text-neutral-600 dark:text-neutral-300"><span className="h-4 w-4 rounded border border-black/10 shadow-sm" style={{ backgroundColor: label.color || 'transparent' }} /><span>{label.color || '--'}</span></span></td>
                        <td className="max-w-sm px-5 py-3"><span className="line-clamp-2 text-xs leading-5 text-neutral-600 dark:text-neutral-400">{label.description || '--'}</span></td>
                        <td className="px-5 py-3 text-right">
                          <div className="flex items-center justify-end gap-1">
                            <button type="button" className={GHOST_BUTTON_CLASS} aria-label={t('editLabelAction')} title={t('editLabelAction')} disabled={pending} onClick={() => beginEdit(label)}><Pencil aria-hidden="true" size={15} />{t('editLabelAction')}</button>
                            <button type="button" className={`${GHOST_BUTTON_CLASS} text-red-600 hover:bg-red-50 hover:text-red-700 dark:text-red-400 dark:hover:bg-red-950/30`} aria-label={t('deleteLabelAction')} title={t('deleteLabelAction')} disabled={pending} onClick={() => setDeleteTarget(label)}><Trash2 aria-hidden="true" size={15} /></button>
                          </div>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
          {nextPageToken ? (
            <div className="flex justify-center border-t border-neutral-200 px-5 py-3 dark:border-neutral-800">
              <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={loadingMore || pending} onClick={() => void loadPage({ append: true, pageToken: nextPageToken })}>
                {loadingMore ? <RefreshCw aria-hidden="true" className="animate-spin" size={15} /> : null}
                {t('loadMoreLabels')}
              </button>
            </div>
          ) : null}
        </section>
      </div>

      <Drawer open={createOpen || editingLabelId !== null} onOpenChange={(open) => { if (!open && !pending) { setCreateOpen(false); cancelEdit(); } }}>
        <DrawerContent size="md">
          <form className="flex min-h-0 flex-1 flex-col" onSubmit={(event) => { if (editingLabelId) { event.preventDefault(); void saveEdit(); } else { void createLabel(event); } }}>
            <DrawerHeader>
              <DrawerTitle>{editingLabelId ? t('editLabelAction') : t('createLabelTitle')}</DrawerTitle>
              <DrawerDescription>{t('labelsPageDescription')}</DrawerDescription>
            </DrawerHeader>
            <DrawerBody className="grid content-start gap-4">
              {editingLabelId ? (
                <>
                  <LabelField label={t('labelNameLabel')}><input autoFocus className={INPUT_CLASS} value={editDisplayName} onChange={(event) => setEditDisplayName(event.target.value)} /></LabelField>
                  <LabelField label={t('labelColorLabel')}><div className="flex items-center gap-2"><input type="color" value={editColor || '#3366FF'} onChange={(event) => setEditColor(event.target.value)} className="h-9 w-11 shrink-0 cursor-pointer rounded-md border border-neutral-300 bg-white p-1 dark:border-neutral-600 dark:bg-neutral-900" /><input className={`${INPUT_CLASS} font-mono text-xs`} value={editColor} onChange={(event) => setEditColor(event.target.value)} placeholder={t('labelColorPlaceholder')} /></div></LabelField>
                  <LabelField label={t('labelDescriptionLabel')}><input className={INPUT_CLASS} value={editDescription} onChange={(event) => setEditDescription(event.target.value)} placeholder={t('labelDescriptionPlaceholder')} /></LabelField>
                </>
              ) : (
                <>
                  <LabelField label={t('labelIdLabel')}><input autoFocus className={`${INPUT_CLASS} font-mono text-xs`} value={id} onChange={(event) => setId(event.target.value)} placeholder={t('labelIdPlaceholder')} /></LabelField>
                  <LabelField label={t('labelKeyLabel')}><input className={`${INPUT_CLASS} font-mono text-xs`} value={labelKey} onChange={(event) => setLabelKey(event.target.value)} placeholder={t('labelKeyPlaceholder')} /></LabelField>
                  <LabelField label={t('labelNameLabel')}><input className={INPUT_CLASS} value={displayName} onChange={(event) => setDisplayName(event.target.value)} placeholder={t('labelNamePlaceholder')} /></LabelField>
                  <LabelField label={t('labelColorLabel')}><div className="flex items-center gap-2"><input type="color" value={color} onChange={(event) => setColor(event.target.value)} className="h-9 w-11 shrink-0 cursor-pointer rounded-md border border-neutral-300 bg-white p-1 dark:border-neutral-600 dark:bg-neutral-900" /><input className={`${INPUT_CLASS} font-mono text-xs`} value={color} onChange={(event) => setColor(event.target.value)} placeholder={t('labelColorPlaceholder')} /></div></LabelField>
                  <LabelField label={t('labelDescriptionLabel')}><input className={INPUT_CLASS} value={description} onChange={(event) => setDescription(event.target.value)} placeholder={t('labelDescriptionPlaceholder')} /></LabelField>
                </>
              )}
            </DrawerBody>
            <DrawerFooter>
              <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={pending} onClick={() => { setCreateOpen(false); cancelEdit(); }}>{t('cancel')}</button>
              <button type="submit" className={PRIMARY_BUTTON_CLASS} disabled={pending || (editingLabelId ? !editDisplayName.trim() : !id.trim() || !labelKey.trim() || !displayName.trim())}>{pending ? t('saving') : editingLabelId ? t('saveLabelAction') : t('createLabelAction')}</button>
            </DrawerFooter>
          </form>
        </DrawerContent>
      </Drawer>

      <OperationsConfirmDialog
        open={deleteTarget !== null}
        title={t('confirmDeleteLabelTitle')}
        message={t('confirmDeleteLabelMessage', { name: deleteTarget?.displayName ?? '' })}
        confirmLabel={t('confirmDeleteLabelAction')}
        cancelLabel={t('cancel')}
        variant="danger"
        onCancel={() => setDeleteTarget(null)}
        onConfirm={() => void deleteLabel()}
      />
    </main>
  );
}

function LabelField({ children, label }: { children: React.ReactNode; label: string }) {
  return <label className="flex min-w-0 flex-col gap-1.5 text-xs font-medium text-neutral-600 dark:text-neutral-400">{label}{children}</label>;
}
