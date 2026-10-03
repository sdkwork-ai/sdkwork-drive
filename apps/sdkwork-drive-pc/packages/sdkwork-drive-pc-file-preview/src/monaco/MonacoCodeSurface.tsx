import React, { useEffect, useRef, useState } from 'react';
import Editor, { type OnMount } from '@monaco-editor/react';
import type { HostColorMode } from '../hooks/useHostColorMode';
import { ensureMonacoConfigured, monacoThemeFor, resolveMonacoLanguage } from './setupMonaco';

/*
 * 装配必须发生在模块求值时，而不是组件挂载后的 effect 里。
 *
 * `@monaco-editor/react` 的 `<Editor>` 一挂载就会调 `loader.init()`，而 loader 在
 * `monaco` 为空时会**立刻**改成从 CDN 拉取（默认 jsdelivr），并把 `isInitialized`
 * 置为 true；之后任何 `loader.config({ monaco })` 都无法撤回。React 又是先跑子组件
 * effect、再跑父组件 effect，所以放在 effect 里必然晚一步：首次打开代码文件就会联网。
 * 这个模块由 `React.lazy` 加载，模块级装配同样只在真正需要编辑器时才执行。
 */
ensureMonacoConfigured();

export interface MonacoCodeSurfaceProps {
  colorMode: HostColorMode;
  language: string;
  readOnly: boolean;
  value: string;
  wrap: boolean;
  /** 只读预览时省略。 */
  onChange?: (value: string) => void;
  /** 编辑器就绪（Monaco 挂载完成）。 */
  onReady?: () => void;
  /** 实际生效的语言 id（可能与请求的不同：未注册的 id 会回落纯文本）。 */
  onLanguageResolved?: (languageId: string) => void;
  /** Ctrl/Cmd+S 请求保存；省略时不接管该快捷键。 */
  onSaveShortcut?: () => void;
}

/**
 * Monaco 宿主：整个包唯一直接依赖编辑器内核的模块。
 *
 * 它被 `React.lazy` 换进来，所以「导入预览包」不会把 Monaco 拉进主包——只有真正
 * 渲染代码预览/编辑器时才加载（编辑器内核 + 语言 worker 都是数 MB 级的资源）。
 * 这条边界也让 jsdom 测试可以导入本包而不触发 Monaco 对 DOM API 的假设。
 */
export function MonacoCodeSurface({
  colorMode,
  language,
  onChange,
  onLanguageResolved,
  onReady,
  onSaveShortcut,
  readOnly,
  value,
  wrap,
}: MonacoCodeSurfaceProps) {
  const [activeLanguage, setActiveLanguage] = useState(language);
  // 回调放 ref：语言解析只应在 `language` 变化时跑一次，而不是每次父组件重渲染
  // 都重新通知一遍（父组件通常传内联函数）。
  const languageResolvedRef = useRef(onLanguageResolved);
  languageResolvedRef.current = onLanguageResolved;

  useEffect(() => {
    const resolved = resolveMonacoLanguage(language);
    setActiveLanguage(resolved);
    languageResolvedRef.current?.(resolved);
  }, [language]);

  const handleMount: OnMount = (editor, monacoApi) => {
    if (onSaveShortcut) {
      // 编辑器里 Ctrl/Cmd+S 是肌肉记忆：接住它，别让浏览器弹「保存网页」。
      editor.addCommand(monacoApi.KeyMod.CtrlCmd | monacoApi.KeyCode.KeyS, () => onSaveShortcut());
    }
    onReady?.();
  };

  return (
    <Editor
      height="100%"
      language={activeLanguage}
      theme={monacoThemeFor(colorMode)}
      value={value}
      onChange={(next) => onChange?.(next ?? '')}
      onMount={handleMount}
      options={{
        readOnly,
        domReadOnly: readOnly,
        minimap: { enabled: false },
        fontFamily: 'JetBrains Mono, Menlo, Monaco, "Courier New", monospace',
        fontSize: 12.5,
        lineHeight: 20,
        wordWrap: wrap ? 'on' : 'off',
        scrollBeyondLastLine: false,
        automaticLayout: true,
        tabSize: 2,
        renderWhitespace: 'selection',
        renderLineHighlight: readOnly ? 'none' : 'line',
        padding: { top: 10, bottom: 10 },
        scrollbar: { vertical: 'visible', horizontal: 'auto', alwaysConsumeMouseWheel: false },
      }}
    />
  );
}
