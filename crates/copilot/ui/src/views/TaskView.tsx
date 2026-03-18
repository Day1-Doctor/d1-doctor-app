export function TaskView() {
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
          <path d="M9 11l3 3L22 4" />
          <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
        </svg>
        <h2 className="text-xl font-semibold text-text-primary">
          Task View
        </h2>
      </div>
      <p className="text-text-muted text-sm">
        Timeline will render here
      </p>
      <div className="w-12 h-0.5 bg-border rounded-full" />
    </div>
  );
}
