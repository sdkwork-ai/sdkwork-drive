import React, { useEffect, useRef, useState } from 'react';
import { Info, Pause, Play, Volume2, VolumeX } from 'lucide-react';
import { useTranslation } from 'sdkwork-drive-pc-commons';
import type { DriveFile } from 'sdkwork-drive-pc-types';

interface VideoModuleProps {
  file: DriveFile;
  previewUrl: string | null;
  previewError: string | null;
  loading: boolean;
}

export function VideoModule({ file, previewUrl, previewError, loading }: VideoModuleProps) {
  const { t } = useTranslation();
  const videoRef = useRef<HTMLVideoElement>(null);

  const [videoPlayState, setVideoPlayState] = useState(false);
  const [videoTimestamp, setVideoTimestamp] = useState(0);
  const [videoVolume, setVideoVolume] = useState(80);
  const [videoMute, setVideoMute] = useState(false);
  const [videoSpeed, setVideoSpeed] = useState(1);

  useEffect(() => {
    if (!videoRef.current) return;
    videoRef.current.playbackRate = videoSpeed;
  }, [videoSpeed]);

  useEffect(() => {
    if (!videoRef.current) return;
    videoRef.current.volume = videoVolume / 100;
    videoRef.current.muted = videoMute;
  }, [videoMute, videoVolume]);

  useEffect(() => {
    if (videoPlayState) {
      videoRef.current?.play().catch(() => setVideoPlayState(false));
    } else {
      videoRef.current?.pause();
    }
  }, [videoPlayState]);

  const formatTime = (seconds: number) => {
    const minutes = Math.floor(seconds / 60);
    const remaining = Math.floor(seconds % 60);
    return `${minutes}:${String(remaining).padStart(2, '0')}`;
  };

  const renderVideoBody = () => {
    if (loading) {
      return (
        <div className="absolute inset-0 flex flex-col items-center justify-center text-xs text-neutral-400 gap-2">
          <div className="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span>{t('previewModules.videoPreviewPreparing')}</span>
        </div>
      );
    }
    if (previewError || !previewUrl) {
      return (
        <div className="absolute inset-0 flex flex-col items-center justify-center text-xs text-neutral-400 gap-3 px-8 text-center">
          <Info size={20} className="text-rose-400" />
          <span>{previewError || t('previewModules.videoPreviewUnavailable')}</span>
        </div>
      );
    }
    return (
      <video
        ref={videoRef}
        src={previewUrl}
        onTimeUpdate={() => setVideoTimestamp(Math.round(videoRef.current?.currentTime || 0))}
        onEnded={() => setVideoPlayState(false)}
        onClick={() => setVideoPlayState((value) => !value)}
        className="absolute inset-0 w-full h-full object-contain cursor-pointer z-10"
        playsInline
      />
    );
  };

  return (
    <div className="w-full h-full max-w-5xl max-h-[75vh] bg-[#0a0a0a] border border-neutral-900 rounded-2xl overflow-hidden aspect-video shadow-2xl relative flex flex-col justify-end group select-none flex-1 min-h-0">
      {renderVideoBody()}

      {!loading && previewUrl && !previewError && !videoPlayState && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/25 z-20 pointer-events-none">
          <button
            className="w-14 h-14 bg-blue-600/95 border border-blue-500 rounded-full flex items-center justify-center text-white scale-95 group-hover:scale-100 transition-all cursor-pointer shadow-xl pointer-events-auto"
            onClick={() => setVideoPlayState(true)}
          >
            <Play size={24} className="fill-current ml-1" />
          </button>
        </div>
      )}

      <div className="absolute top-4 left-4 bg-black/60 border border-neutral-800 px-2.5 py-1 rounded text-[10px] font-mono text-neutral-400 z-20">
        <span className="text-white">{file.name}</span>
      </div>

      <div className="bg-gradient-to-t from-black/90 to-transparent p-4 shrink-0 space-y-3 z-30 relative">
        <div className="space-y-1">
          <div className="flex items-center justify-between text-[10.5px] font-mono text-neutral-400 select-none">
            <span>0:00</span>
            <span>{t('previewModules.stereoPlayback') || 'Drive media preview'}</span>
            <span>
              {videoRef.current && !Number.isNaN(videoRef.current.duration)
                ? formatTime(videoRef.current.duration)
                : '--:--'}
            </span>
          </div>
          <div
            className="w-full bg-neutral-800/80 h-1.5 rounded-full relative cursor-pointer"
            onClick={(e) => {
              if (!videoRef.current || Number.isNaN(videoRef.current.duration)) return;
              const rect = e.currentTarget.getBoundingClientRect();
              const pct = (e.clientX - rect.left) / rect.width;
              const dest = pct * videoRef.current.duration;
              videoRef.current.currentTime = dest;
              setVideoTimestamp(Math.round(dest));
            }}
          >
            <div
              className="absolute left-0 top-0 h-full bg-blue-500 rounded-full"
              style={{ width: `${Math.min(100, (videoTimestamp / (videoRef.current?.duration || 1)) * 100)}%` }}
            />
          </div>
        </div>

        <div className="flex items-center justify-between text-xs font-semibold text-neutral-300">
          <div className="flex items-center gap-3">
            <button
              onClick={() => setVideoPlayState(!videoPlayState)}
              className="hover:text-white cursor-pointer"
            >
              {videoPlayState ? <Pause size={15} /> : <Play size={15} />}
            </button>
            <div className="w-px h-3.5 bg-neutral-800" />
            <div className="flex items-center gap-1.5">
              <button onClick={() => setVideoMute(!videoMute)} className="hover:text-white cursor-pointer">
                {videoMute || videoVolume === 0 ? <VolumeX size={14} /> : <Volume2 size={14} />}
              </button>
              <input
                type="range"
                min="0"
                max="100"
                value={videoMute ? 0 : videoVolume}
                onChange={(e) => {
                  setVideoVolume(parseInt(e.target.value));
                  setVideoMute(false);
                }}
                className="w-16 accent-blue-500 cursor-pointer h-1"
              />
            </div>
          </div>

          <div className="flex items-center gap-3">
            <select
              value={videoSpeed}
              onChange={(e: any) => setVideoSpeed(parseFloat(e.target.value))}
              className="bg-neutral-900 border border-neutral-800 text-[10px] rounded px-1 text-neutral-300 outline-none"
            >
              <option value="0.5">0.5x</option>
              <option value="1">1.0x Normal</option>
              <option value="1.5">1.5x</option>
              <option value="2">2.0x Fast</option>
            </select>
            <span className="font-mono text-[11px] text-neutral-400">
              {formatTime(videoTimestamp)} / {videoRef.current && !Number.isNaN(videoRef.current.duration) ? formatTime(videoRef.current.duration) : '--:--'}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
