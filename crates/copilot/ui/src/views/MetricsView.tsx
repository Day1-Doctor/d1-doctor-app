import { useTranslation } from "react-i18next";
import { useAgentStore } from "../stores/agentStore";
import { useCostStore } from "../stores/costStore";
import { useOfficeStore } from "../stores/officeStore";

function StatCard({ label, value, unit }: { label: string; value: string | number; unit?: string }) {
  return (
    <div className="border border-border rounded-lg px-3 py-2.5 bg-card/60">
      <div className="text-[12px] uppercase tracking-wider text-text-muted mb-1">{label}</div>
      <div className="text-base font-semibold text-text-primary">
        {value}
        {unit && <span className="text-sm font-normal text-text-muted ml-1">{unit}</span>}
      </div>
    </div>
  );
}

export function MetricsView() {
  const { t } = useTranslation();
  const agents = useAgentStore((s) => s.agents);
  const agentCosts = useCostStore((s) => s.agentCosts);
  const sessionCost = useCostStore((s) => s.sessionCost);
  const balance = useCostStore((s) => s.balance);
  const offices = useOfficeStore((s) => s.offices);

  const totalTokens = 24_580; // placeholder
  const tasksDone = 3; // placeholder

  // Per-agent rows
  const agentRows = agents.map((agent) => ({
    name: agent.name,
    role: agent.role,
    cost: agentCosts[agent.name] ?? 0,
    tokens: Math.floor(Math.random() * 5000 + 500), // placeholder
    tasks: agent.status !== "idle" ? 1 : 0,
  }));

  return (
    <div className="flex-1 flex flex-col min-h-0 overflow-y-auto">
      {/* Header */}
      <div className="shrink-0 px-6 py-4 border-b border-border">
        <h1 className="text-sm font-semibold text-text-primary">{t("metrics.title")}</h1>
      </div>

      <div className="flex-1 px-6 py-4 space-y-6">
        {/* 4-stat grid */}
        <div className="grid grid-cols-2 gap-2">
          <StatCard label={t("metrics.totalCost")} value={sessionCost} unit="DD" />
          <StatCard label={t("metrics.totalTokens")} value={totalTokens.toLocaleString()} />
          <StatCard label={t("metrics.tasksDone")} value={tasksDone} />
          <StatCard label={t("metrics.ddBalance")} value={balance} unit="DD" />
        </div>

        {/* By Office */}
        <section>
          <h2 className="text-[12px] uppercase tracking-wider text-text-muted font-semibold mb-2">
            {t("metrics.byOffice")}
          </h2>
          <div className="space-y-2">
            {offices.map((office) => {
              const pct = office.taskProgress;
              return (
                <div key={office.id} className="border border-border rounded-lg px-3 py-2 bg-card/60">
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-[13px] text-text-primary">{office.name}</span>
                    <span className="text-[12px] text-text-muted">{office.agentCount} agents</span>
                  </div>
                  <div className="w-full h-1.5 bg-muted rounded-full overflow-hidden">
                    <div
                      className="h-full bg-accent rounded-full transition-all duration-300"
                      style={{ width: `${pct}%` }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        </section>

        {/* By Agent */}
        <section>
          <h2 className="text-[12px] uppercase tracking-wider text-text-muted font-semibold mb-2">
            {t("metrics.byAgent")}
          </h2>
          <div className="border border-border rounded-lg overflow-hidden">
            <table className="w-full text-[13px]">
              <thead>
                <tr className="border-b border-border bg-muted/30">
                  <th className="text-left px-3 py-1.5 text-text-muted font-medium">Agent</th>
                  <th className="text-right px-3 py-1.5 text-text-muted font-medium">Tokens</th>
                  <th className="text-right px-3 py-1.5 text-text-muted font-medium">Cost</th>
                  <th className="text-right px-3 py-1.5 text-text-muted font-medium">Tasks</th>
                </tr>
              </thead>
              <tbody>
                {agentRows.map((row) => (
                  <tr key={row.name} className="border-b border-border/50 last:border-0 hover:bg-muted/20">
                    <td className="px-3 py-1.5 text-text-primary">
                      <span className="font-medium">{row.name}</span>
                      <span className="text-text-muted ml-1 text-[12px]">{row.role}</span>
                    </td>
                    <td className="px-3 py-1.5 text-text-secondary text-right">{row.tokens.toLocaleString()}</td>
                    <td className="px-3 py-1.5 text-text-secondary text-right">{row.cost} DD</td>
                    <td className="px-3 py-1.5 text-text-secondary text-right">{row.tasks}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      </div>
    </div>
  );
}
