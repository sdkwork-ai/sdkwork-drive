/**
 * 文件种类判定：列表图标、预览分发、编辑器选型都读这一份判定。
 *
 * 判定顺序是「扩展名优先、content_type 兜底」：对象存储里 `content_type` 经常是
 * `application/octet-stream`（上传方没给），而扩展名是用户真实意图；反过来，某些
 * 无扩展名的导出对象只有 content_type 能说明它是什么。
 */

/** 预览/图标使用的文件种类。 */
export type FilePreviewKind =
  | 'folder'
  | 'image'
  | 'video'
  | 'audio'
  | 'pdf'
  | 'word'
  | 'spreadsheet'
  | 'presentation'
  | 'archive'
  | 'text'
  | 'markdown'
  | 'code'
  | 'unsupported';

/**
 * 全部种类的运行时清单。
 *
 * 视觉表与对比度闸门都要遍历它：`FILE_KIND_VISUALS` 是 `Record<FilePreviewKind, …>`，
 * 只有在这里留一份运行时可枚举的清单，才能保证"新增一种 kind 就必须同时给出视觉定义"
 * 这句话是被测试守住的，而不是靠人记得。
 */
export const FILE_PREVIEW_KINDS = [
  'folder',
  'image',
  'video',
  'audio',
  'pdf',
  'word',
  'spreadsheet',
  'presentation',
  'archive',
  'text',
  'markdown',
  'code',
  'unsupported',
] as const satisfies readonly FilePreviewKind[];

const EXTENSION_KIND: Readonly<Record<string, FilePreviewKind>> = {
  // 图片
  png: 'image', jpg: 'image', jpeg: 'image', gif: 'image', webp: 'image', bmp: 'image',
  svg: 'image', ico: 'image', tif: 'image', tiff: 'image', avif: 'image', heic: 'image', heif: 'image',
  // 视频
  mp4: 'video', m4v: 'video', mov: 'video', mkv: 'video', webm: 'video', avi: 'video',
  wmv: 'video', flv: 'video', mpeg: 'video', mpg: 'video', '3gp': 'video',
  // 音频
  mp3: 'audio', wav: 'audio', flac: 'audio', aac: 'audio', ogg: 'audio', oga: 'audio',
  m4a: 'audio', wma: 'audio', opus: 'audio', aiff: 'audio', mid: 'audio', midi: 'audio',
  // 文档
  pdf: 'pdf',
  doc: 'word', docx: 'word', rtf: 'word', odt: 'word', pages: 'word',
  xls: 'spreadsheet', xlsx: 'spreadsheet', xlsm: 'spreadsheet', csv: 'spreadsheet',
  tsv: 'spreadsheet', ods: 'spreadsheet', numbers: 'spreadsheet',
  ppt: 'presentation', pptx: 'presentation', odp: 'presentation', key: 'presentation',
  // 归档
  zip: 'archive', rar: 'archive', '7z': 'archive', tar: 'archive', gz: 'archive', tgz: 'archive',
  bz2: 'archive', xz: 'archive', zst: 'archive', jar: 'archive', war: 'archive',
  // 文本 / 标记
  txt: 'text', log: 'text', ini: 'text', conf: 'text', cfg: 'text', env: 'text',
  md: 'markdown', markdown: 'markdown', mdx: 'markdown',
  // 代码
  js: 'code', jsx: 'code', mjs: 'code', cjs: 'code', ts: 'code', tsx: 'code', mts: 'code', cts: 'code',
  json: 'code', jsonc: 'code', json5: 'code', yml: 'code', yaml: 'code', toml: 'code', xml: 'code',
  html: 'code', htm: 'code', css: 'code', scss: 'code', sass: 'code', less: 'code',
  vue: 'code', svelte: 'code', astro: 'code',
  py: 'code', rb: 'code', php: 'code', java: 'code', kt: 'code', kts: 'code', scala: 'code',
  go: 'code', rs: 'code', c: 'code', h: 'code', cc: 'code', cpp: 'code', hpp: 'code', cs: 'code',
  swift: 'code', m: 'code', mm: 'code', dart: 'code', lua: 'code', pl: 'code', r: 'code',
  sh: 'code', bash: 'code', zsh: 'code', fish: 'code', ps1: 'code', bat: 'code', cmd: 'code',
  sql: 'code', graphql: 'code', gql: 'code', proto: 'code', tf: 'code', hcl: 'code', dockerfile: 'code',
  makefile: 'code', gradle: 'code', properties: 'code', lock: 'code',
};

