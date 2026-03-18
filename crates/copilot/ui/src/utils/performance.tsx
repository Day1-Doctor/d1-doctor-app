import { useState, useEffect, useRef, useCallback } from "react";

/**
 * Hook that measures actual FPS via requestAnimationFrame.
 * Returns a rolling average updated every ~500ms.
 */
export function useFrameRate(): number {
  const [fps, setFps] = useState(0);
  const frameCountRef = useRef(0);
  const lastTimeRef = useRef(performance.now());
  const rafRef = useRef(0);

  const tick = useCallback((now: number) => {
    frameCountRef.current += 1;
    const elapsed = now - lastTimeRef.current;

    if (elapsed >= 500) {
      const measuredFps = Math.round((frameCountRef.current / elapsed) * 1000);
      setFps(measuredFps);
      frameCountRef.current = 0;
      lastTimeRef.current = now;
    }

    rafRef.current = requestAnimationFrame(tick);
  }, []);

  useEffect(() => {
    rafRef.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafRef.current);
  }, [tick]);

  return fps;
}

/**
 * FPS counter component for the Debug View.
 */
export function FpsCounter() {
  const fps = useFrameRate();

  const color = fps >= 50 ? "#22C55E" : fps >= 25 ? "#F59E0B" : "#EF4444";

  return (
    <div className="flex items-center gap-1.5 text-[12px] tabular-nums">
      <span
        className="inline-block w-1.5 h-1.5 rounded-full"
        style={{ backgroundColor: color }}
      />
      <span style={{ color }} className="font-medium">
        {fps} FPS
      </span>
    </div>
  );
}
