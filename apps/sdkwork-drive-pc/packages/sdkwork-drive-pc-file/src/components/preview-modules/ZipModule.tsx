import React, { useEffect, useMemo, useRef, useState } from 'react';
import { Archive, ExternalLink, Info } from 'lucide-react';
import { formatDriveBytes, isDriveConflictError, useTranslation } from 'sdkwork-drive-pc-commons';
import type { DriveFile } from 'sdkwork-drive-pc-types';
import { isDriveAbortError, type DriveArchiveEntry, type DriveFileService } from 'sdkwork-drive-pc-core';

interface ZipModuleProps {
  file: DriveFile;
  fileService: DriveFileService;
  previewUrl: string | null;
  previewError: string | null;
  loading: boolean;
  triggerFeedback: (text: string, type?: 'success' | 'info' | 'error') => void;
  onExtracted?: () => void | Promise<void>;
  isReadOnly?: boolean;
}

export function ZipModule({
  file,
  fileService,
  previewUrl,
  previewError,
  loading,
  triggerFeedback,
  onExtracted,
  isReadOnly = false,
}: ZipModuleProps) {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<DriveArchiveEntry[]>([]);
  const [entriesLoading, setEntriesLoading] = useState(false);
  const [entriesError, setEntriesError] = useState<string | null>(null);
  const [extracting, setExtracting] = useState(false);
  const extractionAbortControllerRef = useRef<AbortController | null>(null);

  useEffect(() => {
    let active = true;
    const archiveListAbortController = new AbortController();
    setEntries([]);
    setEntriesError(null);
    setEntriesLoading(true);

    fileService.listArchiveEntries(file, {
      signal: archiveListAbortController.signal,
    })
      .then((archiveEntries) => {
        if (active) {
          setEntries(archiveEntries);
        }
      })
      .catch((err: any) => {
        if (isDriveAbortError(err)) {
          return;
        }
        if (active) {
          setEntriesError(err?.message || t('previewModules.archiveLoadFailed'));
        }
      })
      .finally(() => {
        if (active) {
          setEntriesLoading(false);
        }
      });

    return () => {
      active = false;
      archiveListAbortController.abort();
    };
  }, [file.id, file.updatedAt, fileService]);

  useEffect(() => {
    return () => {
      extractionAbortControllerRef.current?.abort();
      extractionAbortControllerRef.current = null;
    };
  }, [file.id, file.updatedAt]);

  const archiveSummary = useMemo(() => {
    const fileCount = entries.filter((entry) => !entry.isDirectory).length;
    const folderCount = entries.filter((entry) => entry.isDirectory).length;
    const totalBytes = entries
      .filter((entry) => !entry.isDirectory)
      .reduce((total, entry) => total + entry.uncompressedSizeBytes, 0);

    return { fileCount, folderCount, totalBytes };
  }, [entries]);

  const formatSize = formatDriveBytes;

  const handleExtract = async () => {
    extractionAbortControllerRef.current?.abort();
    const extractionAbortController = new AbortController();
    extractionAbortControllerRef.current = extractionAbortController;
    setExtracting(true);
    try {
      const extracted = await fileService.extractArchiveEntries(file, undefined, {
        signal: extractionAbortController.signal,
      });
      triggerFeedback(
        extracted.length > 0
          ? t('previewModules.archiveExtractedCount', { count: extracted.length })
          : t('previewModules.archiveExtractCompleted'),
        'success',
      );
      await onExtracted?.();
    } catch (err: any) {
      if (isDriveAbortError(err)) {
        return;
      }
      triggerFeedback(
        isDriveConflictError(err)
          ? t('previewModules.archiveNameConflict')
          : err?.message || t('previewModules.archiveExtractFailed'),
        'error',
      );
    } finally {
      if (extractionAbortControllerRef.current === extractionAbortController) {
        extractionAbortControllerRef.current = null;
        setExtracting(false);
      }
    }
  };

  return (
    <div className="w-full max-w-3xl bg-[#131315] border border-neutral-800/80 rounded-2xl overflow-hidden shadow-2xl flex flex-col animate-in zoom-in-95 duration-250">
      <div className="h-12 bg-[#1c1c1c] border-b border-neutral-800/60 px-5 flex items-center justify-between shrink-0 select-none text-xs text-neutral-400 font-bold">
        <div className="flex items-center gap-2.5 min-w-0">
          <Archive size={16} className="text-blue-400 shrink-0" />
          <span className="truncate">{file.name}</span>
        </div>
        <div className="text-[10px] uppercase font-mono tracking-widest text-neutral-600 font-black">
          Drive Archive Object
        </div>
      </div>

      <div className="p-8 flex flex-col items-center justify-center gap-5 text-center min-h-[320px]">
        {entriesLoading ? (
          <>
            <div className="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
            <p className="text-xs text-neutral-400">Loading Drive archive contents...</p>
          </>
        ) : entriesError ? (
          <>
            <Info size={22} className="text-rose-400" />
            <p className="text-xs text-neutral-400 max-w-md">{entriesError}</p>
          </>
        ) : (
          <>
            <div className="w-16 h-16 rounded-2xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400">
              <Archive size={30} />
            </div>
            <div>
              <h4 className="text-sm font-bold text-neutral-100">Archive contents</h4>
              <p className="text-xs text-neutral-500 leading-relaxed mt-2 max-w-md">
                {archiveSummary.fileCount} files, {archiveSummary.folderCount} folders, {formatSize(archiveSummary.totalBytes)} uncompressed.
              </p>
            </div>
            <div className="w-full max-w-md border border-neutral-800/70 rounded-xl overflow-hidden bg-[#0f0f10] text-left">
              {entries.length === 0 ? (
                <div className="px-3 py-3 text-[11px] text-neutral-500">
                  No archive entries returned.
                </div>
              ) : (
                entries.slice(0, 8).map((entry) => (
                  <div
                    key={entry.path}
                    className="flex items-center justify-between gap-4 px-3 py-2 border-b border-neutral-900 last:border-b-0 text-[11px]"
                  >
                    <span className={`truncate ${entry.isDirectory ? 'text-blue-300' : 'text-neutral-300'}`}>
                      {entry.path}
                    </span>
                    {!entry.isDirectory && (
                      <span className="font-mono text-neutral-600 shrink-0">
                        {formatSize(entry.uncompressedSizeBytes)}
                      </span>
                    )}
                  </div>
                ))
              )}
              {entries.length > 8 && (
                <div className="px-3 py-2 text-[11px] text-neutral-500 border-t border-neutral-900">
                  {entries.length - 8} more entries
                </div>
              )}
            </div>
            <div className="flex items-center gap-2">
              {previewUrl && (
                <a
                  href={previewUrl}
                  target="_blank"
                  rel="noreferrer"
                  className="px-3.5 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-xs font-bold flex items-center gap-1.5 transition-colors"
                >
                  <ExternalLink size={13} />
                  {t('previewModules.openArchive')}
                </a>
              )}
              {!isReadOnly && (
                <button
                  onClick={handleExtract}
                  disabled={extracting || entriesLoading}
                  className="px-3.5 py-2 bg-neutral-800 hover:bg-neutral-700 disabled:opacity-60 disabled:cursor-not-allowed text-neutral-200 border border-neutral-700 rounded-lg text-xs font-bold transition-colors"
                >
                  {extracting ? t('previewModules.extracting') : t('previewModules.extract')}
                </button>
              )}
            </div>
            {loading && (
              <p className="text-[11px] text-neutral-600">{t('previewModules.preparingArchiveGrant')}</p>
            )}
            {previewError && !previewUrl && (
              <p className="text-[11px] text-neutral-600 max-w-md">{previewError}</p>
            )}
          </>
        )}
      </div>
    </div>
  );
}
