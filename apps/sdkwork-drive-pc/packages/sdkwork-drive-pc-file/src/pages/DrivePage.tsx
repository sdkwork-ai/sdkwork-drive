import {
  formatDriveBytes,
  useTranslation,
  TRANSFER_SPEED_UPLOADING,
  TRANSFER_TIME_CALCULATING,
  TRANSFER_TIME_FINALIZING,
  TRANSFER_TIME_WAITING_BACKEND,
} from 'sdkwork-drive-pc-commons';
import React, { useRef, useState, useEffect } from 'react';
import { AlertCircle, CheckCircle2 } from 'lucide-react';
import { FileSidebar } from '../components/FileSidebar';
import { FileBrowser } from '../components/FileBrowser';
import type { DriveOpenRequest } from '../types/driveOpenRequest';
import {
  cancelTransferJob,
  applyTransferFailure,
  buildNativeUploadJobFingerprint,
  createRetryFilesForDownloadJob,
  downloadGrantFromJob,
  isDownloadGrantStillValid,
  pauseTransferJob,
  type DownloadJob,
} from 'sdkwork-drive-pc-types';
import { CreateSharedSpaceModal } from '../components/CreateSharedSpaceModal';
import {
  isDriveAbortError,
  loadPersistedTransferJobs,
  NativeLocalUploadFile,
  persistTransferJobs,
  runManagedDownloadTransfer,
  createHostDownloadStreamAdapter,
  useDriveRuntime,
  type DriveFileService,
  type DriveStorageSummary,
  type KnowledgeBaseSpace,
  type SharedSpace,
} from 'sdkwork-drive-pc-core';

const TransferPage = React.lazy(() =>
  import('sdkwork-drive-pc-transfer').then((module) => ({ default: module.TransferPage })),
);

export type DriveSection = string;

export interface DrivePageProps {
  activeSection?: DriveSection;
  fileService: DriveFileService;
  openRequest?: DriveOpenRequest;
  storageSummary?: DriveStorageSummary;
  storageSummaryUnavailable?: boolean;
  onOpenExternal?: (url: string) => Promise<void> | void;
  onOpenRequestHandled?: (requestId: string) => void;
  onOpenStorageSettings?: () => void;
  onSectionChange?: (section: DriveSection) => void;
  shareClaimToken?: string;
  onShareClaimDismiss?: () => void;
}

export function isRetryUploadFileCompatible(job: DownloadJob, selected: File): boolean {
  const selectedFingerprint = `${selected.name}:${selected.size}:${selected.lastModified}`;
  if (job.uploadFileFingerprint) {
    return selectedFingerprint === job.uploadFileFingerprint;
  }
  const sameName = !job.fileName || selected.name === job.fileName;
  const expectedSize = Number(job.totalSize) || 0;
  const sameSize = expectedSize <= 0 || selected.size === expectedSize;
  return sameName && sameSize;
}

function formatExpectedSize(totalSize: number | undefined): string {
  const size = Number(totalSize) || 0;
  if (size <= 0) {
    return 'unknown size';
  }
  return formatDriveBytes(size);
}

type ParsedUploadFingerprint = {
  fileName: string;
  size: number;
  lastModifiedEpochMs: number;
};

function parseUploadFingerprint(fingerprint: string | undefined): ParsedUploadFingerprint | null {
  if (!fingerprint) {
    return null;
  }
  const lastSeparator = fingerprint.lastIndexOf(':');
  if (lastSeparator <= 0 || lastSeparator >= fingerprint.length - 1) {
    return null;
  }
  const secondSeparator = fingerprint.lastIndexOf(':', lastSeparator - 1);
  if (secondSeparator <= 0 || secondSeparator >= lastSeparator - 1) {
    return null;
  }
  const fileName = fingerprint.slice(0, secondSeparator);
  const size = Number(fingerprint.slice(secondSeparator + 1, lastSeparator));
  const lastModifiedEpochMs = Number(fingerprint.slice(lastSeparator + 1));
  if (!fileName || !Number.isFinite(size) || size < 0 || !Number.isFinite(lastModifiedEpochMs) || lastModifiedEpochMs <= 0) {
    return null;
  }
  return {
    fileName,
    size,
    lastModifiedEpochMs,
  };
}

