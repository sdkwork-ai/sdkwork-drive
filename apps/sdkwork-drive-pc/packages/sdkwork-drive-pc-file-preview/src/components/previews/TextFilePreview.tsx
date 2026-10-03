import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Check, ChevronDown, ChevronUp, Copy, Search, WrapText } from 'lucide-react';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { formatFilePreviewLabel } from '../../i18n/filePreviewLabels';
import { countTextLines, countTextWords } from '../../utils/bytes';
import {
  PREVIEW_BADGE_CLASS,
  PREVIEW_ICON_BUTTON_CLASS,
  PREVIEW_INPUT_CLASS,
  PREVIEW_MONO_CLASS,
} from '../previewStyles';

export interface TextFilePreviewProps {
  labels: FilePreviewLabels;
  name: string;
  text: string;
}

/** 超过这个行数只渲染前 N 行：浏览器里 10 万行的 `<pre>` 会让滚动卡死。 */
const MAX_RENDERED_LINES = 5000;

/**
 * 纯文本预览：行号、自动换行开关、文内搜索与跳转、复制与统计。
 *
 * 搜索高亮按「不区分大小写的字面量」实现（用户搜的是文本，不是正则），命中位置
 * 预先算好，跳转时只滚动到对应行，避免在大文件上做整篇 DOM 重排。
 */
