import { useSettingsStore, type RiskLevel } from "../stores/settingsStore";

const MODEL_OPTIONS = [
  "claude-sonnet-4-20250514",
  "claude-opus-4-20250514",
  "claude-3-5-haiku-20241022",
  "gpt-4o",
  "gpt-4o-mini",
  "llama-3.3-70b",
];

const RISK_LABELS: Record<RiskLevel, { label: string; description: string; color: string }> = {
  low: { label: "Low", description: "Read-only operations, memory access", color: "#22C55E" },
  medium: { label: "Medium", description: "File writes, web requests", color: "#F59E0B" },
  high: { label: "High", description: "Shell commands, system access", color: "#F97316" },
  critical: { label: "Critical", description: "Destructive ops, external APIs", color: "#EF4444" },
};

export function SettingsView() {
  const providers = useSettingsStore((s) => s.providers);
  const agents = useSettingsStore((s) => s.agents);
  const autoApprove = useSettingsStore((s) => s.autoApprove);
  const updateProvider = useSettingsStore((s) => s.updateProvider);
  const updateAgent = useSettingsStore((s) => s.updateAgent);
  const setAutoApprove = useSettingsStore((s) => s.setAutoApprove);

  return (
    <div className="flex-1 overflow-y-auto px-6 py-4">
      <h2 className="text-lg font-semibold text-text-primary mb-6">Settings</h2>

      {/* Provider section */}
      <section className="mb-8">
        <h3 className="text-xs text-text-muted uppercase tracking-wider font-medium mb-3">
          Providers
        </h3>
        <div className="space-y-3">
          {providers.map((provider) => (
            <div
              key={provider.id}
              className="rounded-lg border border-border p-3"
              style={{ backgroundColor: "rgba(13, 13, 13, 0.6)" }}
            >
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-text-primary font-medium">{provider.name}</span>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={provider.enabled}
                    onChange={(e) => updateProvider(provider.id, { enabled: e.target.checked })}
                    className="sr-only"
                  />
                  <div
                    className={`w-8 h-4 rounded-full transition-colors duration-150 ${
                      provider.enabled ? "bg-accent" : "bg-muted"
                    }`}
                  >
                    <div
                      className={`w-3 h-3 bg-white rounded-full transition-transform duration-150 mt-0.5 ${
                        provider.enabled ? "translate-x-4 ml-0.5" : "translate-x-0.5"
                      }`}
                    />
                  </div>
                </label>
              </div>
              <div className="space-y-2">
                <div>
                  <label className="text-[10px] text-text-muted uppercase tracking-wider">
                    API Key
                  </label>
                  <input
                    type="password"
                    value={provider.apiKey}
                    onChange={(e) => updateProvider(provider.id, { apiKey: e.target.value })}
                    placeholder="sk-..."
                    className="w-full bg-card border border-border rounded px-2 py-1 text-xs text-text-primary
                      placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent/50
                      font-mono mt-0.5"
                  />
                </div>
                <div>
                  <label className="text-[10px] text-text-muted uppercase tracking-wider">
                    Endpoint
                  </label>
                  <input
                    type="text"
                    value={provider.endpoint}
                    onChange={(e) => updateProvider(provider.id, { endpoint: e.target.value })}
                    placeholder="https://..."
                    className="w-full bg-card border border-border rounded px-2 py-1 text-xs text-text-primary
                      placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent/50
                      font-mono mt-0.5"
                  />
                </div>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* Agent section */}
      <section className="mb-8">
        <h3 className="text-xs text-text-muted uppercase tracking-wider font-medium mb-3">
          Agents
        </h3>
        <div className="space-y-2">
          {agents.map((agent) => (
            <div
              key={agent.id}
              className="rounded-lg border border-border p-3 flex items-center gap-4"
              style={{ backgroundColor: "rgba(13, 13, 13, 0.6)" }}
            >
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm text-text-primary font-medium">{agent.name}</span>
                  <span className="text-[10px] text-text-muted uppercase">{agent.role}</span>
                </div>
                <div className="mt-1">
                  <select
                    value={agent.model}
                    onChange={(e) => updateAgent(agent.id, { model: e.target.value })}
                    className="bg-card border border-border rounded px-2 py-0.5 text-[11px] text-text-primary
                      focus:outline-none focus:ring-1 focus:ring-accent/50 font-mono cursor-pointer"
                  >
                    {MODEL_OPTIONS.map((m) => (
                      <option key={m} value={m}>{m}</option>
                    ))}
                  </select>
                </div>
              </div>
              <label className="relative inline-flex items-center cursor-pointer shrink-0">
                <input
                  type="checkbox"
                  checked={agent.enabled}
                  onChange={(e) => updateAgent(agent.id, { enabled: e.target.checked })}
                  className="sr-only"
                />
                <div
                  className={`w-8 h-4 rounded-full transition-colors duration-150 ${
                    agent.enabled ? "bg-accent" : "bg-muted"
                  }`}
                >
                  <div
                    className={`w-3 h-3 bg-white rounded-full transition-transform duration-150 mt-0.5 ${
                      agent.enabled ? "translate-x-4 ml-0.5" : "translate-x-0.5"
                    }`}
                  />
                </div>
              </label>
            </div>
          ))}
        </div>
      </section>

      {/* Approval section */}
      <section className="mb-8">
        <h3 className="text-xs text-text-muted uppercase tracking-wider font-medium mb-3">
          Auto-Approve by Risk Level
        </h3>
        <div className="space-y-2">
          {(Object.keys(RISK_LABELS) as RiskLevel[]).map((level) => {
            const info = RISK_LABELS[level];
            return (
              <label
                key={level}
                className="flex items-center gap-3 rounded-lg border border-border p-3 cursor-pointer hover:bg-muted/20"
                style={{ backgroundColor: "rgba(13, 13, 13, 0.6)" }}
              >
                <input
                  type="checkbox"
                  checked={autoApprove[level]}
                  onChange={(e) => setAutoApprove(level, e.target.checked)}
                  className="w-4 h-4 rounded border-border accent-accent"
                />
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <span
                      className="inline-block w-2 h-2 rounded-full"
                      style={{ backgroundColor: info.color }}
                    />
                    <span className="text-sm text-text-primary font-medium">{info.label}</span>
                  </div>
                  <p className="text-[11px] text-text-muted mt-0.5">{info.description}</p>
                </div>
              </label>
            );
          })}
        </div>
      </section>
    </div>
  );
}
