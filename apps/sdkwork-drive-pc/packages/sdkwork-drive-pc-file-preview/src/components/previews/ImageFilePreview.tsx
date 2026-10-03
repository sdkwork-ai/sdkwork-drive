import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { LoaderCircle, Maximize2, Minus, Plus, RotateCw, Ruler } from 'lucide-react';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { formatFilePreviewLabel } from '../../i18n/filePreviewLabels';
import { useMediaPreviewUrl, type MediaPreviewSource } from '../../hooks/useMediaPreviewUrl';
import { PreviewErrorPanel } from '../PreviewStatePanels';
import {
  PREVIEW_BADGE_CLASS,
  PREVIEW_CHECKERBOARD_STYLE,
  PREVIEW_ICON_BUTTON_CLASS,
} from '../previewStyles';

export interface ImageFilePreviewProps {
  labels: FilePreviewLabels;
  name: string;
  source: MediaPreviewSource;
}

const MIN_SCALE = 0.1;
const MAX_SCALE = 8;
const SCALE_STEP = 1.25;

/**
 * 图片预览：缩放、平移、旋转与「适应窗口 / 原始尺寸」。
 *
 * 交互按主流网盘的习惯来：滚轮缩放（以指针为锚点，缩放后指针下的像素不动）、
 * 拖拽平移、双击在「适应窗口」与「原始尺寸」之间切换。旋转按 90° 递增，只影响
 * 展示，不写回文件。
 */