function formatExpectedModifiedTime(epochMs: number | undefined): string {
  const value = Number(epochMs) || 0;
  if (value <= 0) {
    return 'unknown modified time';
  }
  return new Date(value).toISOString().replace('T', ' ').slice(0, 19) + 'Z';
}

export function getUploadRetryMismatchContext(job: DownloadJob): {
  expectedName: string;
  expectedSize: string;
  expectedModified: string;
} {
  const parsedFingerprint = parseUploadFingerprint(job.uploadFileFingerprint);
  return {
    expectedName: parsedFingerprint?.fileName || job.fileName || 'unknown file',
    expectedSize: formatExpectedSize(parsedFingerprint?.size ?? job.totalSize),
    expectedModified: formatExpectedModifiedTime(parsedFingerprint?.lastModifiedEpochMs),
  };
}

export function buildUploadRetryMismatchMessage(job: DownloadJob): string {
  const context = getUploadRetryMismatchContext(job);
  return `Selected file does not match this upload task. Expected "${context.expectedName}" (${context.expectedSize}, modified ${context.expectedModified}).`;
}

export function DrivePage({
  activeSection: propActiveSection,
  fileService,
  openRequest,
  storageSummary,
  storageSummaryUnavailable = false,
  onOpenExternal,
  onOpenRequestHandled,
  onOpenStorageSettings,
  onSectionChange,
  shareClaimToken,
  onShareClaimDismiss,
}: DrivePageProps) {
  const { t } = useTranslation();
  const { host } = useDriveRuntime();
  const hostDownloadStream = createHostDownloadStreamAdapter(host);
  const shareClaimAttemptRef = useRef<string | null>(null);
  const shareClaimDeclinedRef = useRef<string | null>(null);
  const shareClaimAbortControllerRef = useRef<AbortController | null>(null);
  const [pendingShareClaimToken, setPendingShareClaimToken] = useState<string | null>(null);
  const [shareClaimInFlight, setShareClaimInFlight] = useState(false);
  const [localActiveSection, setLocalActiveSection] = useState<DriveSection>('my-storage');
  const activeSection = propActiveSection !== undefined ? propActiveSection : localActiveSection;
  const setActiveSection = onSectionChange !== undefined ? onSectionChange : setLocalActiveSection;

  useEffect(() => {
    if (openRequest && activeSection !== openRequest.section) {
      setActiveSection(openRequest.section);
    }
  }, [activeSection, openRequest, setActiveSection]);
  
  const [downloadJobs, setDownloadJobs] = useState<DownloadJob[]>(() => loadPersistedTransferJobs());
  const [sharedSpaces, setSharedSpaces] = useState<SharedSpace[]>([]);
  const [knowledgeBaseSpaces, setKnowledgeBaseSpaces] = useState<KnowledgeBaseSpace[]>([]);
  const [isCreateSpaceOpen, setIsCreateSpaceOpen] = useState(false);
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'err' } | null>(null);
  const uploadAbortControllersRef = useRef(new Map<string, AbortController>());
  const downloadAbortControllersRef = useRef(new Map<string, AbortController>());
  const sharedSpaceListAbortControllerRef = useRef<AbortController | null>(null);
  const knowledgeBaseSpaceListAbortControllerRef = useRef<AbortController | null>(null);
  const createSpaceAbortControllerRef = useRef<AbortController | null>(null);
  const deleteSpaceAbortControllerRef = useRef<AbortController | null>(null);
  const isMountedRef = useRef(true);

  const triggerToast = (message: string, type: 'success' | 'err' = 'success') => {
    setToast({ message, type });
  };

  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(null), 3500);
    return () => clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    const token = shareClaimToken?.trim();
    if (!token) {
      setPendingShareClaimToken(null);
      return;
    }
    if (shareClaimAttemptRef.current === token || shareClaimDeclinedRef.current === token) {
      return;
    }
    setPendingShareClaimToken(token);
  }, [shareClaimToken]);

  const handleAcceptShareClaim = () => {
    const token = pendingShareClaimToken?.trim();
    if (!token || shareClaimInFlight) {
      return;
    }
    shareClaimAbortControllerRef.current?.abort();
    const controller = new AbortController();
    shareClaimAbortControllerRef.current = controller;
    shareClaimAttemptRef.current = token;
    setShareClaimInFlight(true);
    setPendingShareClaimToken(null);
    fileService
      .claimShareLink(token, { signal: controller.signal })
      .then((result) => {
        if (!isMountedRef.current) {
          return;
        }
        setActiveSection('shared');
        triggerToast(
          result.alreadyClaimed
            ? t('fileBrowser.shareLinkAlreadyClaimed')
            : t('fileBrowser.shareLinkClaimSuccess'),
          'success',
        );
      })
      .catch((error: unknown) => {
        if (isDriveAbortError(error)) {
          return;
        }
        shareClaimAttemptRef.current = null;
        if (!isMountedRef.current) {
          return;
        }
        triggerToast(
          error instanceof Error ? error.message : t('fileBrowser.shareLinkClaimFailed'),
          'err',
        );
      })
      .finally(() => {
        if (shareClaimAbortControllerRef.current === controller) {
          shareClaimAbortControllerRef.current = null;
        }
        if (isMountedRef.current) {
          setShareClaimInFlight(false);
        }
      });
  };

  const handleDeclineShareClaim = () => {
    const token = pendingShareClaimToken?.trim();
    if (token) {
      shareClaimDeclinedRef.current = token;
    }
    shareClaimAbortControllerRef.current?.abort();
    shareClaimAbortControllerRef.current = null;
    setPendingShareClaimToken(null);
    onShareClaimDismiss?.();
  };

  useEffect(() => {
    const timer = window.setTimeout(() => {
      persistTransferJobs(downloadJobs);
    }, 500);
    return () => window.clearTimeout(timer);
  }, [downloadJobs]);

  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
      sharedSpaceListAbortControllerRef.current?.abort();
      createSpaceAbortControllerRef.current?.abort();
      deleteSpaceAbortControllerRef.current?.abort();
      knowledgeBaseSpaceListAbortControllerRef.current?.abort();
      shareClaimAbortControllerRef.current?.abort();
      uploadAbortControllersRef.current.forEach((controller) => controller.abort());
      downloadAbortControllersRef.current.forEach((controller) => controller.abort());
      uploadAbortControllersRef.current.clear();
      downloadAbortControllersRef.current.clear();
    };
  }, []);

  // Initialize and load Shared Spaces
  useEffect(() => {
    let active = true;
    sharedSpaceListAbortControllerRef.current?.abort();
    const sharedSpaceListAbortController = new AbortController();
    sharedSpaceListAbortControllerRef.current = sharedSpaceListAbortController;
    setSharedSpaces(fileService.getSharedSpaces());
    fileService.listSharedSpaces({
      signal: sharedSpaceListAbortController.signal,
    })
      .then((spaces) => {
        if (active) {
          setSharedSpaces(spaces);
        }
      })
      .catch((err) => {
        if (isDriveAbortError(err)) {
          return;
        }
        if (active) {
          setSharedSpaces(fileService.getSharedSpaces());
        }
      })
      .finally(() => {
        if (sharedSpaceListAbortControllerRef.current === sharedSpaceListAbortController) {
          sharedSpaceListAbortControllerRef.current = null;
        }
      });
    return () => {
      active = false;
      sharedSpaceListAbortController.abort();
      if (sharedSpaceListAbortControllerRef.current === sharedSpaceListAbortController) {
        sharedSpaceListAbortControllerRef.current = null;
      }
    };
  }, [fileService]);

  // Initialize and load Knowledge Base Spaces
  useEffect(() => {
    let active = true;
    knowledgeBaseSpaceListAbortControllerRef.current?.abort();
    const knowledgeBaseSpaceListAbortController = new AbortController();
    knowledgeBaseSpaceListAbortControllerRef.current = knowledgeBaseSpaceListAbortController;
    setKnowledgeBaseSpaces(fileService.getKnowledgeBaseSpaces());
    fileService.listKnowledgeBaseSpaces({
      signal: knowledgeBaseSpaceListAbortController.signal,
    })
      .then((spaces) => {
        if (active) {
          setKnowledgeBaseSpaces(spaces);
        }
      })
      .catch((err) => {
        if (isDriveAbortError(err)) {
          return;
        }
        if (active) {
          setKnowledgeBaseSpaces(fileService.getKnowledgeBaseSpaces());
        }
      })
      .finally(() => {
        if (knowledgeBaseSpaceListAbortControllerRef.current === knowledgeBaseSpaceListAbortController) {
          knowledgeBaseSpaceListAbortControllerRef.current = null;
        }
      });
    return () => {
      active = false;
      knowledgeBaseSpaceListAbortController.abort();
      if (knowledgeBaseSpaceListAbortControllerRef.current === knowledgeBaseSpaceListAbortController) {
        knowledgeBaseSpaceListAbortControllerRef.current = null;
      }
    };
  }, [fileService]);

  // Handler to submit new space
  const handleCreateSpaceSubmit = (name: string, icon: string, color: string, description: string) => {
    createSpaceAbortControllerRef.current?.abort();
    const createSpaceAbortController = new AbortController();
    createSpaceAbortControllerRef.current = createSpaceAbortController;
    fileService.createSharedSpace(name, icon, color, description, {
      signal: createSpaceAbortController.signal,
    })
      .then((newSpace) => {
        if (!isMountedRef.current || createSpaceAbortControllerRef.current !== createSpaceAbortController) {
          return;
        }
        setSharedSpaces(fileService.getSharedSpaces());
        setIsCreateSpaceOpen(false);
        setActiveSection(newSpace.id);
        triggerToast(t('sharedSpace.createSuccess', { name: newSpace.name }));
      })
      .catch((err) => {
        if (isDriveAbortError(err)) {
          return;
        }
        if (!isMountedRef.current || createSpaceAbortControllerRef.current !== createSpaceAbortController) {
          return;
        }
        setSharedSpaces(fileService.getSharedSpaces());
        triggerToast(err?.message || t('sharedSpace.createFailed'), 'err');
      })
      .finally(() => {
        if (createSpaceAbortControllerRef.current === createSpaceAbortController) {
          createSpaceAbortControllerRef.current = null;
        }
      });
  };

  // Handler to delete space
  const handleDeleteSpace = (id: string) => {
    deleteSpaceAbortControllerRef.current?.abort();
    const deleteSpaceAbortController = new AbortController();
    deleteSpaceAbortControllerRef.current = deleteSpaceAbortController;
    fileService.deleteSharedSpace(id, {
      signal: deleteSpaceAbortController.signal,
    })
      .then(() => {
        if (!isMountedRef.current || deleteSpaceAbortControllerRef.current !== deleteSpaceAbortController) {
          return;
        }
        setSharedSpaces(fileService.getSharedSpaces());
        if (activeSection === id) {
          setActiveSection('my-storage');
        }
        triggerToast(t('sharedSpace.deleteSuccess'));
      })
      .catch((err) => {
        if (isDriveAbortError(err)) {
          return;
        }
        if (!isMountedRef.current || deleteSpaceAbortControllerRef.current !== deleteSpaceAbortController) {
          return;
        }
        setSharedSpaces(fileService.getSharedSpaces());
        triggerToast(err?.message || t('sharedSpace.deleteFailed'), 'err');
      })
      .finally(() => {
        if (deleteSpaceAbortControllerRef.current === deleteSpaceAbortController) {
          deleteSpaceAbortControllerRef.current = null;
        }
      });
  };
  const createUploadAbortController = (jobId: string): AbortController => {
    const controller = new AbortController();
    uploadAbortControllersRef.current.set(jobId, controller);
    return controller;
  };
  const releaseUploadAbortController = (
    jobId: string,
    controller?: AbortController,
  ): void => {
    const current = uploadAbortControllersRef.current.get(jobId);
    if (!controller || current === controller) {
      uploadAbortControllersRef.current.delete(jobId);
    }
  };
  const createDownloadAbortController = (jobId: string): AbortController => {
    const controller = new AbortController();
    downloadAbortControllersRef.current.set(jobId, controller);
    return controller;
  };
  const releaseDownloadAbortController = (
    jobId: string,
    controller?: AbortController,
  ): void => {
    const current = downloadAbortControllersRef.current.get(jobId);
    if (!controller || current === controller) {
      downloadAbortControllersRef.current.delete(jobId);
    }
  };
  const handleCancelJob = (id: string) => {
    uploadAbortControllersRef.current.get(id)?.abort();
    uploadAbortControllersRef.current.delete(id);
    downloadAbortControllersRef.current.get(id)?.abort();
    downloadAbortControllersRef.current.delete(id);
    setDownloadJobs(prev => prev.map(j => j.id === id ? cancelTransferJob(j) : j));
  };
  const runDownloadTransfer = (
    job: DownloadJob,
    downloadController: AbortController,
    resumeExistingProgress = false,
  ) => {
    const grant = resumeExistingProgress ? downloadGrantFromJob(job) : undefined;
    const prepareDownload =
      resumeExistingProgress && grant && isDownloadGrantStillValid(job)
        ? Promise.resolve(grant)
        : (() => {
            const retryFiles = createRetryFilesForDownloadJob(job);
            return job.downloadKind === 'bundle' || retryFiles.length > 1 || retryFiles[0].type === 'folder'
              ? fileService.createDownloadPackage(retryFiles, job.fileName, {
                  signal: downloadController.signal,
                })
              : fileService.createDownloadUrl(retryFiles[0], {
                  signal: downloadController.signal,
                });
          })();

  prepareDownload
      .then((download) => {
        if (downloadAbortControllersRef.current.get(job.id) !== downloadController) {
          return;
        }
        return runManagedDownloadTransfer({
          job,
          grant: download,
          signal: downloadController.signal,
          resumeExistingProgress,
          onJobUpdate: (updater) => {
            setDownloadJobs((prev) =>
              prev.map((item) => (item.id === job.id ? updater(item) : item)),
            );
          },
          onOpenExternal: onOpenExternal,
          saveDownloadStream: hostDownloadStream,
        });
      })
      .catch((err) => {
        if (isDriveAbortError(err)) {
          return;
        }
        if (downloadAbortControllersRef.current.get(job.id) !== downloadController) {
          return;
        }
        setDownloadJobs(prev =>
          prev.map(item =>
            item.id === job.id
              ? applyTransferFailure(item, err?.message || t('transfer.retryTransferFailed'))
              : item,
          ),
        );
      })
      .finally(() => {
        releaseDownloadAbortController(job.id, downloadController);
      });
  };
  const handlePauseJob = (id: string) => {
    const job = downloadJobs.find((item) => item.id === id);
    if (!job || job.type !== 'download') {
      return;
    }
    downloadAbortControllersRef.current.get(id)?.abort();
    downloadAbortControllersRef.current.delete(id);
    setDownloadJobs((prev) =>
      prev.map((item) => (item.id === id ? pauseTransferJob(item) : item)),
    );
  };
  const handleResumeJob = (id: string) => {
    const job = downloadJobs.find((item) => item.id === id);
    if (!job || job.type !== 'download' || job.status !== 'paused') {
      return;
    }
    downloadAbortControllersRef.current.get(job.id)?.abort();
    const downloadController = createDownloadAbortController(job.id);
    runDownloadTransfer(job, downloadController, true);
  };
  const handleRetryJob = (job: DownloadJob) => {
    if (job.type === 'upload') {
      const uploadSection = job.uploadSection;
      if (!uploadSection) {
        setDownloadJobs(prev =>
          prev.map(item =>
            item.id === job.id
              ? applyTransferFailure(
                  item,
                  t('transfer.uploadDestinationUnavailable'),
                )
              : item,
          ),
        );
        return;
      }

      const retryUpload = (
        sourceFile: File | NativeLocalUploadFile,
        metadata: {
          fileName: string;
          mimeType?: string;
          totalSize: number;
          uploadFileFingerprint: string;
          uploadBlob?: File;
          uploadLocalPath?: string;
        },
      ) => {
        uploadAbortControllersRef.current.get(job.id)?.abort();
        const uploadController = createUploadAbortController(job.id);
        setDownloadJobs(prev =>
          prev.map(item =>
            item.id === job.id
              ? {
                  ...item,
                  fileName: metadata.fileName || item.fileName,
                  mimeType: metadata.mimeType || item.mimeType,
                  totalSize: metadata.totalSize > 0 ? metadata.totalSize : item.totalSize,
                  uploadFileFingerprint: metadata.uploadFileFingerprint,
                  uploadLocalPath: metadata.uploadLocalPath,
                  status: 'uploading',
                  progress: 0,
                  downloadedSize: 0,
                  speed: TRANSFER_SPEED_UPLOADING,
                  timeRemaining: TRANSFER_TIME_WAITING_BACKEND,
                  errorMessage: undefined,
                  uploadBlob: metadata.uploadBlob,
                }
              : item,
          ),
        );

        fileService.uploadFile(sourceFile, uploadSection, job.uploadParentId, {
          taskId: job.id,
          signal: uploadController.signal,
          onProgress: (uploadedBytes, totalBytes) => {
            if (uploadAbortControllersRef.current.get(job.id) !== uploadController) {
              return;
            }
            setDownloadJobs(prev =>
              prev.map(item =>
                item.id === job.id
                  ? {
                      ...item,
                      downloadedSize: uploadedBytes,
                      totalSize: totalBytes > 0 ? totalBytes : item.totalSize,
                      progress:
                        totalBytes > 0
                          ? Math.min(100, Math.max(0, Math.round((uploadedBytes / totalBytes) * 100)))
                          : item.progress,
                      status: 'uploading',
                      speed: TRANSFER_SPEED_UPLOADING,
                      timeRemaining: uploadedBytes >= totalBytes ? TRANSFER_TIME_FINALIZING : TRANSFER_TIME_CALCULATING,
                    }
                  : item,
              ),
            );
          },
        })
          .then((uploadedFile) => {
            if (uploadAbortControllersRef.current.get(job.id) !== uploadController) {
              return;
            }
            setDownloadJobs(prev =>
              prev.map(item =>
                item.id === job.id
                  ? {
                      ...item,
                      fileId: uploadedFile.id,
                      fileName: uploadedFile.name || item.fileName,
                      mimeType: uploadedFile.mimeType || item.mimeType,
                      totalSize: uploadedFile.size || item.totalSize,
                      downloadedSize: uploadedFile.size || item.totalSize,
                      progress: 100,
                      status: 'completed',
                      speed: '--',
                      timeRemaining: '',
                      errorMessage: undefined,
                      sourceNodeIds: [uploadedFile.id],
                    }
                  : item,
              ),
            );
          })
          .catch((err) => {
            if (isDriveAbortError(err)) {
              return;
            }
            if (uploadAbortControllersRef.current.get(job.id) !== uploadController) {
              return;
            }
            setDownloadJobs(prev =>
              prev.map(item =>
                item.id === job.id
                  ? applyTransferFailure(item, err?.message || t('transfer.retryUploadFailed'))
                  : item,
              ),
            );
          })
          .finally(() => {
            releaseUploadAbortController(job.id, uploadController);
          });
      };

      if (job.uploadLocalPath) {
        void host.describeLocalUploadFile(job.uploadLocalPath)
          .then((descriptor) => {
            if (
              job.uploadFileFingerprint &&
              buildNativeUploadJobFingerprint(descriptor) !== job.uploadFileFingerprint
            ) {
              setDownloadJobs(prev =>
                prev.map(item =>
                  item.id === job.id
                    ? applyTransferFailure(item, t('transfer.uploadRetryMismatch', getUploadRetryMismatchContext(item)))
                    : item,
                ),
              );
              return;
            }
            retryUpload(new NativeLocalUploadFile(descriptor, host), {
              fileName: descriptor.name,
              mimeType: descriptor.mimeType,
              totalSize: descriptor.size,
              uploadFileFingerprint: buildNativeUploadJobFingerprint(descriptor),
              uploadLocalPath: descriptor.path,
            });
          })
          .catch((err) => {
            setDownloadJobs(prev =>
              prev.map(item =>
                item.id === job.id
                  ? applyTransferFailure(
                      item,
                      err instanceof Error
                        ? err.message
                        : t('transfer.uploadLocalFileUnavailable'),
                    )
                  : item,
              ),
            );
          });
        return;
      }

      if (!job.uploadBlob) {
        const picker = document.createElement('input');
        picker.type = 'file';
        picker.onchange = () => {
          const selected = picker.files?.[0];
          if (!selected) {
            setDownloadJobs(prev =>
              prev.map(item =>
                item.id === job.id
                  ? applyTransferFailure(
                      item,
                      t('transfer.retryUploadCancelledNoFile'),
                    )
                  : item,
              ),
            );
            return;
          }
          if (!isRetryUploadFileCompatible(job, selected)) {
            setDownloadJobs(prev =>
              prev.map(item =>
                item.id === job.id
                  ? applyTransferFailure(
                      item,
                      t('transfer.uploadRetryMismatch', getUploadRetryMismatchContext(item)),
                    )
                  : item,
              ),
            );
            return;
          }
          retryUpload(selected, {
            fileName: selected.name,
            mimeType: selected.type,
            totalSize: selected.size,
            uploadFileFingerprint: `${selected.name}:${selected.size}:${selected.lastModified}`,
            uploadBlob: selected,
          });
        };
        picker.click();
      } else {
        retryUpload(job.uploadBlob, {
          fileName: job.uploadBlob.name,
          mimeType: job.uploadBlob.type,
          totalSize: job.uploadBlob.size,
          uploadFileFingerprint: `${job.uploadBlob.name}:${job.uploadBlob.size}:${job.uploadBlob.lastModified}`,
          uploadBlob: job.uploadBlob,
        });
      }
      return;
    }

    const retryFiles = createRetryFilesForDownloadJob(job);
    downloadAbortControllersRef.current.get(job.id)?.abort();
    const downloadController = createDownloadAbortController(job.id);
    setDownloadJobs(prev =>
      prev.map(item =>
        item.id === job.id
          ? {
              ...item,
              status: 'connecting',
              progress: 0,
              downloadedSize: 0,
              speed: 'Connecting...',
              timeRemaining: 'Calculating...',
              errorMessage: undefined,
            }
          : item,
      ),
    );
    runDownloadTransfer(job, downloadController, false);
  };

  return (
    <div className="relative flex min-h-0 min-w-0 flex-1 h-full w-full overflow-hidden bg-white dark:bg-[#111]">
      {pendingShareClaimToken && (
        <div className="absolute top-6 left-1/2 transform -translate-x-1/2 z-50 flex max-w-lg flex-col gap-3 px-4 py-3 rounded-lg shadow-xl border text-sm animate-in fade-in slide-in-from-top-4 duration-300 bg-white dark:bg-[#252525] border-blue-100 dark:border-blue-900/40 text-gray-900 dark:text-gray-100">
          <p>{t('fileBrowser.shareLinkClaimPrompt')}</p>
          <div className="flex items-center justify-end gap-2">
            <button
              type="button"
              className="px-3 py-1.5 rounded-md text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-neutral-800"
              onClick={handleDeclineShareClaim}
              disabled={shareClaimInFlight}
            >
              {t('fileBrowser.shareLinkClaimDecline')}
            </button>
            <button
              type="button"
              className="px-3 py-1.5 rounded-md text-sm bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-60"
              onClick={handleAcceptShareClaim}
              disabled={shareClaimInFlight}
            >
              {shareClaimInFlight ? t('fileBrowser.shareLinkClaimAccepting') : t('fileBrowser.shareLinkClaimAccept')}
            </button>
          </div>
        </div>
      )}
      {toast && (
        <div className="absolute top-6 left-1/2 transform -translate-x-1/2 z-50 flex items-center gap-2.5 px-4 py-3 rounded-lg shadow-xl border text-sm animate-in fade-in slide-in-from-top-4 duration-300 bg-white dark:bg-[#252525] border-gray-100 dark:border-neutral-800 text-gray-900 dark:text-gray-100">
          {toast.type === 'success' ? (
            <CheckCircle2 className="text-emerald-500 shrink-0" size={18} />
          ) : (
            <AlertCircle className="text-red-500 shrink-0" size={18} />
          )}
          <span>{toast.message}</span>
        </div>
      )}
      <FileSidebar 
        activeSection={activeSection} 
        onSectionChange={setActiveSection} 
        sharedSpaces={sharedSpaces}
        knowledgeBaseSpaces={knowledgeBaseSpaces}
        storageSummary={storageSummary}
        storageSummaryUnavailable={storageSummaryUnavailable}
        onAddSpaceClick={() => setIsCreateSpaceOpen(true)}
        onDeleteSpace={handleDeleteSpace}
        onOpenStorageSettings={onOpenStorageSettings}
      />
      {activeSection === 'transfer' ? (
        <React.Suspense fallback={<DriveTransferFallback />}>
          <TransferPage 
            downloadJobs={downloadJobs}
            setDownloadJobs={setDownloadJobs}
            onOpenDownload={onOpenExternal}
            onRetryJob={handleRetryJob}
            onCancelJob={handleCancelJob}
            onPauseJob={handlePauseJob}
            onResumeJob={handleResumeJob}
          />
        </React.Suspense>
      ) : (
        <FileBrowser 
          activeSection={activeSection} 
          fileService={fileService}
          openRequest={openRequest}
          downloadJobs={downloadJobs}
          setDownloadJobs={setDownloadJobs}
          onOpenDownload={onOpenExternal}
          onRetryJob={handleRetryJob}
          createUploadAbortController={createUploadAbortController}
          releaseUploadAbortController={releaseUploadAbortController}
          createDownloadAbortController={createDownloadAbortController}
          releaseDownloadAbortController={releaseDownloadAbortController}
          onCancelJob={handleCancelJob}
          onOpenRequestHandled={onOpenRequestHandled}
        />
      )}

      {/* Shared Space Creation Modal */}
      <CreateSharedSpaceModal 
        isOpen={isCreateSpaceOpen}
        onClose={() => setIsCreateSpaceOpen(false)}
        onSubmit={handleCreateSpaceSubmit}
      />
    </div>
  );
}

function DriveTransferFallback() {
  const { t } = useTranslation();
  return (
    <div
      aria-label={t('transfer.loadingTransferCenter')}
      className="flex h-full min-h-0 min-w-0 flex-1 items-center justify-center bg-white dark:bg-[#151515]"
    >
      <div className="h-6 w-6 rounded-full border-2 border-blue-500 border-t-transparent animate-spin" />
    </div>
  );
}
