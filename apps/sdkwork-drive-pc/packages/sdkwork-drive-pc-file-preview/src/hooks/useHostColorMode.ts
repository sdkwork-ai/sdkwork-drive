import { useEffect, useState } from 'react';

export type HostColorMode = 'light' | 'dark';

/**
 * 读宿主当前配色模式。
 *
 * UI 框架把 `.dark` 类与 `data-sdk-color-mode` 放在 `<html>` 上（`SdkworkThemeProvider`
 * 的约定），Monaco 这类需要显式主题 id 的第三方组件拿不到 CSS 变量，只能读这个约定。
 * 用 MutationObserver 跟随切换，而不是在挂载时读一次——用户切主题时预览不该停在旧
 * 主题上。
 */
export function useHostColorMode(override?: HostColorMode): HostColorMode {
  const [mode, setMode] = useState<HostColorMode>(() => override ?? readHostColorMode());

  useEffect(() => {
    if (override) {
      setMode(override);
      return undefined;
    }
    const root = document.documentElement;
    setMode(readHostColorMode());
    const observer = new MutationObserver(() => setMode(readHostColorMode()));
    observer.observe(root, { attributes: true, attributeFilter: ['class', 'data-sdk-color-mode'] });
    return () => observer.disconnect();
  }, [override]);

  return mode;
}

function readHostColorMode(): HostColorMode {
  if (typeof document === 'undefined') {
    return 'light';
  }
  const root = document.documentElement;
  if (root.classList.contains('dark')) {
    return 'dark';
  }
  return root.getAttribute('data-sdk-color-mode') === 'dark' ? 'dark' : 'light';
}
