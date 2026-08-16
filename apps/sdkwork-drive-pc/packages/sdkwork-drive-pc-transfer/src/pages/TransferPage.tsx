import React, { useState, useMemo } from 'react';
import { 
  Download, 
  Upload, 
  Search, 
  Pause, 
  Play, 
  X, 
  RotateCcw, 
  Trash2, 
  CheckCircle2, 
  AlertCircle, 
  FileText, 
  Folder, 
  Zap, 
  TrendingUp, 
  Grid,
  Clock,
  RefreshCcw,
  ExternalLink
} from 'lucide-react';
import {
  formatDriveBytes,
  formatTransferInterruptionMessage,
  formatTransferJobProgressDetail,
  useTranslation,
} from 'sdkwork-drive-pc-commons';
import {
  canCancelTransferJob,
  canPauseTransferJob,
  canResumeTransferJob,
  resolveTransferOpenUrl,
  type DownloadJob,
  isActiveTransferStatus,
} from 'sdkwork-drive-pc-types';

interface TransferPageProps {
  downloadJobs: DownloadJob[];
  setDownloadJobs: React.Dispatch<React.SetStateAction<DownloadJob[]>>;
  onOpenDownload?: (url: string) => Promise<void> | void;
  onRetryJob: (job: DownloadJob) => void;
  onCancelJob: (id: string) => void;
  onPauseJob?: (id: string) => void;
  onResumeJob?: (id: string) => void;
}

type FilterStatus = 'all' | 'active' | 'completed' | 'paused' | 'cancelled-failed';

