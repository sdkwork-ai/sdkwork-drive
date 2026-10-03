import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Check, LoaderCircle, RotateCcw, Save, WrapText } from 'lucide-react';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { useHostColorMode, type HostColorMode } from '../../hooks/useHostColorMode';
import { monacoLanguageForFile } from '../../kinds/filePreviewKind';
import {
  PREVIEW_BADGE_CLASS,
  PREVIEW_ICON_BUTTON_CLASS,
  PREVIEW_SECONDARY_BUTTON_CLASS,
} from '../previewStyles';

/** 与 `CodeFilePreview` 同一策略：编辑器内核只在真正编辑时加载。 */
const MonacoCodeSurface = React.lazy(() =>
  import('../../monaco/MonacoCodeSurface').then((module) => ({
    default: module.MonacoCodeSurface,
  })),
);

export type FileEditorSaveState = 'idle' | 'saving' | 'saved' | 'error';

export interface CodeFileEditorProps {
  colorMode?: HostColorMode;
  labels: FilePreviewLabels;
  name: string;
  /** 当前文本。受控：草稿属于上层，切换预览/编辑或保存后重取内容都不会丢。 */
  value: string;
  onChange: (next: string) => void;
  saveState?: FileEditorSaveState;
  saveError?: string;
  onSave: (next: string) => void;
}

/**
 * 代码/文本编辑器：Monaco（VS Code 的编辑器内核）+ 脏标记 + 显式保存。
 *
 * 三条产品级约束在这里落地：
 * 1. **不自动保存**：对象存储的写入是整对象替换，自动保存会让每次停顿都产生一次
 *    远端写入；保存由用户触发（按钮或 Ctrl/Cmd+S）。
 * 2. **草稿归上层**：本组件受控，文本与「是否有未保存修改」都由 `FilePreviewSurface`
 *    持有——否则切一次预览就会卸载编辑器、连带丢掉草稿并解除关闭守卫。
 * 3. **保存失败不丢草稿**：失败只更新状态与错误文案，编辑器里的内容保持原样，用户
 *    可以重试或回退到已保存版本。
 */
export function CodeFileEditor({
  colorMode: colorModeOverride,
  labels,
  name,
  onChange,
  onSave,
  saveError,
  saveState = 'idle',
  value,
}: CodeFileEditorProps) {
  const colorMode = useHostColorMode(colorModeOverride);
  const [wrap, setWrap] = useState(true);
  const [ready, setReady] = useState(false);
  const language = useMemo(() => monacoLanguageForFile(name), [name]);
  const [activeLanguage, setActiveLanguage] = useState(language);
  /** 已保存版本：用于「回退」与脏判定（`value` 是可能含草稿的当前文本）。 */
  const [baseline, setBaseline] = useState(value);
  const dirty = value !== baseline;

  // 保存成功（宿主把 value 换成刚写回的内容）后，当前文本成为新的基线。
  useEffect(() => {
    if (saveState === 'saved') {
      setBaseline(value);
    }
  }, [saveState, value]);

  const dirtyRef = useRef(dirty);
  dirtyRef.current = dirty;
  const valueRef = useRef(value);
  valueRef.current = value;
  const saveRef = useRef(onSave);
  saveRef.current = onSave;
  const savingRef = useRef(saveState);
  savingRef.current = saveState;

  const requestSaveShortcut = useCallback(() => {
    if (dirtyRef.current && savingRef.current !== 'saving') {
      saveRef.current(valueRef.current);
    }
  }, []);

  const requestSave = useCallback(() => {
    if (!dirty || saveState === 'saving') {
      return;
    }
    onSave(value);
  }, [dirty, onSave, saveState, value]);

  const revert = useCallback(() => {
    onChange(baseline);
  }, [baseline, onChange]);

  return (
    <div className="flex h-full w-full flex-col bg-white dark:bg-neutral-950">
      <div className="flex shrink-0 flex-wrap items-center gap-1.5 border-b border-neutral-200 bg-white/90 px-3 py-1.5 dark:border-neutral-800 dark:bg-neutral-900/90">
        <span className="shrink-0 rounded bg-neutral-100 px-1.5 py-0.5 font-mono text-[11px] text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300">
          {activeLanguage}
        </span>
        {dirty ? (
          <span className={`${PREVIEW_BADGE_CLASS} bg-amber-100 text-amber-700 dark:bg-amber-950/40 dark:text-amber-300`}>
            {labels.unsavedChanges}
          </span>
        ) : saveState === 'saved' ? (
          <span className={`${PREVIEW_BADGE_CLASS} bg-emerald-100 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300`}>
            <Check aria-hidden="true" size={11} />
            {labels.saved}
          </span>
        ) : null}
        {saveState === 'error' && saveError ? (
          <span className="min-w-0 truncate text-[11px] text-red-600 dark:text-red-400" title={saveError}>
            {labels.saveFailed}: {saveError}
          </span>
        ) : null}

        <span className="flex-1" />

        <button
          type="button"
          className={PREVIEW_ICON_BUTTON_CLASS}
          title={wrap ? labels.unwrapLines : labels.wrapLines}
          aria-label={wrap ? labels.unwrapLines : labels.wrapLines}
          aria-pressed={wrap}
          onClick={() => setWrap((current) => !current)}
        >
          <WrapText aria-hidden="true" size={14} />
        </button>
        <button
          type="button"
          className={PREVIEW_SECONDARY_BUTTON_CLASS}
          disabled={!dirty || saveState === 'saving'}
          title={labels.discardChanges}
          onClick={revert}
        >
          <RotateCcw aria-hidden="true" size={13} />
          {labels.discardChanges}
        </button>
        <button
          type="button"
          className="inline-flex h-8 items-center justify-center gap-1.5 rounded-md bg-blue-600 px-3 text-xs font-medium text-white shadow-sm transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-blue-500 dark:hover:bg-blue-600"
          disabled={!dirty || saveState === 'saving'}
          onClick={requestSave}
        >
          {saveState === 'saving' ? (
            <LoaderCircle aria-hidden="true" className="animate-spin" size={13} />
          ) : (
            <Save aria-hidden="true" size={13} />
          )}
          {saveState === 'saving' ? labels.saving : labels.save}
        </button>
      </div>

      <div className="relative min-h-0 flex-1">
        {!ready ? (
          <div className="absolute inset-0 z-10 flex items-center justify-center gap-2 bg-white/80 text-xs text-neutral-500 dark:bg-neutral-950/80 dark:text-neutral-400">
            <LoaderCircle aria-hidden="true" className="animate-spin" size={16} />
            {labels.loading}
          </div>
        ) : null}
        <React.Suspense fallback={null}>
          <MonacoCodeSurface
            colorMode={colorMode}
            language={language}
            // 保存进行中把编辑器设为只读：整对象替换期间继续输入会被随后的重取覆盖，
            // 读起来像"保存成功却吞掉了刚敲的字"。
            readOnly={saveState === 'saving'}
            value={value}
            wrap={wrap}
            onChange={onChange}
            onLanguageResolved={setActiveLanguage}
            onReady={() => setReady(true)}
            onSaveShortcut={requestSaveShortcut}
          />
        </React.Suspense>
      </div>
    </div>
  );
}
