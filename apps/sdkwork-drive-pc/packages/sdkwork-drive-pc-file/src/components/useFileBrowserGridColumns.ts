import { useEffect, useState } from 'react';

/** FileGridItem card height (145px) plus grid gap (16px). */
export const FILE_BROWSER_GRID_ROW_HEIGHT_PX = 161;

/** Matches `.sdkwork-drive-file-list-body--grid` responsive column breakpoints. */
export function resolveFileBrowserGridColumnCount(viewportWidth: number): number {
  if (viewportWidth >= 1280) {
    return 6;
  }
  if (viewportWidth >= 1024) {
    return 5;
  }
  if (viewportWidth >= 768) {
    return 4;
  }
  if (viewportWidth >= 640) {
    return 3;
  }
  return 2;
}

export function useFileBrowserGridColumns(enabled: boolean): number {
  const [columns, setColumns] = useState(() =>
    typeof window === 'undefined'
      ? 2
      : resolveFileBrowserGridColumnCount(window.innerWidth),
  );

  useEffect(() => {
    if (!enabled || typeof window === 'undefined') {
      return;
    }

    const updateColumns = () => {
      setColumns(resolveFileBrowserGridColumnCount(window.innerWidth));
    };

    updateColumns();
    window.addEventListener('resize', updateColumns);
    return () => window.removeEventListener('resize', updateColumns);
  }, [enabled]);

  return columns;
}