/** 无扩展名但按名字即代码/文本的常见文件。 */
const WELL_KNOWN_NAME_KIND: Readonly<Record<string, FilePreviewKind>> = {
  dockerfile: 'code',
  makefile: 'code',
  'cmakelists.txt': 'code',
  '.gitignore': 'text',
  '.gitattributes': 'text',
  '.npmrc': 'text',
  '.editorconfig': 'text',
  '.env': 'text',
  license: 'text',
  notice: 'text',
  readme: 'text',
  changelog: 'text',
};

const CONTENT_TYPE_KIND: readonly (readonly [string, FilePreviewKind])[] = [
  ['image/', 'image'],
  ['video/', 'video'],
  ['audio/', 'audio'],
  ['application/pdf', 'pdf'],
  ['application/msword', 'word'],
  ['application/vnd.openxmlformats-officedocument.wordprocessingml', 'word'],
  ['application/vnd.oasis.opendocument.text', 'word'],
  ['application/vnd.ms-excel', 'spreadsheet'],
  ['application/vnd.openxmlformats-officedocument.spreadsheetml', 'spreadsheet'],
  ['text/csv', 'spreadsheet'],
  ['application/vnd.ms-powerpoint', 'presentation'],
  ['application/vnd.openxmlformats-officedocument.presentationml', 'presentation'],
  ['application/zip', 'archive'],
  ['application/x-zip-compressed', 'archive'],
  ['application/x-tar', 'archive'],
  ['application/gzip', 'archive'],
  ['application/x-7z-compressed', 'archive'],
  ['application/x-rar-compressed', 'archive'],
  ['text/markdown', 'markdown'],
  ['application/json', 'code'],
  ['application/xml', 'code'],
  ['application/javascript', 'code'],
  ['application/x-yaml', 'code'],
  ['text/html', 'code'],
  ['text/css', 'code'],
  ['text/x-python', 'code'],
  ['text/plain', 'text'],
  ['text/', 'text'],
];

/** key/文件名的小写扩展名（不含点）；没有扩展名时返回 `''`。 */
export function fileExtension(fileName: string): string {
  const name = fileName.split('/').filter(Boolean).at(-1) ?? fileName;
  const dot = name.lastIndexOf('.');
  if (dot <= 0 || dot === name.length - 1) {
    return '';
  }
  return name.slice(dot + 1).toLowerCase();
}

export interface FileKindInput {
  /** 文件名或对象 key（可含目录前缀）。 */
  name: string;
  contentType?: string;
  isFolder?: boolean;
}

export function resolveFilePreviewKind({ name, contentType, isFolder }: FileKindInput): FilePreviewKind {
  if (isFolder || name.endsWith('/')) {
    return 'folder';
  }
  const extension = fileExtension(name);
  if (extension && EXTENSION_KIND[extension]) {
    return EXTENSION_KIND[extension];
  }
  const baseName = (name.split('/').filter(Boolean).at(-1) ?? name).toLowerCase();
  if (WELL_KNOWN_NAME_KIND[baseName]) {
    return WELL_KNOWN_NAME_KIND[baseName];
  }
  const normalizedContentType = contentType?.toLowerCase();
  if (normalizedContentType) {
    for (const [prefix, kind] of CONTENT_TYPE_KIND) {
      if (normalizedContentType.startsWith(prefix)) {
        return kind;
      }
    }
  }
  return 'unsupported';
}

/** 有独立预览实现的种类；`folder` 与 `unsupported` 不在其中。 */
export function isPreviewableKind(kind: FilePreviewKind): boolean {
  return kind !== 'folder' && kind !== 'unsupported';
}

