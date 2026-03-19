import { useTranslation } from "react-i18next";
import { useArtifactStore } from "../stores/artifactStore";
import { useTaskStore } from "../stores/taskStore";
import { useCostStore } from "../stores/costStore";
import { useEventLogStore } from "../stores/eventLogStore";
import { ArtifactCard } from "../components/task/ArtifactCard";

/**
 * Extract memory-like entries from event log.
 * Memory events come from the backend as tool invocations with toolName "memory".
 * If none exist, we return an empty array (no mock data).
 */
function useMemoryEntries() {
  const toolTraces = useEventLogStore((s) => s.toolTraces);
  return toolTraces
    .filter((t) => t.toolName === "memory" && t.status === "success")
    .map((t) => {
      let content = "";
      try {
        const params = JSON.parse(t.params);
        content = params.value ?? params.content ?? params.key ?? t.params;
      } catch {
        content = t.params;
      }
      return {
        id: t.id,
        content,
        agentName: t.agentName,
        ts: t.timestamp,
      };
    });
}

export function WorkspaceView() {
  const { t } = useTranslation();
  const artifacts = useArtifactStore((s) => s.artifacts);
  const tasks = useTaskStore((s) => s.tasks);
  const sessionCost = useCostStore((s) => s.sessionCost);
  const memoryEntries = useMemoryEntries();

  return (
    <div className="flex-1 flex flex-col min-h-0 overflow-y-auto">
      {/* Header */}
      <div className="shrink-0 px-6 py-4 border-b border-border">
        <h1 className="text-sm font-semibold text-text-primary">{t("workspace.title")}</h1>
      </div>

      <div className="flex-1 px-6 py-4 space-y-6">
        {/* Session summary */}
        <section className="flex items-center gap-4 text-[12px] text-text-muted">
          <span>{tasks.length} {t("workspace.tasks", { defaultValue: "tasks" })}</span>
          <span>{artifacts.length} {t("workspace.artifacts", { defaultValue: "artifacts" })}</span>
          {sessionCost > 0 && <span>{sessionCost} DD {t("workspace.spent", { defaultValue: "spent" })}</span>}
        </section>

        {/* Files & Artifacts section */}
        <section>
          <h2 className="text-[12px] uppercase tracking-wider text-text-muted font-semibold mb-2">
            {t("workspace.files")}
          </h2>
          {artifacts.length === 0 ? (
            <p className="text-text-disabled text-sm">{t("workspace.noFiles")}</p>
          ) : (
            <div className="space-y-2">
              {artifacts.map((artifact) => (
                <ArtifactCard key={artifact.id} artifact={artifact} />
              ))}
            </div>
          )}
        </section>

        {/* Memory section */}
        <section>
          <h2 className="text-[12px] uppercase tracking-wider text-text-muted font-semibold mb-2">
            {t("workspace.memory")}
            {memoryEntries.length > 0 && (
              <span className="ml-1.5 text-text-disabled normal-case font-normal">
                ({memoryEntries.length})
              </span>
            )}
          </h2>
          {memoryEntries.length === 0 ? (
            <p className="text-text-disabled text-sm">{t("workspace.noMemory")}</p>
          ) : (
            <div className="space-y-1.5">
              {memoryEntries.map((entry) => (
                <div
                  key={entry.id}
                  className="border border-border rounded-lg px-3 py-2 bg-card/60"
                >
                  <p className="text-[13px] text-text-primary leading-relaxed">{entry.content}</p>
                  <div className="flex items-center justify-between mt-1">
                    <span className="text-[12px] text-text-muted">{entry.agentName}</span>
                    <span className="text-[12px] text-text-muted">
                      {new Date(entry.ts).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
