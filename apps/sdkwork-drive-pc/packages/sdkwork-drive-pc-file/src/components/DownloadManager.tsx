import React, { useState } from 'react';
import {
  Download,
  CheckCircle2,
  X,
  RotateCcw,
  ChevronDown,
  ChevronUp,
  FolderOpen,
  File,
  Loader2,
  AlertCircle,
} from 'lucide-react';
import {
  formatDriveBytes,
  formatTransferInterruptionMessage,
  formatTransferJobProgressDetail,
  formatTransferJobSpeedLabel,
  formatTransferJobTimeRemainingLabel,
  useTranslation,
} from 'sdkwork-drive-pc-commons';
import type { DownloadJob } from 'sdkwork-drive-pc-types';
import {
  canCancelTransferJob,
  isActiveTransferStatus,
  resolveTransferOpenUrl,
} from 'sdkwork-drive-pc-types';

export type { DownloadJob };

interface DownloadManagerProps {
  jobs: DownloadJob[];
  onCancelJob: (id: string) => void;
  onClearJobs: () => void;
  onDismissPanel: () => void;
  onRetryJob: (job: DownloadJob) => void;
  onOpenDownload?: (url: string) => Promise<void> | void;
}

export function DownloadManager({
  jobs,
  onCancelJob,
  onClearJobs,
  onDismissPanel,
  onRetryJob,
  onOpenDownload,
}: DownloadManagerProps) {
  const [isMinimized, setIsMinimized] = useState(false);
  const { t } = useTranslation();

  if (jobs.length === 0) return null;

  // Derive global counters
  const activeCount = jobs.filter(j => isActiveTransferStatus(j.status)).length;
  const activeUploadCount = jobs.filter(j => j.type === 'upload' && isActiveTransferStatus(j.status)).length;
  const activeDownloadCount = activeCount - activeUploadCount;
  const completedCount = jobs.filter(j => j.status === 'completed').length;
  const readyCount = jobs.filter(j => j.status === 'ready').length;
  const finishedCount = completedCount + readyCount;
  const totalCount = jobs.length;

  const formatSize = formatDriveBytes;

  // Status visual configurations
  const getStatusBadge = (status: DownloadJob['status']) => {
    switch (status) {
      case 'connecting':
        return <span className="text-[10px] font-bold py-0.5 px-2 rounded-full bg-blue-50 dark:bg-blue-950/20 text-blue-600 dark:text-blue-400 animate-pulse">{t('downloadManager.connecting')}</span>;
      case 'compressing':
        return <span className="text-[10px] font-bold py-0.5 px-2 rounded-full bg-amber-50 dark:bg-amber-950/20 text-amber-600 dark:text-amber-400">{t('downloadManager.compressing')}</span>;
      case 'downloading':
        return <span className="text-[10px] font-bold py-0.5 px-2 rounded-full bg-sky-50 dark:bg-sky-950/20 text-sky-650 dark:text-sky-400">{t('downloadManager.downloading')}</span>;
      case 'uploading':
        return <span className="text-[10px] font-bold py-0.5 px-2 rounded-full bg-indigo-50 dark:bg-indigo-950/20 text-indigo-600 dark:text-indigo-400">{t('downloadManager.uploading')}</span>;
      case 'checking':
        return <span className="text-[10px] font-bold py-0.5 px-2 rounded-full bg-teal-50 dark:bg-teal-950/20 text-teal-600 dark:text-teal-400 animate-pulse">{t('downloadManager.scanningVirus')}</span>;
      case 'ready':
        return <span className="text-[10px] font-bold py-0.5 px-2 rounded-full bg-emerald-50/15 text-emerald-600 dark:text-emerald-400">{t('downloadManager.ready')}</span>;
      case 'completed':
        return <span className="text-[10px] font-bold py-0.5 px-2 rounded-full bg-emerald-50/15 text-emerald-600 dark:text-emerald-400">{t('downloadManager.completed')}</span>;
      case 'paused':
        return <span className="text-[10px] font-bold py-0.5 px-2 rounded-full bg-gray-100 dark:bg-[#252525] text-gray-500 dark:text-gray-400">{t('downloadManager.paused')}</span>;
      case 'cancelled':
        return <span className="text-[10px] font-bold py-0.5 px-2 rounded-full bg-neutral-100 dark:bg-[#222] text-neutral-400">{t('downloadManager.cancelled')}</span>;
      case 'failed':
        return <span className="text-[10px] font-bold py-0.5 px-2 rounded-full bg-rose-50 dark:bg-rose-950/20 text-rose-600">{t('downloadManager.failed')}</span>;
      default:
        return null;
    }
  };

  return (
    <div 
      className={`fixed right-6 bottom-6 w-96 bg-white dark:bg-[#1c1c1c] border border-gray-100 dark:border-neutral-800 rounded-2xl shadow-2xl overflow-hidden z-50 transition-all duration-300 flex flex-col ${
        isMinimized 
          ? 'h-14 w-80' 
          : 'max-h-[380px] h-fit md:max-h-[460px]'
      }`}
      id="download-manager-panel"
    >
      
      {/* Panel Header */}
      <div className="h-14 border-b border-gray-100 dark:border-neutral-800/60 bg-gray-50/70 dark:bg-[#141414] px-4 flex items-center justify-between shrink-0 select-none">
        <div className="flex items-center gap-2">
          {activeCount > 0 ? (
            <Loader2 size={16} className="text-blue-500 animate-spin" />
          ) : (
            <Download size={16} className="text-gray-400" />
          )}
          <span className="text-xs font-bold text-gray-800 dark:text-white">
            {activeCount > 0 
              ? activeDownloadCount > 0 && activeUploadCount > 0
                ? `${t('downloadManager.downloading')} ${activeDownloadCount} / ${t('downloadManager.uploading')} ${activeUploadCount}`
                : activeUploadCount > 0
                  ? `${t('downloadManager.uploading')} (${activeUploadCount})`
                  : `${t('downloadManager.downloading')} (${activeDownloadCount})`
              : readyCount > 0
                ? `${t('downloadManager.ready')} (${readyCount}/${totalCount})`
                : `${t('downloadManager.completed')} (${completedCount}/${totalCount})`
            }
          </span>
        </div>
        <div className="flex items-center gap-1.5">
          {finishedCount > 0 && activeCount === 0 && !isMinimized && (
            <button
              type="button"
              onClick={onClearJobs}
              className="text-[10.5px] font-bold text-blue-600 hover:text-blue-700 dark:text-blue-400 hover:underline px-2 py-1 rounded transition-all cursor-pointer"
              title={t('downloadManager.clearAll')}
            >
              {t('downloadManager.clearAll')}
            </button>
          )}
          <button
            type="button"
            onClick={() => setIsMinimized(!isMinimized)}
            className="p-1 px-1.5 hover:bg-gray-100 dark:hover:bg-[#282828] text-gray-400 hover:text-gray-600 rounded transition-all cursor-pointer"
            title={isMinimized ? t('downloadManager.expandPanel') : t('downloadManager.minimize')}
            aria-label={isMinimized ? t('downloadManager.expandPanel') : t('downloadManager.minimize')}
          >
            {isMinimized ? <ChevronUp size={15} /> : <ChevronDown size={15} />}
          </button>
          <button
            type="button"
            onClick={onDismissPanel}
            className="p-1.5 hover:bg-gray-100 dark:hover:bg-[#282828] text-gray-400 hover:text-gray-600 rounded transition-all cursor-pointer"
            title={t('downloadManager.dismissPanel')}
            aria-label={t('downloadManager.dismissPanel')}
          >
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Panel Job Rows */}
      {!isMinimized && (
        <div className="flex-1 overflow-y-auto divide-y divide-gray-100 dark:divide-neutral-800/40 max-h-[300px]">
          {jobs.map((job) => {
            const isWorking = isActiveTransferStatus(job.status);
            const canCancel = canCancelTransferJob(job);
            
            return (
              <div key={job.id} className="p-3.5 space-y-2 group transition-colors hover:bg-gray-50/40 dark:hover:bg-[#222]/20 animate-in fade-in duration-200">
                
                {/* Job Metadata row */}
                <div className="flex items-start gap-2.5">
                  <div className={`p-2 rounded-lg shrink-0 ${
                    job.fileType === 'folder' 
                      ? 'bg-amber-500/10 text-amber-500' 
                      : 'bg-blue-500/10 text-blue-500'
                  }`}>
                    {job.fileType === 'folder' ? <FolderOpen size={17} className="stroke-[1.75]" /> : <File size={17} />}
                  </div>
                  <div className="min-w-0 flex-1 leading-normal">
                    <span 
                      className="text-xs font-semibold text-gray-800 dark:text-neutral-200 block truncate" 
                      title={job.fileName}
                    >
                      {job.fileName}
                    </span>
                    <div className="flex items-center gap-1.5 mt-0.5">
                      <span className="text-[10px] text-gray-400">{formatSize(job.totalSize)}</span>
                      <span className="text-[10px] text-gray-300 dark:text-neutral-800">-</span>
                      {getStatusBadge(job.status)}
                    </div>
                  </div>

                  {/* Actions column */}
                  <div className="flex items-center gap-1 self-center shrink-0">
                    {resolveTransferOpenUrl(job) && onOpenDownload && (
                      <button
                        onClick={() => {
                          const openUrl = resolveTransferOpenUrl(job);
                          if (openUrl) {
                            void onOpenDownload(openUrl);
                          }
                        }}
                        className="p-1.5 hover:bg-emerald-50 dark:hover:bg-emerald-950/20 rounded text-emerald-600 hover:text-emerald-700 dark:text-emerald-400 cursor-pointer"
                        title={t('transfer.openDownload')}
                      >
                        <Download size={12} />
                      </button>
                    )}
                    {(job.status === 'failed' || job.status === 'cancelled') && (
                      <button 
                        onClick={() => onRetryJob(job)}
                        className="p-1.5 hover:bg-gray-100 dark:hover:bg-[#282828] rounded text-blue-500 hover:text-blue-600 cursor-pointer"
                        title={job.type === 'upload' ? t('transfer.retryUpload') : t('transfer.retryDownload')}
                      >
                        <RotateCcw size={12} />
                      </button>
                    )}
                    {canCancel && (
                      <button 
                        onClick={() => onCancelJob(job.id)}
                        className="p-1.5 hover:bg-rose-50 hover:text-rose-600 dark:hover:bg-rose-950/20 rounded text-gray-400 cursor-pointer"
                        title={t('transfer.cancelTransfer')}
                      >
                        <X size={12} />
                      </button>
                    )}
                  </div>
                </div>

                {/* Progress bar and speed stat */}
                {isWorking && (
                  <div className="space-y-1">
                    <div className="h-1.5 bg-gray-100 dark:bg-neutral-800 rounded-full overflow-hidden">
                      <div 
                        className="h-full bg-blue-500 rounded-full transition-all duration-300 ease-out"
                        style={{ width: `${job.progress}%` }}
                      />
                    </div>
                    <div className="flex items-center justify-between text-[9px] text-gray-400 font-mono">
                      <span>{formatSize(job.downloadedSize)} of {formatSize(job.totalSize)}</span>
                      <div className="flex items-center gap-1">
                        <span>{formatTransferJobSpeedLabel(job.speed, t)}</span>
                        <span>-</span>
                        <span>{formatTransferJobTimeRemainingLabel(job.timeRemaining, t)}</span>
                      </div>
                    </div>
                  </div>
                )}
                
                {/* Completed and Cancelled Progress lines */}
                {job.status === 'ready' && (
                  <div className="flex items-center gap-1.5 text-[10px] text-emerald-500 font-medium">
                    <CheckCircle2 size={11} /> <span>{t('downloadManager.ready')}</span>
                  </div>
                )}
                {job.status === 'completed' && (
                  <div className="flex items-center gap-1.5 text-[10px] text-emerald-500 font-medium">
                    <CheckCircle2 size={11} /> <span>{t('downloadManager.completed')}</span>
                  </div>
                )}
                {job.status === 'cancelled' && (
                  <div className="space-y-1">
                    <div className="flex items-center gap-1.5 text-[10px] text-gray-400 font-medium">
                      <span>{t('downloadManager.cancelled')}</span>
                    </div>
                    {job.errorMessage && (
                      <p className="text-[10px] text-gray-500 dark:text-gray-400 leading-snug" role="status">
                        {formatTransferInterruptionMessage(job.errorMessage, t)}
                      </p>
                    )}
                  </div>
                )}
                {job.status === 'failed' && (
                  <div className="space-y-1">
                    <div className="flex items-center gap-1.5 text-[10px] text-rose-600 dark:text-rose-400 font-medium">
                      <AlertCircle size={11} />
                      <span>{t('downloadManager.failed')}</span>
                    </div>
                    {job.errorMessage && (
                      <p className="text-[10px] text-rose-600/90 dark:text-rose-400/90 leading-snug" role="alert">
                        {formatTransferInterruptionMessage(job.errorMessage, t)}
                      </p>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