export function TextFilePreview({ labels, name, text }: TextFilePreviewProps) {
  const [wrap, setWrap] = useState(false);
  const [query, setQuery] = useState('');
  const [activeMatch, setActiveMatch] = useState(0);
  const [copied, setCopied] = useState(false);
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const lineRefs = useRef<Map<number, HTMLDivElement | null>>(new Map());

  const lines = useMemo(() => {
    const all = text.split('\n');
    return all.length > MAX_RENDERED_LINES ? all.slice(0, MAX_RENDERED_LINES) : all;
  }, [text]);
  const truncated = countTextLines(text) > MAX_RENDERED_LINES;

  const matches = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (needle === '') {
      return [];
    }
    const found: number[] = [];
    lines.forEach((line, index) => {
      if (line.toLowerCase().includes(needle)) {
        found.push(index);
      }
    });
    return found;
  }, [lines, query]);

  useEffect(() => setActiveMatch(0), [query]);

  const jump = useCallback(
    (position: number) => {
      if (matches.length === 0) {
        return;
      }
      const index = ((position % matches.length) + matches.length) % matches.length;
      setActiveMatch(index);
      const target = lineRefs.current.get(matches[index]);
      // jsdom 没有 scrollIntoView：预览在非浏览器环境下也必须能跑（测试即宿主）。
      if (target && typeof target.scrollIntoView === 'function') {
        target.scrollIntoView({ block: 'center' });
      }
    },
    [matches],
  );

  const highlight = useCallback(
    (line: string): React.ReactNode => {
      const needle = query.trim();
      if (needle === '') {
        return line;
      }
      const lowered = line.toLowerCase();
      const target = needle.toLowerCase();
      const parts: React.ReactNode[] = [];
      let cursor = 0;
      let found = lowered.indexOf(target);
      let key = 0;
      while (found >= 0) {
        if (found > cursor) {
          parts.push(line.slice(cursor, found));
        }
        parts.push(
          <mark
            key={`mark-${key}`}
            className="rounded bg-amber-200/80 text-inherit dark:bg-amber-500/30"
          >
            {line.slice(found, found + needle.length)}
          </mark>,
        );
        key += 1;
        cursor = found + needle.length;
        found = lowered.indexOf(target, cursor);
      }
      if (cursor < line.length) {
        parts.push(line.slice(cursor));
      }
      return parts;
    },
    [query],
  );

  const stats = useMemo(
    () => ({ lines: countTextLines(text), words: countTextWords(text), characters: text.length }),
    [text],
  );

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
    <div className="flex h-full w-full flex-col">
      <div className="flex shrink-0 flex-wrap items-center gap-1.5 border-b border-neutral-200 bg-white/90 px-3 py-1.5 dark:border-neutral-800 dark:bg-neutral-900/90">
        <div className="relative min-w-0 flex-1 sm:max-w-xs">
          <Search
            aria-hidden="true"
            className="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 text-neutral-400"
            size={13}
          />
          <input
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={labels.searchPlaceholder}
            aria-label={labels.search}
            className={`${PREVIEW_INPUT_CLASS} w-full pl-7`}
          />
        </div>
        {query.trim() !== '' ? (
          <>
            <span className="shrink-0 text-[11px] tabular-nums text-neutral-500 dark:text-neutral-400">
              {matches.length === 0
                ? labels.noMatches
                : formatFilePreviewLabel(labels.matchCount, {
                    current: activeMatch + 1,
                    total: matches.length,
                  })}
            </span>
            <button
              type="button"
              className={PREVIEW_ICON_BUTTON_CLASS}
              title={labels.previous}
              aria-label={labels.previous}
              disabled={matches.length === 0}
              onClick={() => jump(activeMatch - 1)}
            >
              <ChevronUp aria-hidden="true" size={14} />
            </button>
            <button
              type="button"
              className={PREVIEW_ICON_BUTTON_CLASS}
              title={labels.next}
              aria-label={labels.next}
              disabled={matches.length === 0}
              onClick={() => jump(activeMatch + 1)}
            >
              <ChevronDown aria-hidden="true" size={14} />
            </button>
          </>
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
          className={PREVIEW_ICON_BUTTON_CLASS}
          title={copied ? labels.copied : labels.copy}
          aria-label={copied ? labels.copied : labels.copy}
          onClick={copyAll}
        >
          {copied ? <Check aria-hidden="true" size={14} /> : <Copy aria-hidden="true" size={14} />}
        </button>
      </div>

      <div ref={scrollRef} className="min-h-0 flex-1 overflow-auto bg-white dark:bg-neutral-950">
        <div className="flex min-h-full">
          <div
            aria-hidden="true"
            className="sticky left-0 z-10 shrink-0 select-none border-r border-neutral-200 bg-neutral-50 px-2 py-2 text-right dark:border-neutral-800 dark:bg-neutral-900"
          >
            {lines.map((_, index) => (
              <div
                key={`gutter-${index}`}
                className="font-mono text-[12px] leading-[20px] text-neutral-400 dark:text-neutral-600"
              >
                {index + 1}
              </div>
            ))}
          </div>
          <div className={`min-w-0 flex-1 px-3 py-2 ${PREVIEW_MONO_CLASS}`}>
            {lines.map((line, index) => (
              <div
                key={`line-${index}`}
                ref={(element) => {
                  lineRefs.current.set(index, element);
                }}
                className={`min-h-[20px] ${
                  matches[activeMatch] === index ? 'bg-amber-100/70 dark:bg-amber-500/10' : ''
                } ${wrap ? 'whitespace-pre-wrap break-words' : 'whitespace-pre'}`}
              >
                {highlight(line) || '\u00a0'}
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="flex shrink-0 flex-wrap items-center gap-3 border-t border-neutral-200 bg-white/90 px-3 py-1.5 text-[11px] text-neutral-500 dark:border-neutral-800 dark:bg-neutral-900/90 dark:text-neutral-400">
        <span className={PREVIEW_BADGE_CLASS}>
          {labels.linesLabel}: {stats.lines}
        </span>
        <span className={PREVIEW_BADGE_CLASS}>
          {labels.wordsLabel}: {stats.words}
        </span>
        <span className={PREVIEW_BADGE_CLASS}>
          {labels.charactersLabel}: {stats.characters}
        </span>
        {truncated ? (
          <span className="text-amber-600 dark:text-amber-400">
            {formatFilePreviewLabel(labels.rowsTruncated, { count: MAX_RENDERED_LINES })}
          </span>
        ) : null}
        <span className="flex-1" />
        <span className="truncate font-mono">{name}</span>
      </div>
    </div>
  );
}
