export function OfficeView() {
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
          <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
          <polyline points="9 22 9 12 15 12 15 22" />
        </svg>
        <h2 className="text-xl font-semibold text-text-primary">
          Office View
        </h2>
      </div>
      <p className="text-text-muted text-sm">
        Canvas will render here
      </p>
      <div className="w-12 h-0.5 bg-border rounded-full" />
    </div>
  );
}
