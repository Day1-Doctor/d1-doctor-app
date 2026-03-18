import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { Artifact, ArtifactType } from "../../stores/artifactStore";

interface ArtifactCardProps {
  artifact: Artifact;
}

/** SVG icon by artifact type (no emoji). */
function ArtifactIcon({ type }: { type: ArtifactType }) {
  switch (type) {
    case "document":
      return (
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="text-accent shrink-0"
          aria-hidden="true"
        >
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
          <polyline points="14 2 14 8 20 8" />
          <line x1="16" y1="13" x2="8" y2="13" />
          <line x1="16" y1="17" x2="8" y2="17" />
          <polyline points="10 9 9 9 8 9" />
        </svg>
      );
    case "code":
      return (
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="text-success shrink-0"
          aria-hidden="true"
        >
          <polyline points="16 18 22 12 16 6" />
          <polyline points="8 6 2 12 8 18" />
        </svg>
      );
    case "chart":
      return (
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="text-warning shrink-0"
          aria-hidden="true"
        >
          <line x1="18" y1="20" x2="18" y2="10" />
          <line x1="12" y1="20" x2="12" y2="4" />
          <line x1="6" y1="20" x2="6" y2="14" />
        </svg>
      );
    case "image":
      return (
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="text-text-secondary shrink-0"
          aria-hidden="true"
        >
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
          <circle cx="8.5" cy="8.5" r="1.5" />
          <polyline points="21 15 16 10 5 21" />
        </svg>
      );
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  const mb = kb / 1024;
  return `${mb.toFixed(1)} MB`;
}

function formatTimestamp(ts: string): string {
  const date = new Date(ts);
  const hours = date.getHours().toString().padStart(2, "0");
  const minutes = date.getMinutes().toString().padStart(2, "0");
  return `${hours}:${minutes}`;
}

/**
 * ArtifactCard shows an artifact's icon, name, size, timestamp, and agent.
 * Click to preview content inline (for document/code types).
 * "Open" button as a placeholder for system-level open.
 */
export function ArtifactCard({ artifact }: ArtifactCardProps) {
  const [showPreview, setShowPreview] = useState(false);

  return (
    <div className="border border-border rounded-lg bg-card overflow-hidden transition-colors duration-100 hover:border-text-disabled">
      <button
        type="button"
        onClick={() => setShowPreview(!showPreview)}
        className="w-full flex items-center gap-2.5 px-3 py-2.5 text-left cursor-pointer select-none
          focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
      >
        <ArtifactIcon type={artifact.type} />

        <div className="flex flex-col gap-0.5 flex-1 min-w-0">
          <span className="text-text-primary text-[13px] font-medium truncate">
            {artifact.name}
          </span>
          <span className="text-text-disabled text-[13px]">
            {formatSize(artifact.size)} -- {artifact.agent} -- {formatTimestamp(artifact.timestamp)}
          </span>
        </div>

        {/* Open button (placeholder) */}
        <span
          className="text-[13px] text-accent hover:text-accent-hover px-1.5 py-0.5 rounded border border-accent/30 shrink-0"
          role="button"
          tabIndex={0}
          onClick={(e) => {
            e.stopPropagation();
            // placeholder: would open in system app
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.stopPropagation();
            }
          }}
        >
          Open
        </span>
      </button>

      {/* Inline preview for text content */}
      <AnimatePresence initial={false}>
        {showPreview && artifact.content && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.15, ease: "easeInOut" }}
            className="overflow-hidden"
          >
            <div className="px-3 pb-2.5 pt-1 border-t border-border">
              <pre className="bg-background rounded px-2.5 py-2 text-[12px] text-text-muted font-mono leading-relaxed max-h-[120px] overflow-y-auto whitespace-pre-wrap">
                {artifact.content}
              </pre>
            </div>
          </motion.div>
        )}
        {showPreview && !artifact.content && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.15, ease: "easeInOut" }}
            className="overflow-hidden"
          >
            <div className="px-3 pb-2.5 pt-1 border-t border-border">
              <p className="text-text-disabled text-[12px] text-center py-2">
                Preview not available for this file type
              </p>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
