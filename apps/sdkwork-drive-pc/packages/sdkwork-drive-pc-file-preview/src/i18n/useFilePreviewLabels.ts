import { useMemo } from 'react';
import {
  mergeFilePreviewLabels,
  type FilePreviewLabels,
} from './filePreviewLabels';

/** 宿主可以只覆盖自己翻译过的字段。 */
export type FilePreviewLabelsInput = Partial<FilePreviewLabels>;

/**
 * 合并宿主文案与英文兜底。
 *
 * 单独的 hook 而不是在组件里直接合并：预览组件树里每一层都需要同一份 labels，
 * 在这里合并一次并保持引用稳定，能让下层组件的 memo 生效。
 */
export function useFilePreviewLabels(input?: FilePreviewLabelsInput): FilePreviewLabels {
  return useMemo(() => mergeFilePreviewLabels(input), [input]);
}