export function TransferPage({
  downloadJobs,
  setDownloadJobs,
  onOpenDownload,
  onRetryJob,
  onCancelJob,
  onPauseJob,
  onResumeJob,
}: TransferPageProps) {
  const { t } = useTranslation();
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<FilterStatus>('all');

  const formatSize = formatDriveBytes;

  const handleCancel = (id: string) => {
    onCancelJob(id);
  };

  const handleRetry = (job: DownloadJob) => {
    onRetryJob(job);
  };

  const handleRemove = (id: string) => {
    setDownloadJobs(prev => prev.filter(j => j.id !== id));
  };

  const handleOpenDownload = (job: DownloadJob) => {
    if (!onOpenDownload) return;
    const openUrl = resolveTransferOpenUrl(job);
    if (!openUrl) return;
    void onOpenDownload(openUrl);
  };

  // Bulk actions handlers
  const handleClearFinished = () => {
    setDownloadJobs(prev => prev.filter(j => j.status !== 'completed' && j.status !== 'ready'));
  };

  const handleCancelAllActive = () => {
    downloadJobs
      .filter((job) => isActiveTransferStatus(job.status))
      .forEach((job) => onCancelJob(job.id));
  };

  const handleClearAll = () => {
    downloadJobs
      .filter((job) => isActiveTransferStatus(job.status))
      .forEach((job) => onCancelJob(job.id));
    setDownloadJobs([]);
  };

  // Dynamic search and filter projection
  const filteredJobs = useMemo(() => {
    return downloadJobs.filter(job => {
      // 1. Filter by search query
      const matchesSearch = job.fileName.toLowerCase().includes(searchQuery.toLowerCase());
      if (!matchesSearch) return false;

      // 2. Filter by status categorization
      if (statusFilter === 'active') {
        return isActiveTransferStatus(job.status);
      }
      if (statusFilter === 'completed') {
        return job.status === 'completed' || job.status === 'ready';
      }
      if (statusFilter === 'paused') {
        return job.status === 'paused';
      }
      if (statusFilter === 'cancelled-failed') {
        return job.status === 'cancelled' || job.status === 'failed';
      }

      return true;
    });
  }, [downloadJobs, searchQuery, statusFilter]);

  // Compute calculated metrics for bento cards
  const metrics = useMemo(() => {
    const total = downloadJobs.length;
    const active = downloadJobs.filter(j => isActiveTransferStatus(j.status)).length;
    const completed = downloadJobs.filter(j => j.status === 'completed').length;
    const ready = downloadJobs.filter(j => j.status === 'ready').length;
    const failed = downloadJobs.filter(j => j.status === 'cancelled' || j.status === 'failed').length;
    const successful = completed + ready;
    
    // Total transferred volume
    const transferredSum = downloadJobs.reduce((acc, current) => acc + (current.downloadedSize || 0), 0);

    // Calculate active speed
    let speedSum = 0;
    downloadJobs.forEach(j => {
      if (j.status === 'downloading' || j.status === 'uploading') {
        const val = parseFloat(j.speed);
        if (!isNaN(val)) speedSum += val;
      }
    });

    return {
      total,
      active,
      completed: successful,
      ready,
      failed,
      transferredText: formatSize(transferredSum),
      cumulativeSpeed: speedSum > 0 ? `${speedSum.toFixed(1)} MB/s` : '0 B/s',
      successRate: total > 0 ? `${Math.round((successful / (total - active || 1)) * 100)}%` : '--'
    };
  }, [downloadJobs]);

  const getStatusStyle = (status: DownloadJob['status']) => {
    switch (status) {
      case 'connecting':
        return { text: 'text-blue-500', bg: 'bg-blue-50 dark:bg-blue-950/20', border: 'border-blue-100 dark:border-blue-950/35' };
      case 'compressing':
        return { text: 'text-amber-500', bg: 'bg-amber-50 dark:bg-amber-950/20', border: 'border-amber-100 dark:border-amber-950/35' };
      case 'downloading':
        return { text: 'text-sky-500', bg: 'bg-sky-50 dark:bg-sky-950/20', border: 'border-sky-100 dark:border-sky-950/35' };
      case 'uploading':
        return { text: 'text-indigo-500', bg: 'bg-indigo-50 dark:bg-indigo-950/20', border: 'border-indigo-100 dark:border-indigo-950/35' };
      case 'checking':
        return { text: 'text-teal-500', bg: 'bg-teal-50 dark:bg-teal-950/20', border: 'border-teal-100 dark:border-teal-950/35' };
      case 'ready':
        return { text: 'text-emerald-600 dark:text-emerald-400', bg: 'bg-emerald-50 dark:bg-emerald-950/15', border: 'border-emerald-100 dark:border-emerald-950/30' };
      case 'completed':
        return { text: 'text-emerald-600 dark:text-emerald-400', bg: 'bg-emerald-50 dark:bg-emerald-950/15', border: 'border-emerald-100 dark:border-emerald-950/30' };
      case 'paused':
        return { text: 'text-gray-500', bg: 'bg-gray-50 dark:bg-[#252525]', border: 'border-gray-100 dark:border-neutral-800' };
      case 'cancelled':
      case 'failed':
        return { text: 'text-rose-500', bg: 'bg-rose-50 dark:bg-rose-950/20', border: 'border-rose-100 dark:border-rose-950/35' };
      default:
        return { text: 'text-gray-500', bg: 'bg-gray-50', border: 'border-gray-100' };
    }
  };

  return (
    <div className="flex min-h-0 min-w-0 flex-1 bg-white dark:bg-[#151515] flex flex-col h-full overflow-hidden transition-colors relative">
      
      {/* Search & Bulk Options Header */}
      <div className="shrink-0 border-b border-[#f0f0f0] bg-white transition-colors select-none dark:border-[#222] dark:bg-[#151515]">
        <div className="border-b border-[#f5f5f5] px-4 py-2.5 dark:border-[#1e1e1e] sm:px-8">
          <div className="relative w-full">
            <input
              type="search"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder={t('transfer.searchTransfers')}
              className="w-full rounded-lg border border-transparent bg-[#f5f5f5] py-2 pl-10 pr-8 text-[13px] text-gray-800 outline-none transition-all placeholder:text-gray-400 focus:border-blue-500 focus:bg-white focus:shadow-[0_0_0_4px_rgba(59,130,246,0.08)] dark:bg-[#222] dark:text-gray-200 dark:placeholder:text-gray-600 dark:focus:border-blue-500 dark:focus:bg-[#1a1a1a]"
            />
            <Search className="pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 text-[#999] dark:text-[#666]" size={17} />
            {searchQuery && (
              <button
                type="button"
                onClick={() => setSearchQuery('')}
                className="absolute right-3 top-1/2 -translate-y-1/2 cursor-pointer text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
              >
                <X size={15} />
              </button>
            )}
          </div>
        </div>

        <div className="flex flex-nowrap items-center gap-3 overflow-hidden px-4 py-2.5 sm:px-8 sm:py-3">
          <div className="flex min-w-0 flex-1 items-center gap-3 overflow-hidden">
            <Clock className="shrink-0 text-blue-500" size={19} />
            <h2 className="truncate text-md font-bold leading-none text-gray-900 dark:text-white">
              {t('sidebar.transferCenter')}
            </h2>
            <span className="shrink-0 rounded-full bg-gray-100 px-2.5 py-1 text-xs font-semibold text-gray-500 dark:bg-neutral-800 dark:text-gray-400">
              {filteredJobs.length} {t('fileBrowser.items')}
            </span>
          </div>

          <div className="flex shrink-0 flex-nowrap items-center gap-2">
          {metrics.active > 0 && (
            <button 
              onClick={handleCancelAllActive}
              className="px-3.5 py-2 text-xs font-bold text-rose-600 dark:text-rose-450 hover:bg-rose-50 dark:hover:bg-rose-950/20 border border-rose-100 dark:border-rose-950/30 rounded-lg transition-all cursor-pointer"
            >
              {t('transfer.cancelAllActive')}
            </button>
          )}
          {metrics.completed > 0 && (
            <button 
              onClick={handleClearFinished}
              className="px-3.5 py-2 text-xs font-bold text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-950/15 border border-blue-105 dark:border-blue-950/30 rounded-lg transition-all cursor-pointer"
            >
              {t('transfer.clearFinished')}
            </button>
          )}
          {metrics.total > 0 && (
            <button 
              onClick={handleClearAll}
              className="px-3 py-2 text-gray-400 hover:text-rose-600 dark:text-neutral-500 dark:hover:text-rose-400 transition-colors cursor-pointer"
              title={t('transfer.clearLogs')}
            >
              <Trash2 size={16} />
            </button>
          )}
          </div>
        </div>
      </div>

      {/* Main Container - Scrollable area */}
      <div className="flex-1 overflow-y-auto px-8 py-6 space-y-6">
        
        {/* Statistics Widgets Row */}
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4 select-none">
          
          <div className="bg-[#fcfcfc] dark:bg-[#1a1a1a]/40 border border-gray-100 dark:border-neutral-800 rounded-2xl p-4 flex items-center gap-4 transition-all hover:bg-white dark:hover:bg-[#1e1e1e]/60">
            <div className="w-10 h-10 rounded-xl bg-blue-500/10 dark:bg-blue-950/20 text-blue-500 flex items-center justify-center shrink-0">
              <Download size={18} />
            </div>
            <div>
              <span className="text-[10px] uppercase font-bold text-gray-400 tracking-wider">{t('transfer.storageSaved')}</span>
              <h3 className="text-md font-bold text-gray-800 dark:text-gray-100 font-mono mt-0.5">{metrics.transferredText}</h3>
            </div>
          </div>

          <div className="bg-[#fcfcfc] dark:bg-[#1a1a1a]/40 border border-gray-100 dark:border-neutral-800 rounded-2xl p-4 flex items-center gap-4 transition-all hover:bg-white dark:hover:bg-[#1e1e1e]/60">
            <div className="w-10 h-10 rounded-xl bg-amber-500/10 dark:bg-amber-950/20 text-amber-500 flex items-center justify-center shrink-0 animate-pulse">
              <Zap size={18} />
            </div>
            <div>
              <span className="text-[10px] uppercase font-bold text-gray-400 tracking-wider">{t('transfer.connectionSpeed')}</span>
              <h3 className="text-md font-bold text-gray-800 dark:text-gray-100 font-mono mt-0.5">{metrics.cumulativeSpeed}</h3>
            </div>
          </div>

          <div className="bg-[#fcfcfc] dark:bg-[#1a1a1a]/40 border border-gray-100 dark:border-neutral-800 rounded-2xl p-4 flex items-center gap-4 transition-all hover:bg-white dark:hover:bg-[#1e1e1e]/60">
            <div className="w-10 h-10 rounded-xl bg-emerald-500/10 dark:bg-emerald-950/25 text-emerald-500 flex items-center justify-center shrink-0">
              <TrendingUp size={18} />
            </div>
            <div>
              <span className="text-[10px] uppercase font-bold text-gray-400 tracking-wider">{t('transfer.successRate')}</span>
              <h3 className="text-md font-bold text-gray-800 dark:text-gray-100 font-mono mt-0.5">{metrics.successRate}</h3>
            </div>
          </div>

          <div className="bg-[#fcfcfc] dark:bg-[#1a1a1a]/40 border border-gray-100 dark:border-neutral-800 rounded-2xl p-4 flex items-center gap-4 transition-all hover:bg-white dark:hover:bg-[#1e1e1e]/60">
            <div className="w-10 h-10 rounded-xl bg-purple-500/10 dark:bg-purple-950/25 text-purple-500 flex items-center justify-center shrink-0">
              <Clock size={18} />
            </div>
            <div>
              <span className="text-[10px] uppercase font-bold text-gray-400 tracking-wider">{t('transfer.transfersDone')}</span>
              <h3 className="text-md font-bold text-gray-800 dark:text-gray-100 font-mono mt-0.5">{metrics.completed} / {metrics.total}</h3>
            </div>
          </div>
        </div>

        {/* Status filtering categories bar */}
        <div className="flex items-center justify-between border-b border-gray-100 dark:border-neutral-800 pb-2.5">
          <div className="flex items-center gap-1.5 text-xs font-semibold select-none text-gray-500">
            <button 
              onClick={() => setStatusFilter('all')}
              className={`px-3 py-1.5 rounded-lg transition-colors cursor-pointer ${statusFilter === 'all' ? 'bg-gray-100 dark:bg-neutral-800 text-gray-800 dark:text-white font-bold' : 'hover:bg-gray-50 dark:hover:bg-neutral-900'}`}
            >
              {t('transfer.allTransfers')}
            </button>
            <button 
              onClick={() => setStatusFilter('active')}
              className={`px-3 py-1.5 rounded-lg transition-colors cursor-pointer ${statusFilter === 'active' ? 'bg-blue-50 dark:bg-blue-950/30 text-blue-600 dark:text-blue-400 font-bold' : 'hover:bg-gray-50 dark:hover:bg-neutral-900'}`}
            >
              {t('transfer.activeTransfers')} ({metrics.active})
            </button>
            <button 
              onClick={() => setStatusFilter('completed')}
              className={`px-3 py-1.5 rounded-lg transition-colors cursor-pointer ${statusFilter === 'completed' ? 'bg-emerald-50 dark:bg-emerald-950/20 text-emerald-600 dark:text-emerald-400 font-bold' : 'hover:bg-gray-50 dark:hover:bg-neutral-900'}`}
            >
              {t('transfer.completed')} ({metrics.completed})
            </button>
            <button 
              onClick={() => setStatusFilter('paused')}
              className={`px-3 py-1.5 rounded-lg transition-colors cursor-pointer ${statusFilter === 'paused' ? 'bg-gray-100 dark:bg-[#2c2c2c] text-gray-600 dark:text-gray-300 font-bold' : 'hover:bg-gray-50 dark:hover:bg-neutral-900'}`}
            >
              {t('transfer.paused')}
            </button>
            <button 
              onClick={() => setStatusFilter('cancelled-failed')}
              className={`px-3 py-1.5 rounded-lg transition-colors cursor-pointer ${statusFilter === 'cancelled-failed' ? 'bg-rose-50 dark:bg-rose-950/20 text-rose-600 dark:text-rose-450 font-bold' : 'hover:bg-gray-50 dark:hover:bg-neutral-900'}`}
            >
              {t('transfer.wipedCanceled')} ({metrics.failed})
            </button>
          </div>
          
          <span className="text-[11px] text-gray-400 font-mono">
            {t('transfer.synchronizedAt')} {new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
          </span>
        </div>

        {/* Job List Viewport Container */}
        <div className="bg-[#fafafa] dark:bg-[#1a1a1a]/15 border border-gray-100 dark:border-neutral-800/80 rounded-2xl overflow-hidden shadow-sm">
          
          {filteredJobs.length === 0 ? (
            <div className="py-24 text-center flex flex-col items-center justify-center gap-3">
              <div className="w-12 h-12 bg-gray-50 dark:bg-neutral-900 border border-gray-105 dark:border-neutral-800 rounded-full flex items-center justify-center text-gray-400">
                <Grid size={22} />
              </div>
              <div>
                <h4 className="text-sm font-bold text-gray-800 dark:text-gray-300">{t('transfer.noMatchingTransfers')}</h4>
                <p className="text-xs text-gray-400 dark:text-neutral-500 mt-1 max-w-[280px]">
                  {t('transfer.noLogText')}
                </p>
              </div>
            </div>
          ) : (
            <div className="divide-y divide-[#ececec] dark:divide-neutral-800/60 font-sans">
              
              {/* Header Titles */}
              <div className="grid grid-cols-[1.5fr_1fr_1.5fr_0.8fr_0.1fr] px-6 py-3 bg-gray-50/75 dark:bg-[#121212] border-b border-gray-100 dark:border-neutral-800/70 text-[10.5px] font-bold text-gray-400 select-none uppercase tracking-wider">
                <div>{t('transfer.resourceInfo')}</div>
                <div>{t('transfer.status')}</div>
                <div>{t('transfer.transferState')}</div>
                <div className="text-right">{t('transfer.transferredSize')}</div>
                <div></div>
              </div>

              {/* Items Display */}
              {filteredJobs.map((job) => {
                const styles = getStatusStyle(job.status);
                const isWorking = isActiveTransferStatus(job.status);
                const canCancel = canCancelTransferJob(job);
                const canPause = canPauseTransferJob(job);
                const canResume = canResumeTransferJob(job);

                return (
                  <div 
                    key={job.id} 
                    className="grid grid-cols-[1.5fr_1fr_1.5fr_0.8fr_0.1fr] px-6 py-4.5 items-center bg-white dark:bg-[#1a1a1a]/30 hover:bg-[#f6f9fc]/40 dark:hover:bg-[#202020]/20 transition-all duration-200 group animate-in fade-in"
                  >
                    {/* Item 1: Name and Mime Indicator */}
                    <div className="flex items-center gap-3.5 pr-4 min-w-0">
                      <div className={`p-2 rounded-xl border shrink-0 transition-all ${
                        job.fileType === 'folder' 
                          ? 'bg-amber-100/10 border-amber-500/10 text-amber-500' 
                          : 'bg-blue-100/10 border-blue-500/10 text-blue-500'
                      }`}>
                        {job.fileType === 'folder' ? <Folder size={18} className="fill-current" /> : <FileText size={18} />}
                      </div>
                      <div className="min-w-0">
                        <span className="text-[13px] font-bold text-gray-800 dark:text-gray-200 block truncate" title={job.fileName}>
                          {job.fileName}
                        </span>
                        <span className="text-[10px] text-gray-400 uppercase tracking-widest block font-bold font-mono mt-1">
                          {job.fileType === 'folder' ? t('transfer.folderArchive') : (job.mimeType?.split('/')[1]?.split('.').pop() || t('transfer.document'))}
                        </span>
                      </div>
                    </div>

                    {/* Item 2: Status Tag */}
                    <div className="flex items-center pr-4">
                      <div className={`border ${styles.border} ${styles.bg} ${styles.text} text-[10.5px] px-2.5 py-0.5 rounded-full font-bold capitalize flex items-center gap-1.5`}>
                        {isWorking && <span className="w-1.5 h-1.5 rounded-full bg-current animate-ping" />}
                        {job.status === 'ready' && <CheckCircle2 size={11} />}
                        {job.status === 'completed' && <CheckCircle2 size={11} />}
                        {job.status === 'failed' && <AlertCircle size={11} />}
                        {job.status === 'connecting' ? t('downloadManager.connecting') :
                         job.status === 'compressing' ? t('downloadManager.compressing') :
                         job.status === 'downloading' ? t('downloadManager.downloading') :
                         job.status === 'uploading' ? t('downloadManager.uploading') :
                         job.status === 'checking' ? t('downloadManager.scanningVirus') :
                         job.status === 'ready' ? t('downloadManager.ready') :
                         job.status === 'completed' ? t('downloadManager.completed') :
                         job.status === 'paused' ? t('downloadManager.paused') :
                         job.status === 'cancelled' ? t('downloadManager.cancelled') :
                         job.status}
                      </div>
                    </div>

                    {/* Item 3: Progress indicators & rates */}
                    <div className="pr-6 space-y-2.5">
                      <div className="flex items-center justify-between text-xs font-mono select-none text-gray-400">
                        <span className="font-bold">{job.progress}%</span>
                        <span className="text-[10px] text-gray-400 font-medium">
                          {formatTransferJobProgressDetail(job, t)}
                        </span>
                      </div>
                      
                      {/* Active dynamic visual linear gauge */}
                      <div className="w-full bg-[#eee] dark:bg-neutral-800/85 h-2 rounded-full overflow-hidden relative">
                        <div 
                          className={`h-full absolute left-0 top-0 transition-all duration-300 ${
                            job.status === 'completed' ? 'bg-gradient-to-r from-emerald-500 to-teal-500' :
                            job.status === 'ready' ? 'bg-emerald-500' :
                            job.status === 'paused' ? 'bg-gray-400' :
                            job.status === 'cancelled' || job.status === 'failed' ? 'bg-rose-500' :
                            'bg-blue-600'
                          }`}
                          style={{ width: `${job.progress}%` }}
                        />
                      </div>
                      {(job.status === 'failed' || job.status === 'cancelled') && job.errorMessage && (
                        <p
                          className="text-[10px] text-rose-600 dark:text-rose-400 font-medium leading-snug"
                          role="alert"
                        >
                          {formatTransferInterruptionMessage(job.errorMessage, t)}
                        </p>
                      )}
                    </div>

                    {/* Item 4: Numerical Size */}
                    <div className="text-right font-mono text-[11px] font-semibold text-gray-500 dark:text-gray-400 pr-5 select-none">
                      {formatSize(job.downloadedSize)} / {formatSize(job.totalSize)}
                    </div>

                    {/* Item 5: Action Button triggers */}
                    <div className="flex justify-end pr-1">
                      <div className="flex items-center gap-1">
                        {(canCancel || canPause || canResume) && (
                          <>
                            {canResume && onResumeJob && (
                              <button 
                                onClick={() => onResumeJob(job.id)}
                                className="p-1.5 hover:bg-blue-50 dark:hover:bg-blue-950/20 text-blue-600 rounded transition-colors cursor-pointer"
                                title={t('transfer.resumeTransfer')}
                              >
                                <Play size={14} className="fill-current" />
                              </button>
                            )}
                            {canPause && onPauseJob && (
                              <button 
                                onClick={() => onPauseJob(job.id)}
                                className="p-1.5 hover:bg-amber-50 dark:hover:bg-amber-950/20 text-amber-600 rounded transition-colors cursor-pointer"
                                title={t('transfer.pauseTransfer')}
                              >
                                <Pause size={14} className="fill-current" />
                              </button>
                            )}
                            {canCancel && (
                              <button 
                                onClick={() => handleCancel(job.id)}
                                className="p-1.5 hover:bg-rose-50 dark:hover:bg-rose-950/20 text-rose-600 rounded transition-colors cursor-pointer"
                                title={t('transfer.cancelTransfer')}
                              >
                                <X size={14} />
                              </button>
                            )}
                          </>
                        )}

                        {resolveTransferOpenUrl(job) && (
                          <button
                            onClick={() => handleOpenDownload(job)}
                            className="p-1.5 hover:bg-emerald-50 dark:hover:bg-emerald-950/20 text-emerald-600 rounded transition-colors cursor-pointer"
                            title={t('transfer.openDownload')}
                          >
                            <ExternalLink size={14} />
                          </button>
                        )}

                        {(job.status === 'cancelled' || job.status === 'failed') && (
                          <button 
                            onClick={() => handleRetry(job)}
                            className="p-1.5 hover:bg-blue-50 dark:hover:bg-blue-950/20 text-blue-600 rounded transition-colors cursor-pointer"
                            title={job.type === 'upload' ? t('transfer.retryTransfer') : t('transfer.retryTransfer')}
                          >
                            <RotateCcw size={14} />
                          </button>
                        )}

                        {!isWorking && (
                          <button 
                            onClick={() => handleRemove(job.id)}
                            className="p-1.5 hover:bg-gray-100 dark:hover:bg-[#2c2c2c] text-gray-400 hover:text-gray-700 dark:hover:text-neutral-200 rounded transition-colors cursor-pointer opacity-0 group-hover:opacity-100"
                            title={t('transfer.removeLog')}
                          >
                            <X size={14} />
                          </button>
                        )}
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