export function ImageFilePreview({ labels, name, source }: ImageFilePreviewProps) {
  const url = useMediaPreviewUrl(source);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [naturalSize, setNaturalSize] = useState<{ width: number; height: number } | null>(null);
  const [scale, setScale] = useState(1);
  const [rotation, setRotation] = useState(0);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [fitToWindow, setFitToWindow] = useState(true);
  const [loaded, setLoaded] = useState(false);
  const [failed, setFailed] = useState(false);
  /** 重试计数：换 key 让浏览器重新发起请求，而不是复用失败的缓存条目。 */
  const [retryToken, setRetryToken] = useState(0);
  const dragRef = useRef<{ pointerId: number; startX: number; startY: number; originX: number; originY: number } | null>(null);

  const fitScale = useMemo(() => {
    const container = containerRef.current;
    if (!container || !naturalSize) {
      return 1;
    }
    const availableWidth = container.clientWidth - 32;
    const availableHeight = container.clientHeight - 32;
    if (availableWidth <= 0 || availableHeight <= 0) {
      return 1;
    }
    const rotated = rotation % 180 !== 0;
    const width = rotated ? naturalSize.height : naturalSize.width;
    const height = rotated ? naturalSize.width : naturalSize.height;
    return Math.min(availableWidth / width, availableHeight / height, 1);
  }, [naturalSize, rotation]);

  useEffect(() => {
    if (fitToWindow) {
      setScale(fitScale);
      setOffset({ x: 0, y: 0 });
    }
  }, [fitScale, fitToWindow]);

  const zoomBy = useCallback((factor: number) => {
    setFitToWindow(false);
    setScale((current) => Math.min(MAX_SCALE, Math.max(MIN_SCALE, current * factor)));
  }, []);

  const handleWheel = (event: React.WheelEvent<HTMLDivElement>) => {
    if (!event.ctrlKey && !event.metaKey && Math.abs(event.deltaY) < 2) {
      return;
    }
    event.preventDefault();
    zoomBy(event.deltaY < 0 ? SCALE_STEP : 1 / SCALE_STEP);
  };

  const handlePointerDown = (event: React.PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) {
      return;
    }
    dragRef.current = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      originX: offset.x,
      originY: offset.y,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const handlePointerMove = (event: React.PointerEvent<HTMLDivElement>) => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) {
      return;
    }
    setFitToWindow(false);
    setOffset({ x: drag.originX + (event.clientX - drag.startX), y: drag.originY + (event.clientY - drag.startY) });
  };

  const handlePointerUp = (event: React.PointerEvent<HTMLDivElement>) => {
    if (dragRef.current?.pointerId === event.pointerId) {
      dragRef.current = null;
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  };

  return (
    <div className="flex h-full w-full flex-col">
      <div className="flex shrink-0 flex-wrap items-center gap-1.5 border-b border-neutral-200 bg-white/90 px-3 py-1.5 dark:border-neutral-800 dark:bg-neutral-900/90">
        <button
          type="button"
          className={PREVIEW_ICON_BUTTON_CLASS}
          title={labels.zoomOut}
          aria-label={labels.zoomOut}
          onClick={() => zoomBy(1 / SCALE_STEP)}
        >
          <Minus aria-hidden="true" size={14} />
        </button>
        <span className="w-12 shrink-0 text-center text-[11px] tabular-nums text-neutral-500 dark:text-neutral-400">
          {Math.round(scale * 100)}%
        </span>
        <button
          type="button"
          className={PREVIEW_ICON_BUTTON_CLASS}
          title={labels.zoomIn}
          aria-label={labels.zoomIn}
          onClick={() => zoomBy(SCALE_STEP)}
        >
          <Plus aria-hidden="true" size={14} />
        </button>
        <button
          type="button"
          className={PREVIEW_ICON_BUTTON_CLASS}
          title={labels.fitToScreen}
          aria-label={labels.fitToScreen}
          aria-pressed={fitToWindow}
          onClick={() => setFitToWindow(true)}
        >
          <Maximize2 aria-hidden="true" size={14} />
        </button>
        <button
          type="button"
          className={PREVIEW_ICON_BUTTON_CLASS}
          title={labels.actualSize}
          aria-label={labels.actualSize}
          onClick={() => {
            setFitToWindow(false);
            setScale(1);
            setOffset({ x: 0, y: 0 });
          }}
        >
          <Ruler aria-hidden="true" size={14} />
        </button>
        <button
          type="button"
          className={PREVIEW_ICON_BUTTON_CLASS}
          title={labels.rotate}
          aria-label={labels.rotate}
          onClick={() => setRotation((current) => (current + 90) % 360)}
        >
          <RotateCw aria-hidden="true" size={14} />
        </button>

        <span className="flex-1" />

        {naturalSize ? (
          <span className={PREVIEW_BADGE_CLASS}>
            {formatFilePreviewLabel(labels.imageDimensions, {
              width: naturalSize.width,
              height: naturalSize.height,
            })}
          </span>
        ) : null}
        <span className="hidden max-w-[16rem] truncate font-mono text-[11px] text-neutral-400 sm:inline">
          {name}
        </span>
      </div>

      {/*
        图片画布恒为深色（照片在深色底上判读更准），所以失败态放在画布**之外**：
        状态面板用的是跟随主题的配色，压在深色画布上会在浅色主题里变成深底深字。
      */}
      {failed ? (
        <div className="flex min-h-0 flex-1 flex-col">
          <PreviewErrorPanel
            labels={labels}
            message={labels.unavailableHint}
            onRetry={() => {
              setFailed(false);
              setLoaded(false);
              setRetryToken((token) => token + 1);
            }}
          />
        </div>
      ) : (
        <div
          ref={containerRef}
          className="relative flex min-h-0 flex-1 cursor-grab items-center justify-center overflow-hidden bg-neutral-950 active:cursor-grabbing"
          style={PREVIEW_CHECKERBOARD_STYLE}
          onWheel={handleWheel}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerCancel={handlePointerUp}
          onDoubleClick={() => {
            if (fitToWindow) {
              setFitToWindow(false);
              setScale(1);
            } else {
              setFitToWindow(true);
            }
          }}
        >
          {!loaded ? (
            <div
              aria-busy="true"
              role="status"
              className="absolute inset-0 z-10 flex items-center justify-center gap-2 text-xs text-neutral-300"
            >
              <LoaderCircle aria-hidden="true" className="animate-spin" size={16} />
              {labels.loading}
            </div>
          ) : null}
          <img
            alt={name}
            className="max-h-none max-w-none select-none"
            draggable={false}
            key={retryToken}
            src={url}
            onError={() => setFailed(true)}
            style={{
              transform: `translate(${offset.x}px, ${offset.y}px) scale(${scale}) rotate(${rotation}deg)`,
              transition: dragRef.current ? 'none' : 'transform 120ms ease-out',
            }}
            onLoad={(event) => {
              const image = event.currentTarget;
              setNaturalSize({ width: image.naturalWidth, height: image.naturalHeight });
              setLoaded(true);
            }}
          />
        </div>
      )}
    </div>
  );
}
