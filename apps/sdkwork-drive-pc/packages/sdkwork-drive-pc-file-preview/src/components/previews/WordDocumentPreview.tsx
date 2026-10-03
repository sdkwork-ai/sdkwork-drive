import { Fragment, useEffect, useState } from 'react';
import type { ReactNode } from 'react';
import type {
  WordDocumentPreviewModel,
  WordParagraphPreview,
  WordRunPreview,
} from '../../ooxml/wordDocumentPreview';
import { parseWordDocument } from '../../ooxml/wordDocumentPreview';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { describePreviewError } from '../../utils/describePreviewError';
import {
  PREVIEW_MONO_CLASS,
  PREVIEW_SCROLL_CLASS,
} from '../previewStyles';
import { PreviewEmptyPanel, PreviewErrorPanel, PreviewLoadingPanel } from '../PreviewStatePanels';

export interface WordDocumentPreviewProps {
  bytes: Uint8Array;
  labels: FilePreviewLabels;
  name: string;
}

/** 对齐类名：Word 的 `w:jc` 只有这四种在预览里有意义。 */
const ALIGNMENT_CLASS: Readonly<Record<NonNullable<WordParagraphPreview['alignment']>, string>> = {
  left: 'text-left',
  center: 'text-center',
  right: 'text-right',
  justify: 'text-justify',
};

/** 列表项层级缩进：一层 16px，与 Word 默认缩进量级一致。 */
const LIST_INDENT_PIXELS = 16;

/**
 * 段落渲染。
 *
 * 标题从 `h2` 起：预览面板是宿主页面里的一个区域，宿主自己的 `h1` 才是页面标题；
 * 这里再放一个 `h1` 会让屏幕阅读器的标题大纲出现两个同级主标题。文档标题（若有）
 * 占 `h1`，正文标题顺延到 `h2`/`h3`/`h4`。
 */
function headingTagFor(level: number | undefined): { className: string; Tag: 'h2' | 'h3' | 'h4' } {
  if (level !== undefined && level <= 1) {
    return { Tag: 'h2', className: 'text-xl font-semibold' };
  }
  if (level === 2) {
    return { Tag: 'h3', className: 'text-lg font-semibold' };
  }
  return { Tag: 'h4', className: 'text-base font-semibold' };
}

/** 运行样式：`code` 在最内层，加粗在最外层，保证粗体代码、下划线斜体等组合都成立。 */
function renderRun(run: WordRunPreview, key: number): ReactNode {
  let content: ReactNode = run.text;
  if (run.code) {
    content = (
      <code className={`${PREVIEW_MONO_CLASS} rounded bg-neutral-100 px-1 py-0.5 dark:bg-neutral-800`}>
        {content}
      </code>
    );
  }
  if (run.underline) {
    content = <u>{content}</u>;
  }
  if (run.italic) {
    content = <em>{content}</em>;
  }
  if (run.bold) {
    content = <strong className="font-semibold">{content}</strong>;
  }
  return <Fragment key={key}>{content}</Fragment>;
}

function paragraphText(paragraph: WordParagraphPreview): string {
  return paragraph.runs.map((run) => run.text).join('');
}

function WordParagraph({ paragraph }: { paragraph: WordParagraphPreview }) {
  const alignmentClass = paragraph.alignment ? ALIGNMENT_CLASS[paragraph.alignment] : '';

  if (paragraph.kind === 'heading') {
    const { Tag, className } = headingTagFor(paragraph.level);
    return (
      <Tag className={`${className} ${alignmentClass} text-neutral-900 dark:text-neutral-50`}>
        {paragraph.runs.map(renderRun)}
      </Tag>
    );
  }

  if (paragraph.kind === 'listItem') {
    const indent = (paragraph.level ?? 0) * LIST_INDENT_PIXELS;
    return (
      // 缩进是「层级 × 常量」的连续值，Tailwind 的静态类名无法覆盖任意层级，所以走行内 style。
      <div
        className={`flex gap-2 text-sm leading-7 ${alignmentClass} text-neutral-700 dark:text-neutral-200`}
        style={{ paddingLeft: `${indent}px` }}
      >
        <span aria-hidden="true" className="select-none text-neutral-400 dark:text-neutral-500">•</span>
        <span className="min-w-0">{paragraph.runs.map(renderRun)}</span>
      </div>
    );
  }

  if (paragraph.runs.length === 0) {
    // Word 用空段落表达段间距；渲染成占位高度而不是空 `<p>`，避免屏幕阅读器读到空节点。
    return <div aria-hidden="true" className="h-3" />;
  }

  return (
    <p className={`text-sm leading-7 ${alignmentClass} text-neutral-700 dark:text-neutral-200`}>
      {paragraph.runs.map(renderRun)}
    </p>
  );
}

/**
 * Word 文档预览：`.docx` 解析成段落模型后按「纸张」排版渲染。
 *
 * 不渲染真实分页、页眉页脚与浮动图形：那需要完整的排版引擎。这里的目标是让用户
 * 一眼确认「打开的是哪份文档、里面大概写了什么」——标题层级、列表、粗斜下划线、
 * 对齐方式这四类信息已经足够，而且都能从 OOXML 稳定地读出来。
 */
export function WordDocumentPreview({ bytes, labels, name }: WordDocumentPreviewProps) {
  const [model, setModel] = useState<WordDocumentPreviewModel | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [pending, setPending] = useState(true);
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let active = true;
    setPending(true);
    setError(undefined);

    parseWordDocument(bytes)
      .then((result) => {
        // 卸载或换文件后到达的结果必须丢弃：否则会把上一份文档画到新文档上。
        if (!active) {
          return;
        }
        setModel(result);
        setPending(false);
      })
      .catch((cause: unknown) => {
        if (!active) {
          return;
        }
        setModel(undefined);
        setError(describePreviewError(cause, labels));
        setPending(false);
      });

    return () => {
      active = false;
    };
  }, [attempt, bytes, labels.loadFailed]);

  if (error !== undefined) {
    return (
      <PreviewErrorPanel
        labels={labels}
        message={error}
        onRetry={() => setAttempt((value) => value + 1)}
      />
    );
  }

  if (pending || !model) {
    return <PreviewLoadingPanel labels={labels} />;
  }

  const hasContent = model.paragraphs.some((paragraph) => paragraphText(paragraph).trim() !== '');
  if (!hasContent) {
    return <PreviewEmptyPanel labels={labels} name={name} />;
  }

  return (
    <div className={`${PREVIEW_SCROLL_CLASS} bg-neutral-100 px-4 py-6 dark:bg-neutral-950 sm:px-8`}>
      <article
        aria-label={name}
        className="mx-auto max-w-3xl rounded-lg bg-white px-6 py-10 shadow-sm dark:bg-neutral-900 sm:px-10"
      >
        {model.title ? (
          <h1 className="mb-8 text-xl font-semibold text-neutral-900 dark:text-neutral-50">
            {model.title}
          </h1>
        ) : null}
        <div className="space-y-3">
          {model.paragraphs.map((paragraph, index) => (
            // 段落模型没有稳定 id，索引就是文档顺序，重排不会发生（只读视图）。
            <WordParagraph key={index} paragraph={paragraph} />
          ))}
        </div>
      </article>
    </div>
  );
}
