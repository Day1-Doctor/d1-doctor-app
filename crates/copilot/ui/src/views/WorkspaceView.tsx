import { useTranslation } from "react-i18next";
import { useArtifactStore } from "../stores/artifactStore";
import { ArtifactCard } from "../components/task/ArtifactCard";

/** Mock memory entries for placeholder display. */
const mockMemoryEntries = [
  { id: "mem-1", content: "User prefers concise summaries over verbose explanations.", ts: "2026-03-18T09:00:00Z" },
  { id: "mem-2", content: "Project uses TypeScript strict mode throughout.", ts: "2026-03-18T09:15:00Z" },
  { id: "mem-3", content: "API base URL: https://api.day1.doctor/v1", ts: "2026-03-18T09:30:00Z" },
];

export function WorkspaceView() {
  const { t } = useTranslation();
  const artifacts = useArtifactStore((s) => s.artifacts);

  return (
    <div className="flex-1 flex flex-col min-h-0 overflow-y-auto">
      {/* Header */}
      <div className="shrink-0 px-6 py-4 border-b border-border">
        <h1 className="text-sm font-semibold text-text-primary">{t("workspace.title")}</h1>
      </div>

      <div className="flex-1 px-6 py-4 space-y-6">
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
            <span className="ml-1.5 text-text-disabled normal-case font-normal">
              ({mockMemoryEntries.length})
            </span>
          </h2>
          {mockMemoryEntries.length === 0 ? (
            <p className="text-text-disabled text-sm">{t("workspace.noMemory")}</p>
          ) : (
            <div className="space-y-1.5">
              {mockMemoryEntries.map((entry) => (
                <div
                  key={entry.id}
                  className="border border-border rounded-lg px-3 py-2 bg-card/60"
                >
                  <p className="text-[13px] text-text-primary leading-relaxed">{entry.content}</p>
                  <p className="text-[13px] text-text-muted mt-1">
                    {new Date(entry.ts).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                  </p>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
