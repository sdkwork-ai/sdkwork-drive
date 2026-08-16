import React, { useCallback, useRef, useState } from 'react';
import type { StorageProviderAdminService } from '../services/storageProviderAdminService';
import type { StorageProviderObjectView, StorageProviderView } from '../types/storageProviderAdminTypes';
import { formatDriveBytes } from 'sdkwork-drive-pc-commons';
import { formatMutationError } from '../utils/mutationError';
import { useTranslation } from '../hooks/useTranslation';
import { ConfirmDialog } from './ConfirmDialog';
import { PRIMARY_BUTTON_CLASS, SECONDARY_BUTTON_CLASS, INPUT_CLASS } from '../utils/uiPrimitives';

/** 上传大小上限（与服务端对象内容写入上限一致）。 */
const MAX_UPLOAD_BYTES = 8 * 1024 * 1024;

interface StorageObjectBrowserProps {
  provider: StorageProviderView;
  service: StorageProviderAdminService;
}

type PromptKind = 'newFolder' | 'rename';

interface ObjectPrompt {
  kind: PromptKind;
  objectKey?: string;
  initialValue: string;
}

function fileNameOf(key: string): string {
  const segments = key.split('/').filter(Boolean);
  return segments.at(-1) ?? key;
}

function parentPrefixOf(key: string): string {
  const segments = key.split('/').filter(Boolean);
  segments.pop();
  return segments.length > 0 ? `${segments.join('/')}/` : '';
}