/** 可以在编辑器里改写并保存回去的种类。 */
export function isEditableKind(kind: FilePreviewKind): boolean {
  return kind === 'text' || kind === 'code' || kind === 'markdown';
}

/** 文本类（按 UTF-8 读取有意义）的种类。 */
export function isTextLikeKind(kind: FilePreviewKind): boolean {
  return isEditableKind(kind);
}

/** 需要原始字节而非文本的种类（二进制解析）。 */
export function isBinaryKind(kind: FilePreviewKind): boolean {
  return kind === 'word' || kind === 'spreadsheet' || kind === 'presentation' || kind === 'archive';
}

/**
 * 分隔符文本表格（`.csv` / `.tsv`）。
 *
 * 它们与 xlsx 共用「表格」这一种类，但**不是** OOXML 容器：预览前的容器校验必须把它们
 * 排除，否则一份完全正常的 CSV 会被判定成「旧版二进制格式」。
 */
export function isDelimitedTextFile(name: string): boolean {
  const extension = fileExtension(name);
  return extension === 'csv' || extension === 'tsv';
}

/** 媒体与 PDF 优先走宿主直链（避免整文件 base64 往返）。 */
export function prefersDirectUrl(kind: FilePreviewKind): boolean {
  return kind === 'image' || kind === 'video' || kind === 'audio' || kind === 'pdf';
}

/**
 * Monaco 语言 id（与 VS Code 的语言标识一致）。
 *
 * 只列 Monaco 0.55 实际注册过的 id：`vs/basic-languages`（81 个）加上内置语言服务
 * （json / typescript / css / html）。写错一个 id 不会报错——Monaco 静默回落成纯文本，
 * 而工具条仍会显示那个语言名，等于对用户撒谎；所以映射之外的扩展名一律回落
 * `plaintext`，并由 `resolveMonacoLanguage` 在运行时再校验一次。
 */
const MONACO_LANGUAGE: Readonly<Record<string, string>> = {
  js: 'javascript', jsx: 'javascript', mjs: 'javascript', cjs: 'javascript',
  ts: 'typescript', tsx: 'typescript', mts: 'typescript', cts: 'typescript',
  json: 'json', jsonc: 'json', json5: 'json',
  yml: 'yaml', yaml: 'yaml',
  md: 'markdown', markdown: 'markdown', mdx: 'markdown',
  html: 'html', htm: 'html', vue: 'html', svelte: 'html',
  css: 'css', scss: 'scss', sass: 'scss', less: 'less',
  py: 'python', rb: 'ruby', php: 'php', java: 'java', kt: 'kotlin', kts: 'kotlin',
  scala: 'scala', go: 'go', rs: 'rust', c: 'c', h: 'c', cc: 'cpp', cpp: 'cpp',
  hpp: 'cpp', cs: 'csharp', fs: 'fsharp', swift: 'swift', dart: 'dart', lua: 'lua',
  pl: 'perl', r: 'r', jl: 'julia', clj: 'clojure', ex: 'elixir', exs: 'elixir',
  sh: 'shell', bash: 'shell', zsh: 'shell', fish: 'shell', ps1: 'powershell',
  bat: 'bat', cmd: 'bat',
  sql: 'sql', graphql: 'graphql', gql: 'graphql', proto: 'protobuf',
  ini: 'ini', toml: 'ini', cfg: 'ini', conf: 'ini', properties: 'ini',
  xml: 'xml', hcl: 'hcl', tf: 'hcl', sol: 'solidity', vb: 'vb', tcl: 'tcl',
  lock: 'json',
};

export function monacoLanguageForFile(name: string): string {
  const extension = fileExtension(name);
  if (extension && MONACO_LANGUAGE[extension]) {
    return MONACO_LANGUAGE[extension];
  }
  const baseName = (name.split('/').filter(Boolean).at(-1) ?? name).toLowerCase();
  if (baseName === 'dockerfile' || baseName.startsWith('dockerfile.')) {
    return 'dockerfile';
  }
  // Makefile 在 Monaco 里没有对应的基础语言：与其显示一个并不存在的语言名，
  // 不如老实按纯文本打开（`plaintext` 是 Monaco 内置的合法 id）。
  return 'plaintext';
}
