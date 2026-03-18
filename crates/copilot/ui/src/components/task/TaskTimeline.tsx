import { useState } from "react";
import type { TaskStep } from "../../stores/taskStore";
import { StepBadge } from "./StepBadge";
import { StepConnector } from "./StepConnector";

interface TaskTimelineProps {
  steps: TaskStep[];
  className?: string;
}

/**
 * TaskTimeline renders a horizontal progression bar of task steps.
 * Each step is displayed as a StepBadge connected by StepConnectors.
 * Scrolls horizontally when steps overflow.
 */
export function TaskTimeline({ steps, className = "" }: TaskTimelineProps) {
  const [activeStepId, setActiveStepId] = useState<string | null>(null);

  if (steps.length === 0) {
    return (
      <div className={`flex items-center justify-center py-4 ${className}`}>
        <span className="text-text-muted text-xs">No steps to display</span>
      </div>
    );
  }

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
