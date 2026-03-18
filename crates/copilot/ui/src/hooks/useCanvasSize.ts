import { useEffect, useState, type RefObject } from "react";

interface CanvasSize {
  width: number;
  height: number;
}

/**
 * Tracks the content-box dimensions of a container element via ResizeObserver.
 * Returns `{ width, height }` in CSS pixels (not device pixels).
 */
export function useCanvasSize(
  containerRef: RefObject<HTMLDivElement | null>,
): CanvasSize {
  const [size, setSize] = useState<CanvasSize>({ width: 0, height: 0 });

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) return;
      const { width, height } = entry.contentRect;
      setSize({ width: Math.floor(width), height: Math.floor(height) });
    });

    observer.observe(el);
    return () => observer.disconnect();
  }, [containerRef]);

  return size;
}
