import React, { useEffect, useRef, useState } from 'react';
import { Info, Pause, Play, RotateCw } from 'lucide-react';
import { useTranslation } from 'sdkwork-drive-pc-commons';
import type { DriveFile } from 'sdkwork-drive-pc-types';

interface AudioModuleProps {
  file: DriveFile;
  previewUrl: string | null;
  previewError: string | null;
  loading: boolean;
}

export function AudioModule({ file, previewUrl, previewError, loading }: AudioModuleProps) {
  const { t } = useTranslation();
  const audioRef = useRef<HTMLAudioElement>(null);

  const [audioPlayState, setAudioPlayState] = useState(false);
  const [audioTimestamp, setAudioTimestamp] = useState(0);
  const [audioLoop, setAudioLoop] = useState(true);

  useEffect(() => {
    if (audioPlayState) {
      audioRef.current?.play().catch(() => setAudioPlayState(false));
    } else {
      audioRef.current?.pause();
    }
  }, [audioPlayState]);

  const formatTime = (seconds: number) => {
    const minutes = Math.floor(seconds / 60);
    const remaining = Math.floor(seconds % 60);
    return `${minutes}:${String(remaining).padStart(2, '0')}`;
  };

  const renderAudioElement = () => {
    if (loading) {
      return (
        <div className="h-24 flex flex-col items-center justify-center text-xs text-neutral-400 gap-2">
          <div className="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span>{t('previewModules.previewPreparing')}</span>
        </div>
      );
    }
    if (previewError || !previewUrl) {
      return (
        <div className="h-24 flex flex-col items-center justify-center text-xs text-neutral-400 gap-3 px-6 text-center">
          <Info size={20} className="text-rose-400" />
          <span>{previewError || t('previewModules.audioPreviewUnavailable')}</span>
        </div>
      );
    }
    return (
      <audio
        ref={audioRef}
        src={previewUrl}
        onTimeUpdate={() => setAudioTimestamp(Math.round(audioRef.current?.currentTime || 0))}
        onEnded={() => {
          if (!audioLoop) {
            setAudioPlayState(false);
          }
        }}
        loop={audioLoop}
      />
    );
  };

  return (
    <div className="w-full max-w-sm bg-[#161616] border border-neutral-800 rounded-3xl p-6.5 text-center shadow-2xl space-y-6 flex flex-col justify-between animate-in zoom-in-95 duration-200">
      {renderAudioElement()}

      <div className="relative mx-auto mt-2 select-none pointer-events-none">
        <div className={`w-36 h-36 rounded-full bg-neutral-900 border-[8px] border-neutral-800/80 shadow-2xl flex items-center justify-center relative ${audioPlayState ? 'animate-spin' : ''}`} style={{ animationDuration: '6s' }}>
          <div className="w-14 h-14 rounded-full bg-blue-500/20 border-2 border-dashed border-blue-500 flex items-center justify-center">
            <div className="w-4 h-4 rounded-full bg-black border border-neutral-850" />
          </div>
        </div>
        <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 text-[10.5px] font-bold text-neutral-500/80 uppercase tracking-widest font-mono">
          {t('previewModules.vinylLp') || 'AUDIO'}
        </div>
      </div>

      <div>
        <h5 className="font-extrabold text-gray-100 text-sm">{file.name}</h5>
        <p className="text-xs text-neutral-500 mt-0.5">{t('previewModules.stereoPlayback') || 'Drive audio preview'}</p>
      </div>

      <div className="space-y-1 px-1">
        <div
          className="w-full bg-neutral-800 h-1.5 rounded-full relative cursor-pointer"
          onClick={(e) => {
            if (!audioRef.current || Number.isNaN(audioRef.current.duration)) return;
            const rect = e.currentTarget.getBoundingClientRect();
            const pct = (e.clientX - rect.left) / rect.width;
            const dest = pct * audioRef.current.duration;
            audioRef.current.currentTime = dest;
            setAudioTimestamp(Math.round(dest));
          }}
        >
          <div
            className="absolute left-0 top-0 bg-blue-500 h-full rounded-full"
            style={{ width: `${Math.min(100, (audioTimestamp / (audioRef.current?.duration || 1)) * 100)}%` }}
          />
        </div>
        <div className="flex items-center justify-between font-mono text-[9.5px] text-neutral-500">
          <span>{formatTime(audioTimestamp)}</span>
          <span>
            {audioRef.current && !Number.isNaN(audioRef.current.duration)
              ? formatTime(audioRef.current.duration)
              : '--:--'}
          </span>
        </div>
      </div>

      <div className="flex items-center justify-center gap-5">
        <button
          onClick={() => setAudioPlayState(!audioPlayState)}
          className="p-3.5 bg-blue-600 hover:bg-blue-700 text-white rounded-full transition-transform active:scale-95 cursor-pointer shadow-lg outline-none"
        >
          {audioPlayState ? <Pause size={17} className="fill-current" /> : <Play size={17} className="fill-current ml-0.5" />}
        </button>

        <button
          onClick={() => setAudioLoop(!audioLoop)}
          className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${audioLoop ? 'text-emerald-400' : 'text-neutral-500'}`}
          title="Loop track repeat toggle"
        >
          <RotateCw size={14} />
        </button>
      </div>
    </div>
  );
}