export function StorageObjectBrowser({ provider, service }: StorageObjectBrowserProps) {
  const { t } = useTranslation();
  const [objects, setObjects] = useState<StorageProviderObjectView[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentPrefix, setCurrentPrefix] = useState('');
  const [pageToken, setPageToken] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);
  const [prompt, setPrompt] = useState<ObjectPrompt | null>(null);
  const [promptValue, setPromptValue] = useState('');
  const [promptBusy, setPromptBusy] = useState(false);
  const [uploading, setUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  /** 请求序号：loadMore 与导航并发时丢弃过期响应（防止旧前缀分页混入新列表）。 */
  const loadSeqRef = useRef(0);

  const formatSize = formatDriveBytes;

  const loadObjects = useCallback(async (prefix: string, token?: string) => {
    const seq = ++loadSeqRef.current;
    setLoading(true);
    setError(null);
    try {
      const result = await service.listObjects(provider.id, {
        prefix,
        pageToken: token,
      });
      if (seq !== loadSeqRef.current) {
        return;
      }
      if (token) {
        setObjects((prev) => [...prev, ...result.items]);
      } else {
        setObjects(result.items);
      }
      setPageToken(result.nextPageToken || null);
      setHasMore(result.hasMore);
      setCurrentPrefix(prefix);
    } catch (err) {
      if (seq !== loadSeqRef.current) {
        return;
      }
      setError(formatMutationError(err, t('errorLoadObjects')));
    } finally {
      if (seq === loadSeqRef.current) {
        setLoading(false);
      }
    }
  }, [provider.id, service, t]);

  const navigateToFolder = (prefix: string) => {
    loadObjects(prefix);
  };

  const navigateUp = () => {
    loadObjects(parentPrefixOf(currentPrefix));
  };

  const openPrompt = (next: ObjectPrompt) => {
    setPromptValue(next.initialValue);
    setPrompt(next);
  };

  const submitPrompt = async () => {
    if (!prompt) return;
    const value = promptValue.trim();
    if (!value) return;
    setPromptBusy(true);
    setError(null);
    try {
      if (prompt.kind === 'newFolder') {
        await service.writeObjectContent(provider.id, `${currentPrefix}${value}/`, {
          content: '',
        });
      } else if (prompt.objectKey) {
        const destination = `${parentPrefixOf(prompt.objectKey)}${value}`;
        if (destination === prompt.objectKey) {
          setPrompt(null);
          return;
        }
        await service.renameObject(provider.id, prompt.objectKey, destination);
      }
      setPrompt(null);
      await loadObjects(currentPrefix);
    } catch (err) {
      setError(formatMutationError(err, prompt.kind === 'newFolder' ? t('newFolderError') : t('renameError')));
    } finally {
      setPromptBusy(false);
    }
  };

  const uploadFile = async (file: File) => {
    if (file.size > MAX_UPLOAD_BYTES) {
      setError(t('fileTooLarge'));
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
      return;
    }
    setUploading(true);
    setError(null);
    try {
      const base64 = await readFileAsBase64(file);
      await service.writeObjectContent(provider.id, `${currentPrefix}${file.name}`, {
        content: base64,
        encoding: 'base64',
        ...(file.type ? { contentType: file.type } : {}),
      });
      await loadObjects(currentPrefix);
    } catch (err) {
      setError(formatMutationError(err, t('uploadError')));
    } finally {
      setUploading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    }
  };

  const downloadObject = async (object: StorageProviderObjectView) => {
    setError(null);
    try {
      const content = await service.readObjectContent(provider.id, object.key);
      const bytes = base64ToBytes(content.content);
      const arrayBuffer = new ArrayBuffer(bytes.byteLength);
      new Uint8Array(arrayBuffer).set(bytes);
      const blob = new Blob(
        [arrayBuffer],
        { type: content.contentType ?? 'application/octet-stream' },
      );
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement('a');
      anchor.href = url;
      anchor.download = fileNameOf(object.key);
      document.body.appendChild(anchor);
      anchor.click();
      anchor.remove();
      // 延迟释放：部分浏览器在同步 revoke 后会中断下载。
      window.setTimeout(() => URL.revokeObjectURL(url), 60_000);
    } catch (err) {
      setError(formatMutationError(err, t('downloadError')));
    }
  };

  const deleteObject = useCallback(async (key: string) => {
    setDeleteTarget(null);
    setLoading(true);
    setError(null);
    try {
      await service.deleteObject(provider.id, key);
      await loadObjects(currentPrefix);
    } catch (err) {
      setError(formatMutationError(err, t('errorDeleteObject')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, currentPrefix, service, loadObjects, t]);

  const breadcrumbSegments = currentPrefix.split('/').filter(Boolean);

  return (
    <>
    <div className="border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-[#171717]">
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <button type="button" onClick={() => loadObjects('')} disabled={loading || uploading} className="rounded border px-2 py-1 text-xs">
          {t('root')}
        </button>
        {currentPrefix && (
          <button type="button" onClick={navigateUp} disabled={loading || uploading} className="rounded border px-2 py-1 text-xs">
            {t('up')}
          </button>
        )}
        <span className="font-mono text-xs text-neutral-500">
          {breadcrumbSegments.map((segment, index) => (
            <React.Fragment key={`${segment}-${index}`}>
              <button
                type="button"
                className="text-blue-600 hover:underline"
                disabled={loading}
                onClick={() => loadObjects(breadcrumbSegments.slice(0, index + 1).join('/') + '/')}
              >
                {segment}
              </button>
              {index < breadcrumbSegments.length - 1 ? <span className="text-neutral-400">/</span> : null}
            </React.Fragment>
          ))}
        </span>
        <span className="flex-1" />
        <button type="button" onClick={() => loadObjects(currentPrefix)} disabled={loading || uploading} className="rounded border px-2 py-1 text-xs">
          {t('refresh')}
        </button>
        <button
          type="button"
          onClick={() => openPrompt({ kind: 'newFolder', initialValue: '' })}
          disabled={loading || uploading}
          className="rounded border px-2 py-1 text-xs"
        >
          {t('newFolder')}
        </button>
        <label className="cursor-pointer rounded border px-2 py-1 text-xs disabled:opacity-60" aria-disabled={uploading}>
          {uploading ? t('uploading') : t('upload')}
          <input
            ref={fileInputRef}
            className="hidden"
            type="file"
            disabled={uploading}
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (file) void uploadFile(file);
            }}
          />
        </label>
      </div>

      {error && <div className="mb-3 rounded bg-red-50 px-3 py-2 text-xs text-red-700 dark:bg-red-950/20 dark:text-red-300">{error}</div>}

      {objects.length === 0 && !loading ? (
        <div className="py-8 text-center text-xs text-neutral-400">{t('empty')}</div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-xs">
            <thead>
              <tr className="border-b text-left text-neutral-500">
                <th className="py-2 pr-4">{t('nameHeader')}</th>
                <th className="py-2 pr-4">{t('sizeHeader')}</th>
                <th className="py-2 pr-4">{t('modifiedHeader')}</th>
                <th className="py-2">{t('actHeader')}</th>
              </tr>
            </thead>
            <tbody>
              {objects.map((obj) => (
                <tr key={obj.key} className="border-b border-neutral-100 dark:border-neutral-800">
                  <td className="py-2 pr-4 font-mono">
                    {obj.isFolder ? (
                      <button type="button" onClick={() => navigateToFolder(obj.key)} className="text-blue-600 hover:underline">
                        {fileNameOf(obj.key)}/
                      </button>
                    ) : (
                      fileNameOf(obj.key)
                    )}
                  </td>
                  <td className="py-2 pr-4">{obj.isFolder ? '-' : formatSize(obj.sizeBytes)}</td>
                  <td className="py-2 pr-4 text-neutral-400">{obj.lastModified ?? '-'}</td>
                  <td className="py-2">
                    {!obj.isFolder && obj.sizeBytes > MAX_UPLOAD_BYTES && (
                      <span className="mr-2 text-xs text-neutral-400" title={t('fileTooLarge')}>
                        {t('fileTooLarge')}
                      </span>
                    )}
                    {!obj.isFolder && obj.sizeBytes <= MAX_UPLOAD_BYTES && (
                      <button type="button" onClick={() => void downloadObject(obj)} disabled={loading} className="mr-2 text-blue-600 hover:underline">
                        {t('download')}
                      </button>
                    )}
                    {!obj.isFolder && (
                      <button
                        type="button"
                        onClick={() => openPrompt({ kind: 'rename', objectKey: obj.key, initialValue: fileNameOf(obj.key) })}
                        disabled={loading}
                        className="mr-2 text-neutral-600 hover:underline dark:text-neutral-300"
                      >
                        {t('rename')}
                      </button>
                    )}
                    {!obj.isFolder && (
                      <button type="button" onClick={() => setDeleteTarget(obj.key)} disabled={loading} className="text-red-600 hover:underline">
                        {t('del')}
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {hasMore && (
        <button type="button" onClick={() => loadObjects(currentPrefix, pageToken ?? undefined)} disabled={loading} className="mt-3 text-xs text-blue-600 hover:underline">
          {t('loadMore')}
        </button>
      )}
    </div>
    {prompt ? (
      <div
        className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
        role="presentation"
        onPointerDown={(event) => {
          if (event.target === event.currentTarget && !promptBusy) {
            setPrompt(null);
          }
        }}
      >
        <div
          aria-modal="true"
          className="w-full max-w-sm rounded-lg border border-neutral-200 bg-white p-4 shadow-xl dark:border-neutral-800 dark:bg-[#1b1b1b]"
          role="dialog"
          aria-labelledby="storage-object-prompt-title"
        >
          <h3 className="mb-3 text-sm font-medium text-neutral-800 dark:text-neutral-100" id="storage-object-prompt-title">
            {prompt.kind === 'newFolder' ? t('newFolderDialogTitle') : t('renameDialogTitle')}
          </h3>
          <input
            autoFocus
            className={INPUT_CLASS}
            value={promptValue}
            placeholder={prompt.kind === 'newFolder' ? t('folderNamePlaceholder') : t('newNamePlaceholder')}
            onChange={(event) => setPromptValue(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter' && !promptBusy) void submitPrompt();
              if (event.key === 'Escape' && !promptBusy) setPrompt(null);
            }}
          />
          <div className="mt-4 flex justify-end gap-2">
            <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={promptBusy} onClick={() => setPrompt(null)}>
              {t('cancel')}
            </button>
            <button type="button" className={PRIMARY_BUTTON_CLASS} disabled={promptBusy || !promptValue.trim()} onClick={() => void submitPrompt()}>
              {promptBusy ? t('saving') : t('confirm')}
            </button>
          </div>
        </div>
      </div>
    ) : null}
    <ConfirmDialog
      busy={loading}
      confirmLabel={t('del')}
      message={t('deleteObjectConfirm', { key: deleteTarget ?? '' })}
      onCancel={() => setDeleteTarget(null)}
      onConfirm={() => { if (deleteTarget) void deleteObject(deleteTarget); }}
      open={deleteTarget !== null}
      title={t('del')}
      variant="danger"
    />
    </>
  );
}

function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error ?? new Error('File read failed.'));
    reader.onload = () => {
      const result = typeof reader.result === 'string' ? reader.result : '';
      const comma = result.indexOf(',');
      resolve(comma >= 0 ? result.slice(comma + 1) : result);
    };
    reader.readAsDataURL(file);
  });
}

function base64ToBytes(base64: string): Uint8Array {
  const binary = window.atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}
