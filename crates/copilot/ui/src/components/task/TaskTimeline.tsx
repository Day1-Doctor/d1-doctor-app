import { useState, useMemo } from "react";
import type { TaskStep } from "../../stores/taskStore";
import { StepBadge } from "./StepBadge";
import { StepConnector } from "./StepConnector";

interface TaskTimelineProps {
  steps: TaskStep[];
  className?: string;
}

/** A segment is either a single sequential step or a group of parallel steps. */
type TimelineSegment =
  | { type: "single"; step: TaskStep }
  | { type: "parallel"; group: string; steps: TaskStep[] };

/**
 * Organize flat steps into segments: sequential singles and parallel groups.
 */
function buildSegments(steps: TaskStep[]): TimelineSegment[] {
  const segments: TimelineSegment[] = [];
  const parallelMap = new Map<string, TaskStep[]>();
  const groupOrder: string[] = [];

  for (const step of steps) {
    if (step.isParallel && step.parallelGroup) {
      if (!parallelMap.has(step.parallelGroup)) {
        parallelMap.set(step.parallelGroup, []);
        groupOrder.push(step.parallelGroup);
      }
      parallelMap.get(step.parallelGroup)!.push(step);
    } else {
      // Flush any accumulated parallel group that was inserted before this
      // (parallel groups are flushed when we encounter the first non-parallel
      //  step after them, but we also need to handle the case where the
      //  parallel group appears at a specific position)
      segments.push({ type: "single", step });
    }
  }

  // Now re-build with parallel groups inserted at the right position
  const result: TimelineSegment[] = [];
  const groupsInserted = new Set<string>();
  for (const step of steps) {
    if (step.isParallel && step.parallelGroup) {
      if (!groupsInserted.has(step.parallelGroup)) {
        groupsInserted.add(step.parallelGroup);
        result.push({
          type: "parallel",
          group: step.parallelGroup,
          steps: parallelMap.get(step.parallelGroup)!,
        });
      }
    } else {
      result.push({ type: "single", step });
    }
  }
  return result;
}

/** Returns the aggregate status for a parallel group (used for connectors). */
function parallelGroupStatus(steps: TaskStep[]): TaskStep["status"] {
  if (steps.every((s) => s.status === "completed")) return "completed";
  if (steps.some((s) => s.status === "running")) return "running";
  if (steps.some((s) => s.status === "failed")) return "failed";
  return "pending";
}

/** Fork/join SVG node rendered at diverge/converge points. */
function ForkJoinNode({ variant }: { variant: "fork" | "join" }) {
  return (
    <div
      className="flex items-center justify-center shrink-0"
      style={{ width: 28, height: 28 }}
      aria-label={variant === "fork" ? "Parallel fork" : "Parallel join"}
    >
      <svg width="28" height="28" viewBox="0 0 28 28" fill="none">
        <circle cx="14" cy="14" r="10" stroke="#F97316" strokeWidth="1.5" fill="#0D0D0D" />
        {variant === "fork" ? (
          <>
            <line x1="14" y1="8" x2="9" y2="20" stroke="#F97316" strokeWidth="1.5" strokeLinecap="round" />
            <line x1="14" y1="8" x2="19" y2="20" stroke="#F97316" strokeWidth="1.5" strokeLinecap="round" />
          </>
        ) : (
          <>
            <line x1="9" y1="8" x2="14" y2="20" stroke="#F97316" strokeWidth="1.5" strokeLinecap="round" />
            <line x1="19" y1="8" x2="14" y2="20" stroke="#F97316" strokeWidth="1.5" strokeLinecap="round" />
          </>
        )}
      </svg>
    </div>
  );
}

/**
 * TaskTimeline renders a horizontal progression bar of task steps.
 * Each step is displayed as a StepBadge connected by StepConnectors.
 * Supports parallel branches: steps with isParallel/parallelGroup are
 * rendered as stacked vertical lanes between fork and join points.
 */
