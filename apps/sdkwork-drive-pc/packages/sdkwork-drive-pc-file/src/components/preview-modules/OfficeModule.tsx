import React from 'react';
import { ExternalLink, FileText, Info, Presentation, Table2 } from 'lucide-react';
import { useTranslation } from 'sdkwork-drive-pc-commons';
import type { DriveFile } from 'sdkwork-drive-pc-types';

interface OfficeModuleProps {
  file: DriveFile;
  previewUrl: string | null;
  previewError: string | null;
  loading: boolean;
  kind: 'document' | 'spreadsheet' | 'presentation';
}

export function OfficeModule({
  file,
  previewUrl,
  previewError,
  loading,
  kind,
}: OfficeModuleProps) {
  const { t } = useTranslation();
  const Icon = kind === 'spreadsheet' ? Table2 : kind === 'presentation' ? Presentation : FileText;
  const kindLabel = kind === 'spreadsheet'
    ? t('previewModules.officeKindSpreadsheet')
    : kind === 'presentation'
      ? t('previewModules.officeKindPresentation')
      : t('previewModules.officeKindDocument');

  return (
    <div className="w-full max-w-3xl bg-[#131315] border border-neutral-800/80 rounded-2xl overflow-hidden shadow-2xl flex flex-col animate-in zoom-in-95 duration-250">
      <div className="h-12 bg-[#1c1c1c] border-b border-neutral-800/60 px-5 flex items-center justify-between shrink-0 select-none text-xs text-neutral-400 font-bold">
        <div className="flex items-center gap-2.5 min-w-0">
          <Icon size={16} className="text-blue-400 shrink-0" />
          <span className="truncate">{file.name}</span>
        </div>
        <div className="text-[10px] uppercase font-mono tracking-widest text-neutral-600 font-black">
          {t('previewModules.officeDriveKind', { kind: kindLabel })}
        </div>
      </div>

      <div className="p-8 flex flex-col items-center justify-center gap-5 text-center min-h-[320px]">
        {loading ? (
          <>
            <div className="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
            <p className="text-xs text-neutral-400">{t('previewModules.previewGrantPreparing')}</p>
          </>
        ) : previewError || !previewUrl ? (
          <>
            <Info size={22} className="text-rose-400" />
            <p className="text-xs text-neutral-400 max-w-md">{previewError || t('previewModules.mediaPreviewUnavailable')}</p>
          </>
        ) : (
          <>
            <div className="w-16 h-16 rounded-2xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400">
              <Icon size={30} />
            </div>
            <div>
              <h4 className="text-sm font-bold text-neutral-100">{t('previewModules.officeReadyTitle', { kind: kindLabel })}</h4>
              <p className="text-xs text-neutral-500 leading-relaxed mt-2 max-w-md">
                {t('previewModules.officeReadyDesc')}
              </p>
            </div>
            <a
              href={previewUrl}
              target="_blank"
              rel="noreferrer"
              className="px-3.5 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-xs font-bold flex items-center gap-1.5 transition-colors"
            >
              <ExternalLink size={13} />
              {t('previewModules.officeOpenFile')}
            </a>
          </>
        )}
      </div>
    </div>
  );
}
