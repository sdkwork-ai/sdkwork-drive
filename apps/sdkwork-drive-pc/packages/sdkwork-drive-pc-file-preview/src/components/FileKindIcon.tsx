import React from 'react';
import { fileKindVisual } from '../kinds/fileKindVisuals';
import type { FilePreviewKind } from '../kinds/filePreviewKind';

export type FileKindIconSize = 'sm' | 'md' | 'lg' | 'xl';

const TILE_CLASS: Readonly<Record<FileKindIconSize, string>> = {
  /** 列表行：与行高配套的紧凑图块。 */
  sm: 'h-6 w-6 rounded-md',
  /** 详情/预览头部。 */
  md: 'h-8 w-8 rounded-lg',
  /** 卡片、选择器。 */
  lg: 'h-11 w-11 rounded-xl',
  /** 空态与预览占位。 */
  xl: 'h-16 w-16 rounded-2xl',
};

const ICON_SIZE: Readonly<Record<FileKindIconSize, number>> = {
  sm: 14,
  md: 17,
  lg: 22,
  xl: 32,
};

export interface FileKindIconProps {
  className?: string;
  /** 文件名或对象 key（可含目录前缀）。 */
  kind: FilePreviewKind;
  size?: FileKindIconSize;
}

/**
 * 文件种类的彩色图块。
 *
 * 列表行、预览头部与空态共用同一个组件：图标、底色与圆角随 `size` 成比例变化，
 * 所以同一份文件在列表里和预览窗口里是同一个视觉对象，而不是两套手写的样式。
 */
export function FileKindIcon({ className, kind, size = 'sm' }: FileKindIconProps) {
  const visual = fileKindVisual(kind);
  const { Icon } = visual;
  return (
    <span
      aria-hidden="true"
      className={`inline-flex shrink-0 items-center justify-center ${TILE_CLASS[size]} ${visual.tileClass} ${className ?? ''}`}
      data-file-kind={kind}
    >
      <Icon
        size={ICON_SIZE[size]}
        className={`${visual.iconClass} ${visual.filled ? 'fill-current' : ''}`}
        strokeWidth={visual.filled ? 1.5 : 1.75}
      />
    </span>
  );
}
