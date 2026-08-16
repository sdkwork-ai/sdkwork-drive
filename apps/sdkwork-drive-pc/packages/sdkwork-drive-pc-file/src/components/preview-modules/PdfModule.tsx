import React, { useEffect, useRef, useState } from 'react';
import { ExternalLink, Info, Minus, Plus } from 'lucide-react';
import { useTranslation } from 'sdkwork-drive-pc-commons';
import type { DriveFile } from 'sdkwork-drive-pc-types';
import { isDriveAbortError, type DriveFileService } from 'sdkwork-drive-pc-core';

interface PdfModuleProps {
  file: DriveFile;
  fileService: DriveFileService;
  previewUrl: string | null;
  previewError: string | null;
  loading: boolean;
  triggerFeedback: (text: string, type?: 'success' | 'info' | 'error') => void;
  isReadOnly?: boolean;
}

export function PdfModule({
  file,
  fileService,
  previewUrl,
  previewError,
  loading,
  triggerFeedback,
  isReadOnly = false,
}: PdfModuleProps) {
  const { t } = useTranslation();
  const [pdfZoom, setPdfZoom] = useState(100);
  const [signing, setSigning] = useState(false);
  const previewFrameRef = useRef<HTMLIFrameElement | null>(null);
  const signAbortControllerRef = useRef<AbortController | null>(null);

  useEffect(() => {
    signAbortControllerRef.current?.abort();
    signAbortControllerRef.current = null;
    setSigning(false);

    return () => {
      signAbortControllerRef.current?.abort();
      signAbortControllerRef.current = null;
    };
  }, [file.id]);

  const handleSign = async () => {
    if (isReadOnly || signing) return;
    signAbortControllerRef.current?.abort();
    const signAbortController = new AbortController();
    signAbortControllerRef.current = signAbortController;
    setSigning(true);
    try {
      await fileService.signPdfFile(file, {
        signal: signAbortController.signal,
      });
      if (signAbortControllerRef.current !== signAbortController) {
        return;
      }
      triggerFeedback(t('previewModules.pdfAcknowledged'), 'success');
    } catch (err: any) {
      if (isDriveAbortError(err)) {
        return;
      }
      if (signAbortControllerRef.current !== signAbortController) {
        return;
      }
      triggerFeedback(err?.message || t('previewModules.pdfAcknowledgementFailed'), 'error');
    } finally {
      if (signAbortControllerRef.current === signAbortController) {
        signAbortControllerRef.current = null;
        setSigning(false);
      }
    }
  };

  const handlePrint = () => {
    if (!previewUrl) {
      triggerFeedback(previewError || t('previewModules.pdfPreviewUnavailable'), 'error');
      return;
    }
    try {
      previewFrameRef.current?.contentWindow?.focus();
      previewFrameRef.current?.contentWindow?.print();
    } catch {
      window.print();
    }
    triggerFeedback(t('previewModules.pdfPrintOpened'), 'info');
  };

  const renderPdfBody = () => {
    if (loading) {
      return (
        <div className="w-full h-full flex flex-col items-center justify-center text-xs text-neutral-400 gap-2">
          <div className="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span>{t('previewModules.pdfPreviewPreparing')}</span>
        </div>
      );
    }
    if (previewError || !previewUrl) {
      return (
        <div className="w-full h-full flex flex-col items-center justify-center text-xs text-neutral-400 gap-3 px-8 text-center">
          <Info size={20} className="text-rose-400" />
          <span>{previewError || t('previewModules.pdfPreviewUnavailable')}</span>
        </div>
      );
    }
    return (
      <iframe
        ref={previewFrameRef}
        title={file.name}
        src={previewUrl}
        className="w-full h-full bg-white border-0"
        style={{
          transform: `scale(${pdfZoom / 100})`,
          transformOrigin: 'top center',
          width: `${10000 / pdfZoom}%`,
          height: `${10000 / pdfZoom}%`,
        }}
      />
    );
  };

  return (
    <div className="w-full max-w-5xl bg-[#131315] border border-neutral-800/80 rounded-2xl overflow-hidden shadow-2xl flex flex-col h-full max-h-[78vh] animate-in zoom-in-95 duration-250">
      <div className="h-14 bg-[#1e1e1e] border-b border-neutral-800/70 px-5 flex items-center justify-between shrink-0 select-none shadow-sm z-10">
        <div className="min-w-0">
          <div className="text-xs font-bold text-neutral-200 truncate">{file.name}</div>
          <div className="text-[10px] font-mono text-neutral-500 mt-0.5">Drive App SDK preview grant</div>
        </div>

        <div className="flex items-center gap-2.5">
          <button
            onClick={() => setPdfZoom((value) => Math.max(50, value - 25))}
            className="p-1.5 bg-[#252525] hover:bg-neutral-800 rounded text-neutral-300 cursor-pointer"
            title={t('previewModules.zoomOut')}
          >
            <Minus size={13} />
          </button>
          <span className="text-[11px] font-mono text-neutral-400 w-12 text-center">{pdfZoom}%</span>
          <button
            onClick={() => setPdfZoom((value) => Math.min(200, value + 25))}
            className="p-1.5 bg-[#252525] hover:bg-neutral-800 rounded text-neutral-300 cursor-pointer"
            title={t('previewModules.zoomIn')}
          >
            <Plus size={13} />
          </button>
          {!isReadOnly && (
            <button
              onClick={() => void handleSign()}
              disabled={signing}
              className="px-3 py-1.5 bg-neutral-800 hover:bg-neutral-700 disabled:opacity-60 disabled:cursor-not-allowed text-white border border-neutral-700 font-bold rounded-lg text-xs transition-colors cursor-pointer"
            >
              {signing ? t('previewModules.signing') : t('previewModules.sign')}
            </button>
          )}
          <button
            onClick={handlePrint}
            className="px-3 py-1.5 bg-neutral-800 hover:bg-neutral-700 text-white border border-neutral-700 font-bold rounded-lg text-xs transition-colors cursor-pointer"
          >
            {t('previewModules.print')}
          </button>
          {previewUrl && (
            <a
              href={previewUrl}
              target="_blank"
              rel="noreferrer"
              className="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-lg text-xs transition-colors cursor-pointer flex items-center gap-1.5"
            >
              <ExternalLink size={13} />
              Open
            </a>
          )}
        </div>
      </div>

      <div className="flex-1 bg-[#222222] overflow-auto">
        {renderPdfBody()}
      </div>
    </div>
  );
}
