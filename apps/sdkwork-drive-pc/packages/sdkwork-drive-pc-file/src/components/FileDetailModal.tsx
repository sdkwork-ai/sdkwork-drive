import React, { useEffect, useMemo, useState } from 'react';
import {
  Calendar,
  Check,
  ChevronLeft,
  ChevronRight,
  Download,
  Edit2,
  HardDrive,
  Info,
  Palette,
  Sparkles,
  Star,
  X,
} from 'lucide-react';
import { formatDriveBytes, hasSiblingNameConflict, isDriveConflictError, useTranslation } from 'sdkwork-drive-pc-commons';
import type { DriveFile } from 'sdkwork-drive-pc-types';
import { isDriveAbortError, type DriveFileService } from 'sdkwork-drive-pc-core';

const AudioModule = React.lazy(() => import('./preview-modules/AudioModule').then((module) => ({ default: module.AudioModule })));
const ImageModule = React.lazy(() => import('./preview-modules/ImageModule').then((module) => ({ default: module.ImageModule })));
const OfficeModule = React.lazy(() => import('./preview-modules/OfficeModule').then((module) => ({ default: module.OfficeModule })));
const PdfModule = React.lazy(() => import('./preview-modules/PdfModule').then((module) => ({ default: module.PdfModule })));
const TextEditorModule = React.lazy(() => import('./preview-modules/TextEditorModule').then((module) => ({ default: module.TextEditorModule })));
const VideoModule = React.lazy(() => import('./preview-modules/VideoModule').then((module) => ({ default: module.VideoModule })));
const ZipModule = React.lazy(() => import('./preview-modules/ZipModule').then((module) => ({ default: module.ZipModule })));

interface FileDetailModalProps {
  file: DriveFile & { isStarred?: boolean; color?: string };
  fileService: DriveFileService;
  onClose: () => void;
  onDownload: (file: DriveFile) => void;
  onToggleStar: (fileId: string, fileName: string) => void;
  onSetColor?: (folderId: string, color: string) => void;
  onRename?: (file: DriveFile) => void;
  files?: DriveFile[];
  isTrashSection?: boolean;
  onNavigatePreview?: (file: DriveFile) => void;
  onRefreshFolderContent?: () => void;
}

const CUSTOMIZE_COLORS = [
  { name: 'amber', bg: 'bg-amber-500', labelKey: 'colorWarmOrange' },
  { name: 'rose', bg: 'bg-rose-500', labelKey: 'colorSoftPink' },
  { name: 'blue', bg: 'bg-blue-500', labelKey: 'colorClassicBlue' },
  { name: 'emerald', bg: 'bg-emerald-500', labelKey: 'colorMintGreen' },
  { name: 'violet', bg: 'bg-violet-500', labelKey: 'colorNeonPurple' },
  { name: 'red', bg: 'bg-red-500', labelKey: 'colorGoogleRed' },
  { name: 'gray', bg: 'bg-gray-500', labelKey: 'colorSlateGray' },
] as const;

function PreviewModuleFallback() {
  const { t } = useTranslation();
  return (
    <div className="flex min-h-[240px] w-full max-w-3xl flex-col items-center justify-center gap-3 rounded-2xl border border-neutral-800/80 bg-[#131315] text-center shadow-2xl">
      <div className="h-6 w-6 rounded-full border-2 border-blue-500 border-t-transparent animate-spin" />
      <p className="text-xs font-semibold text-neutral-400">{t('fileDetail.previewLoadingModule')}</p>
    </div>
  );
}