export function TaskTimeline({ steps, className = "" }: TaskTimelineProps) {
  const [activeStepId, setActiveStepId] = useState<string | null>(null);

  const segments = useMemo(() => buildSegments(steps), [steps]);
  const hasParallel = segments.some((s) => s.type === "parallel");

  if (steps.length === 0) {
    return (
      <div className={`flex items-center justify-center py-4 ${className}`}>
        <span className="text-text-muted text-sm">No steps to display</span>
      </div>
    );
  }

  // Simple linear rendering (no parallel branches)
  if (!hasParallel) {
    return (
      <div
        className={`overflow-x-auto scrollbar-thin ${className}`}
        role="list"
        aria-label="Task timeline"
      >
        <div className="flex items-center gap-0 py-2 px-2 min-w-max">
          {steps.map((step, index) => (
            <div key={step.id} className="flex items-center" role="listitem">
              <StepBadge
                step={step}
                isActive={activeStepId === step.id}
                onClick={() =>
                  setActiveStepId(
                    activeStepId === step.id ? null : step.id,
                  )
                }
              />
              {index < steps.length - 1 && (
                <StepConnector
                  fromStatus={step.status}
                  toStatus={steps[index + 1].status}
                />
              )}
            </div>
          ))}
        </div>
      </div>
    );
  }

  // Parallel-aware rendering
  return (
    <div
      className={`overflow-x-auto scrollbar-thin ${className}`}
      role="list"
      aria-label="Task timeline"
    >
      <div className="flex items-center gap-0 py-2 px-2 min-w-max">
        {segments.map((segment, segIdx) => {
          // Determine the status of the previous segment (for connectors)
          const prevSegment = segIdx > 0 ? segments[segIdx - 1] : null;
          const prevStatus: TaskStep["status"] | null = prevSegment
            ? prevSegment.type === "single"
              ? prevSegment.step.status
              : parallelGroupStatus(prevSegment.steps)
            : null;

          if (segment.type === "single") {
            const nextSegment = segIdx < segments.length - 1 ? segments[segIdx + 1] : null;
            return (
              <div key={segment.step.id} className="flex items-center" role="listitem">
                {prevStatus !== null && (
                  <StepConnector
                    fromStatus={prevStatus}
                    toStatus={segment.step.status}
                  />
                )}
                <StepBadge
                  step={segment.step}
                  isActive={activeStepId === segment.step.id}
                  onClick={() =>
                    setActiveStepId(
                      activeStepId === segment.step.id ? null : segment.step.id,
                    )
                  }
                />
                {/* If next segment exists and is parallel, we draw the connector+fork inline below */}
                {nextSegment && nextSegment.type === "single" && (
                  <StepConnector
                    fromStatus={segment.step.status}
                    toStatus={nextSegment.step.status}
                  />
                )}
              </div>
            );
          }

          // Parallel group rendering
          const groupStatus = parallelGroupStatus(segment.steps);
          const nextSegment = segIdx < segments.length - 1 ? segments[segIdx + 1] : null;
          const nextStatus: TaskStep["status"] | null = nextSegment
            ? nextSegment.type === "single"
              ? nextSegment.step.status
              : parallelGroupStatus(nextSegment.steps)
            : null;

          return (
            <div key={segment.group} className="flex items-center" role="group" aria-label="Parallel steps">
              {/* Connector from previous to fork */}
              {prevStatus !== null && (
                <StepConnector fromStatus={prevStatus} toStatus={groupStatus} />
              )}
              <ForkJoinNode variant="fork" />

              {/* Parallel lanes stacked vertically */}
              <div className="flex flex-col gap-1.5 mx-1">
                {segment.steps.map((step) => (
                  <div key={step.id} role="listitem">
                    <StepBadge
                      step={step}
                      isActive={activeStepId === step.id}
                      onClick={() =>
                        setActiveStepId(
                          activeStepId === step.id ? null : step.id,
                        )
                      }
                    />
                  </div>
                ))}
              </div>

              <ForkJoinNode variant="join" />
              {/* Connector from join to next */}
              {nextStatus !== null && (
                <StepConnector fromStatus={groupStatus} toStatus={nextStatus} />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
