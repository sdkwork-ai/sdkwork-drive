import React, { useEffect, useMemo, useRef, useState } from 'react';
import { Info, Save, Search, Sparkles } from 'lucide-react';
import { useTranslation } from 'sdkwork-drive-pc-commons';
import type { DriveFile } from 'sdkwork-drive-pc-types';
import { isDriveAbortError, type DriveFileService } from 'sdkwork-drive-pc-core';
import Editor from '@monaco-editor/react';

interface TextEditorModuleProps {
  file: DriveFile;
  fileService: DriveFileService;
  triggerFeedback: (text: string, type?: 'success' | 'info' | 'error') => void;
  onSaved?: () => void;
  isReadOnly?: boolean;
}

export function TextEditorModule({ file, fileService, triggerFeedback, onSaved, isReadOnly = false }: TextEditorModuleProps) {
  const { t } = useTranslation();

  const [wordTextValue, setWordTextValue] = useState('');
  const [contentType, setContentType] = useState<string | undefined>(file.mimeType);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [isLoadingContent, setIsLoadingContent] = useState(true);
  const [isSavingContent, setIsSavingContent] = useState(false);
  const [editorFont, setEditorFont] = useState<'sans' | 'mono'>('mono');
  const [editorTheme, setEditorTheme] = useState<'light' | 'dark'>('dark');
  const [editorViewMode, setEditorViewMode] = useState<'edit' | 'markdown'>('edit');
  const [findKeyword, setFindKeyword] = useState('');
  const [replaceKeyword, setReplaceKeyword] = useState('');
  const [showFindReplace, setShowFindReplace] = useState(false);
  const saveAbortControllerRef = useRef<AbortController | null>(null);

  useEffect(() => {
    let active = true;
    const contentAbortController = new AbortController();
    setIsLoadingContent(true);
    setLoadError(null);
    setWordTextValue('');

    fileService.readFileText(file, {
      signal: contentAbortController.signal,
    })
      .then((result) => {
        if (active) {
          setWordTextValue(result.content);
          setContentType(result.contentType || file.mimeType);
        }
      })
      .catch((err: any) => {
        if (isDriveAbortError(err)) {
          return;
        }
        if (active) {
          setLoadError(err?.message || t('previewModules.textLoadFailed'));
        }
      })
      .finally(() => {
        if (active) {
          setIsLoadingContent(false);
        }
      });

    return () => {
      active = false;
      contentAbortController.abort();
    };
  }, [file.id, file.updatedAt, fileService]);

  useEffect(() => {
    saveAbortControllerRef.current?.abort();
    saveAbortControllerRef.current = null;
    setIsSavingContent(false);

    return () => {
      saveAbortControllerRef.current?.abort();
      saveAbortControllerRef.current = null;
    };
  }, [file.id]);

  const handleSaveTextDocument = () => {
    if (isReadOnly || isSavingContent) return;
    saveAbortControllerRef.current?.abort();
    const saveAbortController = new AbortController();
    saveAbortControllerRef.current = saveAbortController;
    setIsSavingContent(true);
    fileService.saveFileText(file, wordTextValue, contentType || file.mimeType || 'text/plain', {
      signal: saveAbortController.signal,
    })
      .then(() => {
        if (saveAbortControllerRef.current !== saveAbortController) {
          return;
        }
        triggerFeedback(t('previewModules.textSavedToDrive'), 'success');
        onSaved?.();
      })
      .catch((err: any) => {
        if (isDriveAbortError(err)) {
          return;
        }
        if (saveAbortControllerRef.current !== saveAbortController) {
          return;
        }
        triggerFeedback(err?.message || t('previewModules.textSaveFailed'), 'error');
      })
      .finally(() => {
        if (saveAbortControllerRef.current === saveAbortController) {
          saveAbortControllerRef.current = null;
          setIsSavingContent(false);
        }
      });
  };

  const textStats = useMemo(() => {
    const chars = wordTextValue.length;
    const words = wordTextValue.trim() === '' ? 0 : wordTextValue.trim().split(/\s+/).length;
    const lines = wordTextValue === '' ? 0 : wordTextValue.split('\n').length;
    const readMin = Math.max(1, Math.ceil(words / 200));
    return { chars, words, lines, readMin };
  }, [wordTextValue]);

  const handleFindReplaceAll = () => {
    if (isReadOnly || !findKeyword) return;
    const updated = wordTextValue.replaceAll(findKeyword, replaceKeyword);
    setWordTextValue(updated);
    triggerFeedback(t('fileDetail.replacedOccurrences', { find: findKeyword, replace: replaceKeyword }));
  };

  const editorLanguage = useMemo(() => {
    const ext = file.name.split('.').pop()?.toLowerCase();
    if (!ext) return 'plaintext';
    switch (ext) {
      case 'md':
      case 'markdown':
        return 'markdown';
      case 'json':
        return 'json';
      case 'js':
      case 'jsx':
        return 'javascript';
      case 'ts':
      case 'tsx':
        return 'typescript';
      case 'html':
      case 'htm':
        return 'html';
      case 'css':
        return 'css';
      case 'yml':
      case 'yaml':
        return 'yaml';
      case 'xml':
        return 'xml';
      default:
        return 'plaintext';
    }
  }, [file.name]);

  return (
    <div className="w-full max-w-5xl bg-[#131315] border border-neutral-800/80 rounded-2xl overflow-hidden shadow-2xl flex flex-col h-[78vh] max-h-[720px] animate-in zoom-in-95 duration-250 select-text">
      <div className="h-12 bg-[#1c1c1c] border-b border-neutral-800/60 px-4 flex items-center justify-between shrink-0 text-xs font-semibold text-neutral-400 select-none">
        <div className="flex items-center gap-3">
          <button
            onClick={() => setEditorViewMode('edit')}
            className={`px-3 py-1 rounded-md transition-colors ${editorViewMode === 'edit' ? 'bg-[#282828] text-white font-bold' : 'hover:text-white'}`}
          >
            {t('fileDetail.codeAndTextEditor')}
          </button>
          <button
            onClick={() => setEditorViewMode('markdown')}
            className={`px-3 py-1 rounded-md transition-colors ${editorViewMode === 'markdown' ? 'bg-[#282828] text-white font-bold' : 'hover:text-white'}`}
          >
            {t('fileDetail.markdownViewer')}
          </button>
        </div>

        <div className="flex items-center gap-2.5">
          <div className="flex items-center gap-1.5 text-neutral-500 bg-neutral-900 px-2 py-1 rounded border border-neutral-800 text-[10px] uppercase font-mono">
            <Sparkles size={11} className="text-amber-400" />
            <span>Drive Content</span>
          </div>

          <button
            onClick={() => setShowFindReplace(!showFindReplace)}
            className={`p-1.5 hover:bg-[#282828] rounded ${showFindReplace ? 'text-blue-400' : ''}`}
            title={t('fileDetail.findAndReplace')}
          >
            <Search size={14} />
          </button>
          <select
            value={editorFont}
            onChange={(e: any) => setEditorFont(e.target.value)}
            className="bg-[#252525] border border-neutral-800 rounded px-1.5 py-0.5 text-[11px] outline-none text-neutral-300"
          >
            <option value="sans">{t('fileDetail.fontSans')}</option>
            <option value="mono">{t('fileDetail.fontMono')}</option>
          </select>
          <select
            value={editorTheme}
            onChange={(e: any) => setEditorTheme(e.target.value)}
            className="bg-[#252525] border border-neutral-800 rounded px-1.5 py-0.5 text-[11px] outline-none text-neutral-300"
          >
            <option value="light">{t('fileDetail.themeLight')}</option>
            <option value="dark">{t('fileDetail.themeDark')}</option>
          </select>
        </div>
      </div>

      {showFindReplace && (
        <div className="px-4 py-2 bg-[#1c1c1c] border-b border-neutral-800/60 flex items-center gap-2 text-xs select-none">
          <input
            type="text"
            placeholder={t('fileDetail.findWord')}
            value={findKeyword}
            onChange={(e) => setFindKeyword(e.target.value)}
            className="bg-[#2a2a2a] border border-neutral-800 rounded px-2.5 py-1 text-xs text-white max-w-[120px] outline-none"
          />
          <input
            type="text"
            placeholder={t('fileDetail.replaceWith')}
            value={replaceKeyword}
            onChange={(e) => setReplaceKeyword(e.target.value)}
            className="bg-[#2a2a2a] border border-neutral-800 rounded px-2.5 py-1 text-xs text-white max-w-[120px] outline-none"
          />
          <button
            onClick={handleFindReplaceAll}
            disabled={isReadOnly}
            className="px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded text-[11px] cursor-pointer"
          >
            {t('fileDetail.replaceAll')}
          </button>
        </div>
      )}

      <div className="flex-1 relative overflow-hidden h-full flex flex-col">
        {isLoadingContent ? (
          <div className="w-full h-full bg-[#181818] flex flex-col items-center justify-center text-xs text-neutral-400 gap-2">
            <div className="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
            <span>Loading Drive content...</span>
          </div>
        ) : loadError ? (
          <div className="w-full h-full bg-[#181818] flex flex-col items-center justify-center text-xs text-neutral-400 gap-3 px-8 text-center">
            <Info size={20} className="text-rose-400" />
            <span>{loadError}</span>
          </div>
        ) : editorViewMode === 'edit' ? (
          <div className="w-full h-full flex-1">
            <Editor
              height="100%"
              language={editorLanguage}
              theme={editorTheme === 'dark' ? 'vs-dark' : 'light'}
              value={wordTextValue}
              onChange={(value) => setWordTextValue(value || '')}
              loading={
                <div className="w-full h-full bg-[#181818] flex flex-col items-center justify-center text-xs text-neutral-400 gap-2">
                  <div className="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
                  <span>Loading editor...</span>
                </div>
              }
              options={{
                fontFamily: editorFont === 'mono' ? 'JetBrains Mono, Menlo, Monaco, Courier New, monospace' : 'Inter, system-ui, sans-serif',
                fontSize: 13,
                lineHeight: 20,
                minimap: { enabled: true },
                wordWrap: 'on',
                scrollBeyondLastLine: false,
                automaticLayout: true,
                readOnly: isReadOnly,
                padding: { top: 12 },
                scrollbar: {
                  vertical: 'visible',
                  horizontal: 'visible',
                },
              }}
            />
          </div>
        ) : (
          <div className={`p-8 leading-relaxed prose prose-invert overflow-y-auto max-w-none flex-1 h-full ${
            editorTheme === 'light' ? 'bg-[#fcfdfa] text-neutral-800' : 'bg-[#141414] text-neutral-200'
          }`}>
            <div className="space-y-4 text-xs whitespace-pre-wrap">{wordTextValue}</div>
          </div>
        )}
      </div>

      <div className="h-10 bg-[#1c1c1c] border-t border-neutral-800/60 px-4.5 flex items-center justify-between text-[11px] font-mono text-neutral-500 select-none shrink-0">
        <div className="flex items-center gap-3">
          <span>{t('fileDetail.lines')}: {textStats.lines}</span>
          <span>{t('fileDetail.words')}: {textStats.words}</span>
          <span>{t('fileDetail.chars')}: {textStats.chars}</span>
          <span className="text-blue-400 font-semibold">{t('fileDetail.readMin', { count: textStats.readMin })}</span>
        </div>
        {!isReadOnly && (
          <button
            onClick={handleSaveTextDocument}
            disabled={isSavingContent}
            className="px-3.5 py-1 bg-neutral-800 hover:bg-neutral-700 text-white rounded font-sans font-bold text-xs flex items-center gap-1.5 transition-all cursor-pointer"
          >
            <Save size={12} /> {t('fileDetail.syncAndSave')}
          </button>
        )}
      </div>
    </div>
  );
}
