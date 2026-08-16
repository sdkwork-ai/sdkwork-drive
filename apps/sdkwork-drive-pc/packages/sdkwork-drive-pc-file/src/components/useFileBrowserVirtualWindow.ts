import { useEffect, useMemo, useRef, useState } from 'react';

interface VirtualWindowMetrics {
  scrollTop: number;
  viewportHeight: number;
}

interface VirtualWindowOptions {
  itemCount: number;
  itemHeight: number;
  /** Grid column count; defaults to 1 (list rows). */
  columns?: number;
  overscan?: number;
  /** When this value changes, scroll position and viewport metrics are reset. */
  resetKey?: string;
}

export function computeFileBrowserVirtualWindowRange(
  itemCount: number,
  itemHeight: number,
  columns: number,
  overscan: number,
  metrics: VirtualWindowMetrics,
) {
  const columnCount = Math.max(1, columns);
  const rowCount = itemCount === 0 ? 0 : Math.ceil(itemCount / columnCount);

  if (rowCount === 0 || metrics.viewportHeight <= 0) {
    return { rowStart: 0, rowEnd: 0, startIndex: 0, endIndex: 0 };
  }

  const unclampedRowStart = Math.max(0, Math.floor(metrics.scrollTop / itemHeight) - overscan);
  const rowStart = rowCount === 0 ? 0 : Math.min(unclampedRowStart, rowCount - 1);
  const visibleRows = Math.ceil(metrics.viewportHeight / itemHeight) + overscan * 2;
  const rowEnd = Math.min(rowCount, rowStart + visibleRows);
  const startIndex = rowStart * columnCount;
  const endIndex = Math.min(itemCount, rowEnd * columnCount);

  return {
    rowStart,
    rowEnd,
    startIndex,
    endIndex,
  };
}

export function useFileBrowserVirtualWindow({
  itemCount,
  itemHeight,
  columns = 1,
  overscan = 8,
  resetKey,
}: VirtualWindowOptions) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(0);

  useEffect(() => {
    setScrollTop(0);
    setViewportHeight(0);
    if (containerRef.current) {
      containerRef.current.scrollTop = 0;
    }
  }, [resetKey]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return;
    }

    const updateViewport = () => {
      setViewportHeight(container.clientHeight);
      setScrollTop(container.scrollTop);
    };

    updateViewport();

    const resizeObserver =
      typeof ResizeObserver !== 'undefined'
        ? new ResizeObserver(updateViewport)
        : undefined;
    resizeObserver?.observe(container);
    container.addEventListener('scroll', updateViewport, { passive: true });
    window.addEventListener('resize', updateViewport);

    return () => {
      resizeObserver?.disconnect();
      container.removeEventListener('scroll', updateViewport);
      window.removeEventListener('resize', updateViewport);
    };
  }, [itemCount, itemHeight, resetKey]);

  const columnCount = Math.max(1, columns);
  const rowCount = itemCount === 0 ? 0 : Math.ceil(itemCount / columnCount);

  const range = useMemo(
    () =>
      computeFileBrowserVirtualWindowRange(
        itemCount,
        itemHeight,
        columnCount,
        overscan,
        { scrollTop, viewportHeight },
      ),
    [columnCount, itemCount, itemHeight, overscan, scrollTop, viewportHeight],
  );

  return {
    containerRef,
    startIndex: range.startIndex,
    endIndex: range.endIndex,
    totalHeight: rowCount * itemHeight,
    offsetTop: range.rowStart * itemHeight,
    columnCount,
    visibleRowCount: range.rowEnd - range.rowStart,
  };
}
