import type React from 'react';

/**
 * 预览组件共用的样式词表。
 *
 * 这个包不依赖宿主的 UI 基础库（管理端与 drive 控制台各自有自己的一套），所以
 * 预览层自带一份最小、稳定的类名集合：所有预览共享同一套按钮、徽标、表面与滚动
 * 容器，视觉一致靠这份词表，而不是每个预览各写一套 class。
 */

export const PREVIEW_SURFACE_CLASS =
  'flex min-h-0 flex-1 flex-col overflow-hidden bg-neutral-50 dark:bg-neutral-950';

export const PREVIEW_TOOLBAR_CLASS =
  'flex shrink-0 flex-wrap items-center gap-2 border-b border-neutral-200 bg-white/95 px-3 py-2 backdrop-blur dark:border-neutral-800 dark:bg-neutral-900/95';

export const PREVIEW_GHOST_BUTTON_CLASS =
  'inline-flex h-8 items-center justify-center gap-1.5 rounded-md px-2 text-xs font-medium text-neutral-600 transition-colors hover:bg-neutral-100 hover:text-neutral-900 disabled:cursor-not-allowed disabled:opacity-50 dark:text-neutral-300 dark:hover:bg-neutral-800 dark:hover:text-white';

export const PREVIEW_PRIMARY_BUTTON_CLASS =
  'inline-flex h-8 items-center justify-center gap-1.5 rounded-md bg-blue-600 px-3 text-xs font-medium text-white shadow-sm transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-blue-500 dark:hover:bg-blue-600';

export const PREVIEW_SECONDARY_BUTTON_CLASS =
  'inline-flex h-8 items-center justify-center gap-1.5 rounded-md border border-neutral-300 bg-white px-2.5 text-xs font-medium text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-600 dark:bg-neutral-800 dark:text-neutral-200 dark:hover:bg-neutral-700';

export const PREVIEW_ICON_BUTTON_CLASS =
  'inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-neutral-500 transition-colors hover:bg-neutral-100 hover:text-neutral-900 disabled:cursor-not-allowed disabled:opacity-40 dark:text-neutral-400 dark:hover:bg-neutral-800 dark:hover:text-white';

export const PREVIEW_INPUT_CLASS =
  'h-7 min-w-0 rounded-md border border-neutral-300 bg-white px-2 text-xs text-neutral-900 outline-none transition-colors placeholder:text-neutral-400 focus:border-blue-500 focus:ring-1 focus:ring-blue-500/20 dark:border-neutral-600 dark:bg-neutral-900 dark:text-neutral-100 dark:placeholder:text-neutral-500';

export const PREVIEW_BADGE_CLASS =
  'inline-flex shrink-0 items-center gap-1 rounded-full bg-neutral-100 px-2 py-0.5 text-[11px] font-medium text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300';

export const PREVIEW_PANEL_CLASS =
  'overflow-hidden rounded-lg border border-neutral-200 bg-white shadow-sm dark:border-neutral-800 dark:bg-neutral-900';

export const PREVIEW_SCROLL_CLASS = 'min-h-0 flex-1 overflow-auto';

/** 图片/视频/音频的深色画布：媒体内容在两种主题下都放在同一个中性底上。 */
export const PREVIEW_CANVAS_CLASS =
  'relative flex min-h-0 flex-1 items-center justify-center overflow-hidden bg-neutral-900/95';

/** 棋盘格底：透明图片（PNG/SVG）需要能看出透明区域。 */
export const PREVIEW_CHECKERBOARD_STYLE: React.CSSProperties = {
  backgroundImage:
    'linear-gradient(45deg, rgba(255,255,255,0.06) 25%, transparent 25%), linear-gradient(-45deg, rgba(255,255,255,0.06) 25%, transparent 25%), linear-gradient(45deg, transparent 75%, rgba(255,255,255,0.06) 75%), linear-gradient(-45deg, transparent 75%, rgba(255,255,255,0.06) 75%)',
  backgroundSize: '20px 20px',
  backgroundPosition: '0 0, 0 10px, 10px -10px, -10px 0px',
};

export const PREVIEW_MONO_CLASS =
  'font-mono text-[12px] leading-[20px] text-neutral-800 dark:text-neutral-200';
