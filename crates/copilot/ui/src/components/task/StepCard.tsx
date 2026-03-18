import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { TaskStep } from "../../stores/taskStore";

interface StepCardProps {
  step: TaskStep;
}

const statusLabel: Record<TaskStep["status"], string> = {
  pending: "Pending",
  running: "Running",
  completed: "Completed",
  failed: "Failed",
  cancelled: "Cancelled",
};

const statusBadgeColor: Record<TaskStep["status"], string> = {
  pending: "bg-text-muted/20 text-text-muted",
  running: "bg-accent/20 text-accent",
  completed: "bg-success/20 text-success",
  failed: "bg-error/20 text-error",
  cancelled: "bg-text-muted/20 text-text-muted",
};

function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remaining = seconds % 60;
  return `${minutes}m ${remaining}s`;
}

/** Mock tool calls for expanded detail view. */
interface ToolCall {
  name: string;
  duration: number;
  status: "success" | "error";
}

function getMockToolCalls(stepId: string): ToolCall[] {
  const toolSets: Record<string, ToolCall[]> = {
    "1": [
      { name: "web_search", duration: 12000, status: "success" },
      { name: "read_document", duration: 8500, status: "success" },
      { name: "summarize", duration: 5200, status: "success" },
    ],
    "2": [
      { name: "data_analysis", duration: 15000, status: "success" },
      { name: "compare_results", duration: 3200, status: "success" },
    ],
    "p-0": [
      { name: "web_search", duration: 18000, status: "success" },
      { name: "parse_results", duration: 9000, status: "success" },
    ],
    "p-1a": [
      { name: "data_analysis", duration: 7000, status: "success" },
    ],
    "p-1b": [
      { name: "review_sources", duration: 4500, status: "success" },
    ],
  };
  return toolSets[stepId] ?? [
    { name: "generic_tool", duration: 2000, status: "success" },
  ];
}

function getMockTokenUsage(stepId: string): { input: number; output: number } {
  const usages: Record<string, { input: number; output: number }> = {
    "1": { input: 2400, output: 1800 },
    "2": { input: 3100, output: 2200 },
    "3": { input: 1500, output: 3400 },
    "4": { input: 800, output: 400 },
    "p-0": { input: 2000, output: 1600 },
    "p-1a": { input: 1800, output: 1200 },
    "p-1b": { input: 1400, output: 900 },
    "p-2": { input: 1000, output: 2800 },
  };
  return usages[stepId] ?? { input: 500, output: 300 };
}

function getMockRawOutput(stepId: string): string {
  const outputs: Record<string, string> = {
    "1": "Searched 12 sources across academic papers, documentation, and benchmarks. Found 8 relevant frameworks with active communities...",
    "2": "Comparative analysis complete. TensorFlow leads in production deployments, PyTorch in research adoption. JAX showing fastest growth...",
    "3": "Report draft pending. Will synthesize analysis from previous steps into a structured comparison document...",
    "p-0": "Research phase complete. Gathered primary and secondary sources from 15 databases...",
    "p-1a": "Running statistical analysis on performance benchmarks across 5 hardware configurations...",
    "p-1b": "Cross-referencing 23 sources for accuracy and recency. 18/23 verified so far...",
  };
  return outputs[stepId] ?? "Step output will appear here when available.";
}

/**
 * StepCard is an expandable card for each task step.
 * Collapsed: step title, status badge, agent name, duration.
 * Expanded: tool calls list, token usage, raw output preview.
 */
export function StepCard({ step }: StepCardProps) {
  const [expanded, setExpanded] = useState(false);
  const toolCalls = getMockToolCalls(step.id);
  const tokenUsage = getMockTokenUsage(step.id);
  const rawOutput = getMockRawOutput(step.id);

  return (
    <div
      className="border border-border rounded-lg bg-card overflow-hidden transition-colors duration-100 hover:border-text-disabled"
    >
      {/* Collapsed header - always visible */}
      <button
        type="button"
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-3 px-4 py-3 text-left cursor-pointer select-none
          focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
        aria-expanded={expanded}
      >
        {/* Expand/collapse chevron */}
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          className={`text-text-muted shrink-0 transition-transform duration-200 ${
            expanded ? "rotate-90" : ""
          }`}
          aria-hidden="true"
        >
          <polyline points="9 18 15 12 9 6" />
        </svg>

        {/* Title */}
        <span className="text-text-primary text-sm font-medium truncate flex-1">
          {step.title}
        </span>

        {/* Status badge */}
        <span
          className={`text-[12px] font-medium px-2 py-0.5 rounded-full shrink-0 ${statusBadgeColor[step.status]}`}
        >
          {statusLabel[step.status]}
        </span>

        {/* Agent name */}
        {step.agentName && (
          <span className="text-text-muted text-[12px] shrink-0">
            {step.agentName}
          </span>
        )}

        {/* Duration */}
        {step.duration != null && (
          <span className="text-text-disabled text-[12px] shrink-0">
            {formatDuration(step.duration)}
          </span>
        )}
      </button>

      {/* Expanded detail */}
      <AnimatePresence initial={false}>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2, ease: "easeInOut" }}
            className="overflow-hidden"
          >
            <div className="px-4 pb-3 pt-1 space-y-3 border-t border-border">
              {/* Tool calls */}
              <div>
                <h4 className="text-[12px] text-text-secondary font-medium uppercase tracking-wider mb-1.5">
                  Tool Calls
                </h4>
                <div className="space-y-1">
                  {toolCalls.map((tc, i) => (
                    <div
                      key={`${tc.name}-${i}`}
                      className="flex items-center gap-2 text-[12px]"
                    >
                      <span
                        className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                          tc.status === "success" ? "bg-success" : "bg-error"
                        }`}
                      />
                      <span className="text-text-primary font-mono">{tc.name}</span>
                      <span className="text-text-disabled ml-auto">
                        {formatDuration(tc.duration)}
                      </span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Token usage */}
              <div>
                <h4 className="text-[12px] text-text-secondary font-medium uppercase tracking-wider mb-1.5">
                  Token Usage
                </h4>
                <div className="flex gap-4 text-[12px]">
                  <span className="text-text-muted">
                    Input: <span className="text-text-primary">{tokenUsage.input.toLocaleString()}</span>
                  </span>
                  <span className="text-text-muted">
                    Output: <span className="text-text-primary">{tokenUsage.output.toLocaleString()}</span>
                  </span>
                  <span className="text-text-muted">
                    Total: <span className="text-text-primary">{(tokenUsage.input + tokenUsage.output).toLocaleString()}</span>
                  </span>
                </div>
              </div>

              {/* Raw output preview */}
              <div>
                <h4 className="text-[12px] text-text-secondary font-medium uppercase tracking-wider mb-1.5">
                  Output Preview
                </h4>
                <div className="bg-background rounded px-3 py-2 text-[12px] text-text-muted font-mono leading-relaxed max-h-[80px] overflow-y-auto">
                  {rawOutput}
                </div>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
