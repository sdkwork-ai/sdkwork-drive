import React, { useMemo, useState } from 'react';
import { Check, Copy, LoaderCircle, WrapText } from 'lucide-react';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { useHostColorMode, type HostColorMode } from '../../hooks/useHostColorMode';
import { monacoLanguageForFile } from '../../kinds/filePreviewKind';
import { PREVIEW_ICON_BUTTON_CLASS } from '../previewStyles';

/**
 * Monaco 宿主按需加载：`import('...')` 只在真正渲染代码预览时发生。
 *
 * 这一点很关键——把 `monaco-editor` 静态引入预览包，会让「导入这个包」本身就把
 * 数 MB 的编辑器内核（以及它对 DOM API 的假设）带进任何宿主，包括纯 jsdom 的测试。
 */
const MonacoCodeSurface = React.lazy(() =>
  import('../../monaco/MonacoCodeSurface').then((module) => ({
    default: module.MonacoCodeSurface,
  })),
);

export interface CodeFilePreviewProps {
  colorMode?: HostColorMode;
  labels: FilePreviewLabels;
  name: string;
  text: string;
}

/**
 * 代码预览：与 VS Code 同一套编辑器内核（Monaco），只读打开。
 *
 * 只读而不是纯文本渲染，是因为语法高亮、括号配对、折叠与缩进参考线都来自同一份
 * 语言定义——用户在预览里看到的结构和真正编辑时完全一致。写回入口在
 * `CodeFileEditor` 里，两者共用语言判定、主题规则与同一个 Monaco 宿主。
 */
export function CodeFilePreview({ colorMode: colorModeOverride, labels, name, text }: CodeFilePreviewProps) {
  const colorMode = useHostColorMode(colorModeOverride);
  const [wrap, setWrap] = useState(true);
  const [copied, setCopied] = useState(false);
  const [ready, setReady] = useState(false);
  const language = useMemo(() => monacoLanguageForFile(name), [name]);
  const [activeLanguage, setActiveLanguage] = useState(language);

  const copyAll = () => {
    void navigator.clipboard?.writeText(text).then(
      () => {
        setCopied(true);
        window.setTimeout(() => setCopied(false), 1500);
      },
      () => undefined,
    );
  };

  return (
    <div className="flex h-full w-full flex-col bg-white dark:bg-neutral-950">
      <div className="flex shrink-0 flex-wrap items-center gap-1.5 border-b border-neutral-200 bg-white/90 px-3 py-1.5 dark:border-neutral-800 dark:bg-neutral-900/90">
        <span className="shrink-0 rounded bg-neutral-100 px-1.5 py-0.5 font-mono text-[11px] text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300">
          {activeLanguage}
        </span>
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
          className={PREVIEW_ICON_BUTTON_CLASS}
          title={copied ? labels.copied : labels.copy}
          aria-label={copied ? labels.copied : labels.copy}
          onClick={copyAll}
        >
          {copied ? <Check aria-hidden="true" size={14} /> : <Copy aria-hidden="true" size={14} />}
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
            readOnly
            value={text}
            wrap={wrap}
            onLanguageResolved={setActiveLanguage}
            onReady={() => setReady(true)}
          />
        </React.Suspense>
      </div>
    </div>
  );
}