export function FileDetailModal({
  file,
  fileService,
  onClose,
  onDownload,
  onToggleStar,
  onSetColor,
  onRename,
  files,
  isTrashSection = false,
  onNavigatePreview,
  onRefreshFolderContent,
}: FileDetailModalProps) {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<'preview' | 'info'>('preview');
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [modalFeedback, setModalFeedback] = useState<{ text: string; type: 'success' | 'info' | 'error' } | null>(null);
  const [versionHistory, setVersionHistory] = useState<Array<{ versionNo: number; createdAt?: string; contentLength?: number }>>([]);
  const [versionHistoryLoading, setVersionHistoryLoading] = useState(false);
  const [versionHistoryError, setVersionHistoryError] = useState<string | null>(null);
  const [previewSourceUrl, setPreviewSourceUrl] = useState<string | null>(null);
  const [previewGrantError, setPreviewGrantError] = useState<string | null>(null);
  const [previewGrantLoading, setPreviewGrantLoading] = useState(false);
  const [isHeaderRenameEditing, setIsHeaderRenameEditing] = useState(false);
  const [headerRenameValue, setHeaderRenameValue] = useState(file.name);
  const headerRenameAbortControllerRef = React.useRef<AbortController | null>(null);

  useEffect(() => {
    headerRenameAbortControllerRef.current?.abort();
    setHeaderRenameValue(file.name);
    setIsHeaderRenameEditing(false);
    return () => {
      headerRenameAbortControllerRef.current?.abort();
      headerRenameAbortControllerRef.current = null;
    };
  }, [file.id, file.name]);

  useEffect(() => {
    if (file.type === 'folder') {
      setVersionHistory([]);
      setVersionHistoryError(null);
      setVersionHistoryLoading(false);
      return;
    }

    let active = true;
    const controller = new AbortController();
    setVersionHistory([]);
    setVersionHistoryError(null);
    setVersionHistoryLoading(true);

    fileService
      .listFileVersions(file.id, { signal: controller.signal })
      .then((versions) => {
        if (!active) return;
        setVersionHistory(versions);
      })
      .catch((error: unknown) => {
        if (!active || isDriveAbortError(error)) return;
        setVersionHistoryError(
          error instanceof Error ? error.message : t('fileDetail.versionHistoryLoadFailed'),
        );
      })
      .finally(() => {
        if (active) {
          setVersionHistoryLoading(false);
        }
      });

    return () => {
      active = false;
      controller.abort();
    };
  }, [file.id, file.type, fileService, t]);

  useEffect(() => {
    if (file.type === 'folder') {
      setPreviewSourceUrl(null);
      setPreviewGrantError(null);
      setPreviewGrantLoading(false);
      return;
    }

    let active = true;
    const previewAbortController = new AbortController();
    setPreviewSourceUrl(null);
    setPreviewGrantError(null);
    setPreviewGrantLoading(true);

    fileService.createDownloadUrl(file, {
      signal: previewAbortController.signal,
    })
      .then((grant) => {
        if (!active) return;
        const sourceUrl = grant.signedSourceUrl || grant.downloadUrl;
        if (!sourceUrl) {
          setPreviewGrantError(t('fileDetail.previewUrlMissing'));
          return;
        }
        setPreviewSourceUrl(sourceUrl);
      })
      .catch((err: any) => {
        if (isDriveAbortError(err)) {
          return;
        }
        if (active) {
          setPreviewGrantError(err?.message || t('fileDetail.previewUrlFailed'));
        }
      })
      .finally(() => {
        if (active) {
          setPreviewGrantLoading(false);
        }
      });

    return () => {
      active = false;
      previewAbortController.abort();
    };
  }, [file.id, file.updatedAt, fileService]);

  const currentIndex = useMemo(() => {
    if (!files) return -1;
    return files.findIndex((item) => item.id === file.id);
  }, [files, file.id]);

  const hasPrev = currentIndex > 0;
  const hasNext = Boolean(files && currentIndex >= 0 && currentIndex < files.length - 1);

  const triggerFeedback = (text: string, type: 'success' | 'info' | 'error' = 'success') => {
    setModalFeedback({ text, type });
    const timer = setTimeout(() => setModalFeedback(null), 3000);
    return () => clearTimeout(timer);
  };

  const handleHeaderRenameSubmit = async () => {
    if (isTrashSection) {
      setIsHeaderRenameEditing(false);
      return;
    }
    const trimmed = headerRenameValue.trim();
    if (!trimmed || trimmed === file.name) {
      setIsHeaderRenameEditing(false);
      return;
    }
    const siblingNames = (files ?? []).map((item) => item.name);
    if (hasSiblingNameConflict(trimmed, siblingNames, file.name)) {
      triggerFeedback(t('fileBrowser.nameConflict'), 'error');
      return;
    }
    headerRenameAbortControllerRef.current?.abort();
    const headerRenameAbortController = new AbortController();
    headerRenameAbortControllerRef.current = headerRenameAbortController;
    try {
      await fileService.renameFile(file.id, trimmed, {
        signal: headerRenameAbortController.signal,
      });
      triggerFeedback(t('fileDetail.renameSuccess'), 'success');
      setIsHeaderRenameEditing(false);
      onRefreshFolderContent?.();
    } catch (err: any) {
      if (isDriveAbortError(err)) {
        return;
      }
      triggerFeedback(
        isDriveConflictError(err)
          ? t('fileBrowser.nameConflict')
          : err?.message || t('fileDetail.renameFailed'),
        'error',
      );
    } finally {
      if (headerRenameAbortControllerRef.current === headerRenameAbortController) {
        headerRenameAbortControllerRef.current = null;
      }
    }
  };

  const handlePrev = () => {
    if (files && hasPrev && onNavigatePreview) {
      onNavigatePreview(files[currentIndex - 1]);
    }
  };

  const handleNext = () => {
    if (files && hasNext && onNavigatePreview) {
      onNavigatePreview(files[currentIndex + 1]);
    }
  };

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement;
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.getAttribute('contenteditable') === 'true'
      ) {
        return;
      }
      if (event.key === 'ArrowLeft') {
        handlePrev();
      } else if (event.key === 'ArrowRight') {
        handleNext();
      } else if (event.key === 'Escape') {
        onClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [currentIndex, files, file.id, hasNext, hasPrev]);

  const formatSize = formatDriveBytes;

  const formatDate = (dateString: string) => {
    try {
      const value = new Date(dateString);
      return `${value.toLocaleDateString('default', { month: 'long', day: 'numeric', year: 'numeric' })} ${value.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
    } catch {
      return dateString;
    }
  };

  const fileExt = file.name.split('.').pop()?.toUpperCase() || 'DOCUMENT';
  const fileNameLower = file.name.toLowerCase();
  const mimeTypeLower = file.mimeType?.toLowerCase() || '';
  const isTextPreview =
    mimeTypeLower.startsWith('text/') ||
    /\.(txt|md|markdown|json|xml|ya?ml|js|jsx|ts|tsx|css|html|htm|log)$/i.test(file.name);
  const isDocumentPreview =
    mimeTypeLower.includes('wordprocessingml') ||
    /\.(doc|docx)$/i.test(file.name);
  const isSpreadsheetPreview =
    mimeTypeLower.includes('spreadsheetml') ||
    /\.(xls|xlsx|csv)$/i.test(file.name);
  const isPresentationPreview =
    mimeTypeLower.includes('presentationml') ||
    /\.(ppt|pptx)$/i.test(file.name);
  const isArchivePreview =
    mimeTypeLower.includes('zip') ||
    /\.zip$/i.test(file.name);

  const renderPreview = () => {
    if (file.type === 'folder') {
      return (
        <div className="w-full max-w-md space-y-6 text-center animate-in zoom-in-95 duration-200">
          <div className="flex justify-center">
            <div className={`w-24 h-24 ${
              file.color === 'rose' ? 'text-rose-500 bg-rose-950/20 border-rose-800/20' :
              file.color === 'emerald' ? 'text-emerald-500 bg-emerald-950/20 border-emerald-800/20' :
              file.color === 'blue' ? 'text-blue-500 bg-blue-950/20 border-blue-800/20' :
              file.color === 'violet' ? 'text-violet-500 bg-violet-950/20 border-pink-800/20' :
              file.color === 'amber' ? 'text-amber-500 bg-amber-950/20 border-amber-800/20' :
              file.color === 'red' ? 'text-red-500 bg-red-950/20 border-red-800/20' :
              file.color === 'gray' ? 'text-gray-400 bg-neutral-900 border-neutral-800' :
              'text-[#f39c12] bg-[#f39c12]/10 border-[#f39c12]/20'
            } rounded-3xl flex items-center justify-center shadow-inner border transition-all duration-300 hover:scale-105`}>
              <Palette size={48} className="stroke-[1.5]" />
            </div>
          </div>
          <div>
            <h4 className="text-[17px] font-bold text-gray-100">{file.name}</h4>
            <p className="text-xs text-neutral-500 mt-1">{t('fileDetail.folderMetadataHint')}</p>
          </div>

          {onSetColor && !isTrashSection && (
            <div className="bg-[#181818]/80 border border-neutral-800/80 p-5 rounded-2xl shadow-lg text-left">
              <span className="text-[10px] font-bold text-neutral-500 uppercase tracking-wider block mb-3 font-mono">
                {t('fileDetail.folderDisplayColorLabel')}
              </span>
              <div className="grid grid-cols-7 gap-2.5">
                {CUSTOMIZE_COLORS.map((color) => (
                  <button
                    key={color.name}
                    onClick={() => onSetColor(file.id, color.name)}
                    className={`w-8 h-8 rounded-full ${color.bg} relative flex items-center justify-center transition-all duration-150 hover:scale-110 active:scale-95 cursor-pointer shadow-md`}
                    title={t(`fileDetail.${color.labelKey}`)}
                  >
                    {file.color === color.name && <Check size={14} className="text-white stroke-[3.5]" />}
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>
      );
    }

    if (isTextPreview) {
      return (
        <TextEditorModule
          file={file}
          fileService={fileService}
          triggerFeedback={triggerFeedback}
          onSaved={onRefreshFolderContent}
          isReadOnly={isTrashSection}
        />
      );
    }
    if (isDocumentPreview || isSpreadsheetPreview || isPresentationPreview) {
      return (
        <OfficeModule
          file={file}
          previewUrl={previewSourceUrl}
          previewError={previewGrantError}
          loading={previewGrantLoading}
          kind={isPresentationPreview ? 'presentation' : isSpreadsheetPreview ? 'spreadsheet' : 'document'}
        />
      );
    }
    if (mimeTypeLower.includes('pdf') || fileNameLower.endsWith('.pdf')) {
      return (
        <PdfModule
          file={file}
          fileService={fileService}
          previewUrl={previewSourceUrl}
          previewError={previewGrantError}
          loading={previewGrantLoading}
          triggerFeedback={triggerFeedback}
          isReadOnly={isTrashSection}
        />
      );
    }
    if (mimeTypeLower.includes('video') || fileNameLower.endsWith('.mp4') || fileNameLower.endsWith('.mov')) {
      return <VideoModule file={file} previewUrl={previewSourceUrl} previewError={previewGrantError} loading={previewGrantLoading} />;
    }
    if (mimeTypeLower.includes('audio') || fileNameLower.endsWith('.mp3') || fileNameLower.endsWith('.wav')) {
      return <AudioModule file={file} previewUrl={previewSourceUrl} previewError={previewGrantError} loading={previewGrantLoading} />;
    }
    if (
      mimeTypeLower.includes('image') ||
      fileNameLower.endsWith('.png') ||
      fileNameLower.endsWith('.jpg') ||
      fileNameLower.endsWith('.svg') ||
      fileNameLower.endsWith('.gif')
    ) {
      return (
        <ImageModule
          file={file}
          previewUrl={previewSourceUrl}
          previewError={previewGrantError}
          loading={previewGrantLoading}
          triggerFeedback={triggerFeedback}
        />
      );
    }
    if (isArchivePreview) {
      return (
        <ZipModule
          file={file}
          fileService={fileService}
          previewUrl={previewSourceUrl}
          previewError={previewGrantError}
          loading={previewGrantLoading}
          triggerFeedback={triggerFeedback}
          onExtracted={onRefreshFolderContent}
          isReadOnly={isTrashSection}
        />
      );
    }
    return (
      <OfficeModule
        file={file}
        previewUrl={previewSourceUrl}
        previewError={previewGrantError}
        loading={previewGrantLoading}
        kind="document"
      />
    );
  };

  return (
    <div className="fixed inset-0 bg-[#09090b] z-50 flex flex-col overflow-hidden animate-in fade-in duration-300 text-white font-sans w-screen h-screen" id="file-detail-modal">
      <div className="h-16 border-b border-neutral-900/80 bg-neutral-950/90 backdrop-blur-md flex items-center justify-between px-6 shrink-0 select-none">
        <div className="flex items-center gap-3.5 min-w-0">
          <span className="text-[11px] font-bold px-2.5 py-1 rounded bg-[#252525] text-blue-400 font-mono tracking-widest border border-blue-500/10">
            {file.type === 'folder' ? t('fileDetail.folderBadge').toUpperCase() : fileExt}
          </span>
          <div className="min-w-0">
            {isHeaderRenameEditing ? (
              <input
                type="text"
                value={headerRenameValue}
                onChange={(event) => setHeaderRenameValue(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') void handleHeaderRenameSubmit();
                  if (event.key === 'Escape') setIsHeaderRenameEditing(false);
                }}
                onBlur={() => void handleHeaderRenameSubmit()}
                autoFocus
                className="bg-neutral-800 text-white border border-blue-500 rounded px-2 py-0.5 text-[13.5px] font-semibold outline-none max-w-[200px] md:max-w-xs focus:ring-1 focus:ring-blue-500"
              />
            ) : (
              <h3
                className="text-[14.5px] font-bold text-gray-100 truncate flex items-center gap-2 cursor-pointer hover:text-blue-400 transition-colors"
                title={t('fileDetail.renameInlineTitle')}
                onDoubleClick={() => {
                  if (!isTrashSection) {
                    setIsHeaderRenameEditing(true);
                  }
                }}
              >
                {file.name}
                {file.isStarred && <Star className="text-amber-500 fill-amber-500 inline-block shrink-0" size={13} />}
              </h3>
            )}
            <p className="text-[11px] text-neutral-500 truncate mt-0.5">
              {t('fileDetail.ownerLastSync', {
                owner: file.ownerId,
                date: formatDate(file.updatedAt).split(',')[0],
              })}
            </p>
          </div>
        </div>

        {modalFeedback && (
          <div className={`hidden md:flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-semibold animate-in fade-in slide-in-from-top-2 border ${
            modalFeedback.type === 'success' ? 'bg-emerald-950/40 text-emerald-400 border-emerald-500/20' :
            modalFeedback.type === 'error' ? 'bg-rose-950/40 text-rose-400 border-rose-500/20' :
            'bg-blue-950/40 text-blue-400 border-blue-500/20'
          }`}>
            <Sparkles size={12} className="animate-pulse" />
            <span>{modalFeedback.text}</span>
          </div>
        )}

        <div className="flex items-center gap-2">
          {!isTrashSection && (
            <>
              <button
                onClick={() => onToggleStar(file.id, file.name)}
                className={`p-2 rounded-lg hover:bg-[#2c2c2c] transition-colors cursor-pointer ${file.isStarred ? 'text-amber-400' : 'text-neutral-500 hover:text-neutral-200'}`}
                title={t('fileDetail.starToggleTitle')}
              >
                <Star size={17} className={file.isStarred ? 'fill-current' : ''} />
              </button>

              {onRename && (
                <button
                  onClick={() => {
                    setHeaderRenameValue(file.name);
                    setIsHeaderRenameEditing(true);
                  }}
                  className="p-2 text-neutral-500 hover:text-neutral-200 hover:bg-[#2c2c2c] rounded-lg transition-colors cursor-pointer"
                  title={t('fileDetail.renameButtonTitle')}
                >
                  <Edit2 size={16} />
                </button>
              )}

              <button
                onClick={() => onDownload(file)}
                className="p-2 text-neutral-500 hover:text-neutral-200 hover:bg-[#2c2c2c] rounded-lg transition-colors cursor-pointer"
                title={t('fileDetail.downloadLocalTitle')}
              >
                <Download size={16} />
              </button>

              <span className="w-px h-5 bg-[#252525] mx-1" />
            </>
          )}

          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            className={`p-2 rounded-lg hover:bg-[#2c2c2c] transition-colors cursor-pointer ${sidebarOpen ? 'text-blue-400 bg-[#252525]/50' : 'text-neutral-500 hover:text-neutral-200'}`}
            title={t('fileDetail.togglePropertiesTitle')}
          >
            <Info size={18} />
          </button>

          <button
            onClick={onClose}
            className="p-1.5 bg-[#252525] hover:bg-rose-500/15 hover:text-rose-400 text-neutral-300 rounded-lg cursor-pointer transition-all border border-transparent hover:border-rose-500/30 flex items-center gap-1.5 px-3 text-xs font-semibold"
            title={t('fileDetail.closePreviewTitle')}
          >
            <X size={15} />
            <span>{t('fileDetail.close')}</span>
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-hidden flex min-h-0 bg-[#0f0f0f]">
        <div className="flex-1 flex flex-col overflow-hidden relative">
          {hasPrev && (
            <button
              onClick={handlePrev}
              className="absolute left-4 top-1/2 -translate-y-1/2 z-30 p-2.5 rounded-full bg-black/60 hover:bg-neutral-800 border border-neutral-800 text-neutral-300 hover:text-white transition-all cursor-pointer opacity-40 hover:opacity-100 shadow-xl"
              title={t('fileDetail.previousFileTitle')}
            >
              <ChevronLeft size={22} className="stroke-[2.5]" />
            </button>
          )}

          {hasNext && (
            <button
              onClick={handleNext}
              className="absolute right-4 top-1/2 -translate-y-1/2 z-30 p-2.5 rounded-full bg-black/60 hover:bg-neutral-800 border border-neutral-800 text-neutral-300 hover:text-white transition-all cursor-pointer opacity-40 hover:opacity-100 shadow-xl"
              title={t('fileDetail.nextFileTitle')}
            >
              <ChevronRight size={22} className="stroke-[2.5]" />
            </button>
          )}

          <div className="flex-1 overflow-y-auto p-6 flex flex-col justify-center items-center relative select-none">
            <React.Suspense fallback={<PreviewModuleFallback />}>
              {renderPreview()}
            </React.Suspense>
          </div>

          <div className="h-12 border-t border-neutral-900 px-6 shrink-0 bg-[#161616] flex items-center justify-between text-xs font-mono text-neutral-400 select-none">
            <span className="flex items-center gap-1.5 uppercase font-bold text-[10px] tracking-wider text-neutral-500">
              <HardDrive size={13} className="text-blue-500" /> {t('fileDetail.fileSystemLocation')}:{' '}
              {file.parentId
                ? t('fileDetail.subdirectoryNode', { nodeId: file.parentId })
                : t('fileDetail.rootDirectory')}
            </span>

            {files && files.length > 0 && (
              <span className="text-neutral-500 font-bold">
                {t('fileDetail.fileCounter', { current: currentIndex + 1, total: files.length })}
              </span>
            )}
          </div>
        </div>

        {sidebarOpen && (
          <div className="w-80 border-l border-neutral-900 bg-[#151515] p-6 flex flex-col gap-6 overflow-y-auto shrink-0 select-none animate-in slide-in-from-right duration-250">
            <div className="flex border-b border-neutral-900 text-xs font-semibold text-neutral-500">
              <button
                onClick={() => setActiveTab('preview')}
                className={`flex-1 pb-2 border-b-2 text-center transition-all cursor-pointer ${activeTab === 'preview' ? 'border-blue-500 text-blue-400 font-bold' : 'border-transparent hover:text-neutral-300'}`}
              >
                {t('fileDetail.resourceMetaTab')}
              </button>
              <button
                onClick={() => setActiveTab('info')}
                className={`flex-1 pb-2 border-b-2 text-center transition-all cursor-pointer ${activeTab === 'info' ? 'border-blue-500 text-blue-400 font-bold' : 'border-transparent hover:text-neutral-300'}`}
              >
                {t('fileDetail.activityTab')}
              </button>
            </div>

            {activeTab === 'preview' ? (
              <div className="space-y-5 text-xs text-neutral-300">
                <div>
                  <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-widest block mb-1">{t('previewModules.fileName')}</label>
                  <span className="font-bold text-gray-200 select-text break-all block leading-relaxed">{file.name}</span>
                </div>

                <div>
                  <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-widest block mb-1">{t('previewModules.typeClassification')}</label>
                  <span className="font-medium text-neutral-400 capitalize">
                    {file.type === 'folder' ? t('fileDetail.workspaceFolderNode') : t('fileDetail.digitalWorkspaceDocument')}
                  </span>
                </div>

                {file.type !== 'folder' && (
                  <>
                    <div>
                      <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-widest block mb-1">{t('previewModules.mimeHeader')}</label>
                      <span className="font-mono text-xs text-neutral-400 select-text block break-all leading-normal">{file.mimeType || 'application/octet-stream'}</span>
                    </div>
                    <div>
                      <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-widest block mb-1">{t('fileBrowser.fileSize')}</label>
                      <span className="font-mono font-bold text-gray-200 block">{formatSize(file.size)}</span>
                    </div>
                  </>
                )}

                <div>
                  <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-widest block mb-1">{t('previewModules.workspaceLocation')}</label>
                  <span className="text-neutral-400 break-all select-text font-mono text-[11px] block leading-normal">
                    {file.spaceId
                      ? t('fileDetail.workspaceLocationValue', {
                          spaceId: file.spaceId,
                          parentId: file.parentId || t('fileDetail.rootDirectory'),
                        })
                      : t('fileDetail.workspaceLocationUnknown')}
                  </span>
                </div>

                <div>
                  <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-widest block mb-1">{t('previewModules.timestampModified')}</label>
                  <div className="flex items-center gap-2 text-neutral-400 font-mono text-[11px]">
                    <Calendar size={13} className="text-neutral-600 shrink-0" />
                    <span>{formatDate(file.updatedAt)}</span>
                  </div>
                </div>
              </div>
            ) : (
              <div className="space-y-6 text-xs text-neutral-300">
                <div>
                  <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-widest block mb-2">{t('previewModules.verifiedOwner')}</label>
                  <div className="flex items-center gap-3 bg-[#1e1e1e] border border-neutral-900 p-2.5 rounded-xl">
                    <div className="w-8 h-8 rounded-lg bg-blue-500/10 text-blue-400 flex items-center justify-center font-bold font-mono text-xs border border-blue-500/10">
                      {file.ownerId.charAt(0)}
                    </div>
                    <div className="min-w-0">
                      <span className="font-bold text-gray-200 block truncate leading-normal">{file.ownerId}</span>
                      <span className="text-[10px] text-neutral-500 block truncate mt-0.5">{t('previewModules.workspaceAdmin')}</span>
                    </div>
                  </div>
                </div>

                <div>
                  <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-widest block mb-3">{t('previewModules.versionHistory')}</label>
                  {versionHistoryLoading ? (
                    <p className="text-[11px] text-neutral-500">{t('fileDetail.versionHistoryLoading')}</p>
                  ) : versionHistoryError ? (
                    <p className="text-[11px] text-rose-400">{versionHistoryError}</p>
                  ) : versionHistory.length === 0 ? (
                    <p className="text-[11px] text-neutral-500">{t('fileDetail.versionHistoryEmpty')}</p>
                  ) : (
                    <div className="space-y-3 font-mono text-[11px] text-neutral-500">
                      {versionHistory.map((version) => (
                        <div key={version.versionNo} className="flex items-start gap-2 border-l border-neutral-800 pl-3">
                          <div>
                            <span className="text-gray-200 font-bold block leading-normal">
                              {t('fileDetail.versionEntryLabel', { versionNo: version.versionNo })}
                            </span>
                            {version.createdAt ? (
                              <span className="text-[10px] text-neutral-500 block mt-1">{formatDate(version.createdAt)}</span>
                            ) : null}
                            {typeof version.contentLength === 'number' ? (
                              <span className="text-[10px] text-neutral-500 block mt-1">
                                {formatDriveBytes(version.contentLength)}
                              </span>
                            ) : null}
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            )}

            <div className="bg-sky-500/5 border border-sky-500/10 p-4.5 rounded-2xl flex gap-2.5 mt-auto text-sky-400">
              <Sparkles className="text-sky-400 shrink-0 mt-0.5" size={14} />
              <div>
                <span className="text-[11px] font-bold block uppercase tracking-wider">{t('previewModules.sdkPreviewTitle')}</span>
                <p className="text-[11.5px] text-sky-400/70 mt-0.5 leading-relaxed font-normal">
                  {t('previewModules.sdkPreviewDesc')}
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
