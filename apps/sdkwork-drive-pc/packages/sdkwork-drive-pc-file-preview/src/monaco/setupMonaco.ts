import { loader } from '@monaco-editor/react';
import * as monaco from 'monaco-editor';

/**
 * Monaco 的本地化装配（与 VS Code 同源的编辑器栈）。
 *
 * 两件事必须在这里做一次，否则会静默降级：
 * 1. `loader.config({ monaco })` 把 `@monaco-editor/react` 从 CDN 拉取改成使用本包
 *    依赖里的 Monaco —— 商业交付不能依赖运行时能否访问 jsdelivr；
 * 2. `MonacoEnvironment.getWorker` 用 `new URL(..., import.meta.url)` 声明 worker，
 *    让打包器把它们切成独立 chunk（语言服务在 worker 里跑，主线程不被阻塞）。
 *
 * 本模块只被 `MonacoCodeSurface` 引用，而后者又是 lazy 加载的，所以 Monaco 的体积
 * 只落在真正打开代码文件的那条路径上。
 */
let configured = false;

interface MonacoWorkerEnvironment {
  getWorker(workerId: string, label: string): Worker;
}

function createWorker(label: string): Worker {
  switch (label) {
    case 'json':
      return new Worker(
        new URL('monaco-editor/esm/vs/language/json/json.worker.js', import.meta.url),
        { type: 'module' },
      );
    case 'css':
    case 'scss':
    case 'less':
      return new Worker(
        new URL('monaco-editor/esm/vs/language/css/css.worker.js', import.meta.url),
        { type: 'module' },
      );
    case 'html':
    case 'handlebars':
    case 'razor':
      return new Worker(
        new URL('monaco-editor/esm/vs/language/html/html.worker.js', import.meta.url),
        { type: 'module' },
      );
    case 'typescript':
    case 'javascript':
      return new Worker(
        new URL('monaco-editor/esm/vs/language/typescript/ts.worker.js', import.meta.url),
        { type: 'module' },
      );
    default:
      return new Worker(
        new URL('monaco-editor/esm/vs/editor/editor.worker.js', import.meta.url),
        { type: 'module' },
      );
  }
}

export function ensureMonacoConfigured(): void {
  if (configured) {
    return;
  }
  configured = true;
  const host = globalThis as typeof globalThis & { MonacoEnvironment?: MonacoWorkerEnvironment };
  /**
   * 合并而不是整体替换 `MonacoEnvironment`。
   *
   * 宿主自己也可能装配过 Monaco（Web Server 的 Server Config 页面就写过同一个全局对象）。
   * 整个对象覆盖会让先装配的一方丢掉 worker 工厂——表现为对方页面的语言服务静默失效，
   * 而且只在两个页面都挂载过之后才出现，极难定位。这里保留宿主的实现，只补我们认识的
   * worker；宿主已经能处理 label 时继续用它自己的。
   */
  const previous = host.MonacoEnvironment;
  // `Object.assign` 而不是直接赋值：宿主可能已经用 Monaco 自己的类型声明过这个全局量，
  // 直接赋值会与那套交叉签名冲突；这里只保证"保留已有实现 + 补上我们的工厂"。
  Object.assign(host, {
    MonacoEnvironment: {
      ...previous,
      getWorker: (workerId: string, label: string) =>
        previous?.getWorker?.(workerId, label) ?? createWorker(label),
    },
  });
  loader.config({ monaco });
}

/** 主题 id：Monaco 内置主题名与预览层的浅/深色对应表。 */
export function monacoThemeFor(colorMode: 'light' | 'dark'): string {
  return colorMode === 'dark' ? 'vs-dark' : 'light';
}

/**
 * 校验语言 id 是否真的被 Monaco 注册。
 *
 * Monaco 对未知 id 不会报错，只会静默按纯文本渲染；如果工具条仍然显示那个语言名，
 * 用户看到的就是「标着 TypeScript 却毫无着色」。所以渲染前问一次注册表，未注册的
 * 一律回落 `plaintext`，让界面显示的语言名与实际渲染一致。
 */
export function resolveMonacoLanguage(requested: string): string {
  if (requested === 'plaintext') {
    return requested;
  }
  const registered = monaco.languages
    .getLanguages()
    .some((language) => language.id === requested);
  return registered ? requested : 'plaintext';
}
