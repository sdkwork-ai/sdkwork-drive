import {
  File as FileIcon,
  FileArchive,
  FileAudio,
  FileCode,
  FileImage,
  FileQuestion,
  FileSpreadsheet,
  FileText,
  FileVideo,
  Folder,
  Presentation,
  type LucideIcon,
} from 'lucide-react';
import type { FilePreviewKind } from './filePreviewKind';

/**
 * 每种文件种类的视觉身份：图标 + 底色 + 强调色。
 *
 * 列表行、预览头部、空态与徽标都读这一份，所以「红色永远是 PDF、绿色永远是表格」
 * 在整个产品里只定义一次——主流网盘正是靠这套稳定的颜色语言让用户扫一眼就能分类。
 * 底色都用 `bg-*-50 / dark:bg-*-950/40` 这一对，浅色模式是柔和的浅底，深色模式是
 * 同色系的深底，不会在深色主题里出现刺眼的纯白块。
 *
 * `iconClass` 的明度按 WCAG 1.4.11（图形对象 3:1）逐对校准过：浅色底上用 600/700，
 * 深色底上用 300/400。之前文件夹用的是 `amber-500`，在 `amber-50` 上只有 2.07:1，
 * 是列表里最常见、却最难看清的图标；`npm test` 里的对比度用例会守住这条线。
 */
export interface FileKindVisual {
  Icon: LucideIcon;
  /** 图标前景色（含深色模式变体，逐主题校准对比度）。 */
  iconClass: string;
  /** 圆角图块底色。 */
  tileClass: string;
  /** 进度条 / 徽标等强调元素用色。 */
  accentClass: string;
  /** 默认英文名；宿主可用自己的字典覆盖。 */
  defaultLabel: string;
  /** 文件夹用实心图标，贴近主流网盘的观感。 */
  filled?: boolean;
}

export const FILE_KIND_VISUALS: Readonly<Record<FilePreviewKind, FileKindVisual>> = {
  folder: {
    Icon: Folder,
    iconClass: 'text-amber-700 dark:text-amber-300',
    tileClass: 'bg-amber-50 dark:bg-amber-950/40',
    accentClass: 'text-amber-600 dark:text-amber-400',
    defaultLabel: 'Folder',
    filled: true,
  },
  image: {
    // 图片用「科技蓝」：这是列表里出现最多的一类（照片桶几乎满屏），也是产品主色所在，
    // 紫色在深色主题下会显得偏"装饰"，蓝色与整体控制台语言一致。
    Icon: FileImage,
    iconClass: 'text-blue-700 dark:text-blue-300',
    tileClass: 'bg-blue-50 dark:bg-blue-950/40',
    accentClass: 'text-blue-600 dark:text-blue-400',
    defaultLabel: 'Image',
  },
  video: {
    Icon: FileVideo,
    iconClass: 'text-rose-600 dark:text-rose-300',
    tileClass: 'bg-rose-50 dark:bg-rose-950/40',
    accentClass: 'text-rose-600 dark:text-rose-400',
    defaultLabel: 'Video',
  },
  audio: {
    Icon: FileAudio,
    iconClass: 'text-orange-700 dark:text-orange-300',
    tileClass: 'bg-orange-50 dark:bg-orange-950/40',
    accentClass: 'text-orange-600 dark:text-orange-400',
    defaultLabel: 'Audio',
  },
  pdf: {
    Icon: FileText,
    iconClass: 'text-red-700 dark:text-red-300',
    tileClass: 'bg-red-50 dark:bg-red-950/40',
    accentClass: 'text-red-600 dark:text-red-400',
    defaultLabel: 'PDF',
  },
  word: {
    // 文档让出正蓝给图片，改用浅一号的天蓝：仍属"文档蓝"的语义，但与图片一眼可分。
    Icon: FileText,
    iconClass: 'text-sky-700 dark:text-sky-300',
    tileClass: 'bg-sky-50 dark:bg-sky-950/40',
    accentClass: 'text-sky-600 dark:text-sky-400',
    defaultLabel: 'Document',
  },
  spreadsheet: {
    Icon: FileSpreadsheet,
    iconClass: 'text-emerald-700 dark:text-emerald-300',
    tileClass: 'bg-emerald-50 dark:bg-emerald-950/40',
    accentClass: 'text-emerald-600 dark:text-emerald-400',
    defaultLabel: 'Spreadsheet',
  },
  presentation: {
    // 幻灯片改用品红：琥珀被文件夹占用（同色同底的两类图标在列表里等于没有区分），
    // 而橙/红/玫红都已各有归属；品红既醒目又与它们完全不同色相。
    Icon: Presentation,
    iconClass: 'text-fuchsia-700 dark:text-fuchsia-300',
    tileClass: 'bg-fuchsia-50 dark:bg-fuchsia-950/40',
    accentClass: 'text-fuchsia-600 dark:text-fuchsia-400',
    defaultLabel: 'Presentation',
  },
  archive: {
    Icon: FileArchive,
    iconClass: 'text-yellow-700 dark:text-yellow-300',
    tileClass: 'bg-yellow-50 dark:bg-yellow-950/40',
    accentClass: 'text-yellow-600 dark:text-yellow-400',
    defaultLabel: 'Archive',
  },
  text: {
    Icon: FileText,
    iconClass: 'text-slate-600 dark:text-slate-300',
    tileClass: 'bg-slate-100 dark:bg-slate-800/60',
    accentClass: 'text-slate-600 dark:text-slate-300',
    defaultLabel: 'Text',
  },
  markdown: {
    // 接住图片让出的紫色：13 类仍然两两不同色。
    Icon: FileText,
    iconClass: 'text-violet-700 dark:text-violet-300',
    tileClass: 'bg-violet-50 dark:bg-violet-950/40',
    accentClass: 'text-violet-600 dark:text-violet-400',
    defaultLabel: 'Markdown',
  },
  code: {
    Icon: FileCode,
    iconClass: 'text-cyan-700 dark:text-cyan-300',
    tileClass: 'bg-cyan-50 dark:bg-cyan-950/40',
    accentClass: 'text-cyan-600 dark:text-cyan-400',
    defaultLabel: 'Code',
  },
  unsupported: {
    Icon: FileQuestion,
    iconClass: 'text-neutral-600 dark:text-neutral-300',
    tileClass: 'bg-neutral-100 dark:bg-neutral-800/60',
    accentClass: 'text-neutral-500 dark:text-neutral-400',
    defaultLabel: 'File',
  },
};

/** 全局未知种类（例如宿主传入了一个本包还不认识的 kind）时的兜底视觉。 */
export const FALLBACK_FILE_KIND_VISUAL: FileKindVisual = {
  Icon: FileIcon,
  iconClass: 'text-neutral-600 dark:text-neutral-300',
  tileClass: 'bg-neutral-100 dark:bg-neutral-800/60',
  accentClass: 'text-neutral-500 dark:text-neutral-400',
  defaultLabel: 'File',
};

export function fileKindVisual(kind: FilePreviewKind): FileKindVisual {
  return FILE_KIND_VISUALS[kind] ?? FALLBACK_FILE_KIND_VISUAL;
}
