import React, { useState } from 'react';
import { Info } from 'lucide-react';
import { useTranslation } from 'sdkwork-drive-pc-commons';
import type { DriveFile } from 'sdkwork-drive-pc-types';

interface ImageModuleProps {
  file: DriveFile;
  previewUrl: string | null;
  previewError: string | null;
  loading: boolean;
  triggerFeedback: (text: string, type?: 'success' | 'info' | 'error') => void;
}

const IMAGE_FILTERS = ['none', 'sepia', 'grayscale', 'sunset', 'neon'] as const;
type ImageFilter = (typeof IMAGE_FILTERS)[number];

const FILTER_LABEL_KEYS: Record<ImageFilter, string> = {
  none: 'filterNone',
  sepia: 'filterSepia',
  grayscale: 'filterGrayscale',
  sunset: 'filterSunset',
  neon: 'filterNeon',
};

export function ImageModule({
  file,
  previewUrl,
  previewError,
  loading,
  triggerFeedback,
}: ImageModuleProps) {
  const { t } = useTranslation();

  const [imageZoom, setImageZoom] = useState(100);
  const [imageRotate, setImageRotate] = useState(0);
  const [imageFlipH, setImageFlipH] = useState(false);
  const [imageFlipV, setImageFlipV] = useState(false);
  const [activePhotoFilter, setActivePhotoFilter] = useState<ImageFilter>('none');

  const getFilterStyle = () => {
    switch (activePhotoFilter) {
      case 'sepia':
        return 'sepia contrast-125';
      case 'grayscale':
        return 'grayscale brightness-95';
      case 'neon':
        return 'invert hue-rotate-180 brightness-110 saturate-150';
      case 'sunset':
        return 'sepia saturate-200 hue-rotate-340 hover:brightness-105';
      default:
        return '';
    }
  };

  const renderImageBody = () => {
    if (loading) {
      return (
        <div className="flex flex-col items-center justify-center text-xs text-neutral-400 gap-2">
          <div className="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span>{t('previewModules.previewPreparing')}</span>
        </div>
      );
    }
    if (previewError || !previewUrl) {
      return (
        <div className="flex flex-col items-center justify-center text-xs text-neutral-400 gap-3 px-8 text-center">
          <Info size={20} className="text-rose-400" />
          <span>{previewError || t('previewModules.mediaPreviewUnavailable')}</span>
        </div>
      );
    }
    return (
      <img
        src={previewUrl}
        alt={file.name}
        className={`max-w-full max-h-full object-contain select-none pointer-events-none transition-all duration-300 ${getFilterStyle()}`}
        style={{
          filter: getFilterStyle(),
          transform: `scale(${imageZoom / 100}) rotate(${imageRotate}deg) ${imageFlipH ? 'scaleX(-1)' : ''} ${imageFlipV ? 'scaleY(-1)' : ''}`,
        }}
      />
    );
  };

  return (
    <div className="w-full h-full max-w-4xl max-h-[75vh] flex flex-col justify-between gap-4 animate-in zoom-in-95 duration-200">
      <div className="bg-[#181818] border border-neutral-800 p-2 rounded-xl flex items-center justify-between text-xs text-neutral-400 shrink-0 shadow-md">
        <span className="font-bold text-[10.5px] uppercase font-mono pl-2 text-neutral-500">{t('previewModules.applyFilters')}</span>
        <div className="flex items-center gap-1">
          {IMAGE_FILTERS.map((filter) => (
            <button
              key={filter}
              onClick={() => {
                setActivePhotoFilter(filter);
                triggerFeedback(
                  t('previewModules.imageFilterApplied', {
                    filter: t(`previewModules.${FILTER_LABEL_KEYS[filter]}`),
                  }),
                );
              }}
              className={`px-2 py-1 rounded text-[10.5px] capitalize font-medium transition-colors ${activePhotoFilter === filter ? 'bg-blue-600 font-bold text-white shadow-sm' : 'hover:bg-neutral-800 text-neutral-300'}`}
            >
              {t(`previewModules.${FILTER_LABEL_KEYS[filter]}`)}
            </button>
          ))}
        </div>
      </div>

      <div className="flex-1 min-h-0 w-full border border-neutral-800 bg-[#dfdfdf] dark:bg-[#151515] rounded-2xl overflow-hidden shadow-2xl flex items-center justify-center relative group">
        {renderImageBody()}
        <div className="absolute top-3 right-3 bg-black/75 border border-neutral-800/50 px-2.5 py-1 text-[10px] font-mono text-neutral-400 rounded select-none backdrop-blur-md">
          {t('previewModules.imageZoomLabel', { percent: imageZoom })}
        </div>
      </div>

      <div className="bg-[#181818]/60 p-3 rounded-xl border border-neutral-800 flex items-center justify-between text-xs shrink-0 shadow-md">
        <div className="flex items-center gap-2">
          <span className="font-mono text-[10px] text-neutral-500 uppercase">{t('previewModules.zoomSlider')}</span>
          <input
            type="range"
            min="50"
            max="200"
            value={imageZoom}
            onChange={(e) => setImageZoom(parseInt(e.target.value))}
            className="accent-blue-500 w-24 h-1 cursor-pointer bg-neutral-800"
          />
        </div>

        <div className="flex items-center gap-1.5 select-none">
          <button
            onClick={() => {
              setImageRotate((prev) => (prev + 90) % 360);
              triggerFeedback(t('previewModules.imageRotated'));
            }}
            className="p-1 px-2.5 bg-[#252525] hover:bg-[#333] border border-neutral-800 rounded font-mono text-[10px] font-bold text-neutral-300 cursor-pointer text-center"
            title={t('previewModules.imageRotateTitle')}
          >
            {t('previewModules.imageRotate')}
          </button>
          <button
            onClick={() => {
              setImageFlipH(!imageFlipH);
              triggerFeedback(t('previewModules.imageFlipHorizontal'));
            }}
            className="p-1 px-2.5 bg-[#252525] hover:bg-[#333] border border-neutral-800 rounded font-mono text-[10px] font-bold text-neutral-300 cursor-pointer text-center"
            title={t('previewModules.imageFlipHorizontalTitle')}
          >
            {t('previewModules.imageFlipH')}
          </button>
          <button
            onClick={() => {
              setImageFlipV(!imageFlipV);
              triggerFeedback(t('previewModules.imageFlipVertical'));
            }}
            className="p-1 px-2.5 bg-[#252525] hover:bg-[#333] border border-neutral-800 rounded font-mono text-[10px] font-bold text-neutral-300 cursor-pointer text-center"
            title={t('previewModules.imageFlipVerticalTitle')}
          >
            {t('previewModules.imageFlipV')}
          </button>
        </div>
      </div>
    </div>
  );
}
