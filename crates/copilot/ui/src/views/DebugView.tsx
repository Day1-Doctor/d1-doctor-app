export function DebugView() {
  return (
    <div className="flex-1 flex flex-col items-center justify-center gap-4">
      <div className="flex items-center gap-3">
        <svg
          width="32"
          height="32"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="text-text-muted"
          aria-hidden="true"
        >
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <h2 className="text-xl font-semibold text-text-primary">
          Debug View
        </h2>
      </div>
      <p className="text-text-muted text-sm">
        Event log will render here
      </p>
      <div className="w-12 h-0.5 bg-border rounded-full" />
    </div>
  );
}
