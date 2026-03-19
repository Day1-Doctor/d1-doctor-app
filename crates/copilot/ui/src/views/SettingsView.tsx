import { useTranslation } from "react-i18next";
import { useSettingsStore, type RiskLevel } from "../stores/settingsStore";

const MODEL_OPTIONS = [
  "claude-sonnet-4-20250514",
  "claude-opus-4-20250514",
  "claude-3-5-haiku-20241022",
  "gpt-4o",
  "gpt-4o-mini",
  "llama-3.3-70b",
];

const RISK_LEVELS: RiskLevel[] = ["low", "medium", "high", "critical"];

const RISK_COLORS: Record<RiskLevel, string> = {
  low: "#22C55E",
  medium: "#F59E0B",
  high: "#F97316",
  critical: "#EF4444",
};

const RISK_DESC_KEYS: Record<RiskLevel, string> = {
  low: "Read-only operations, memory access",
  medium: "File writes, web requests",
  high: "Shell commands, system access",
  critical: "Destructive ops, external APIs",
};

const LANGUAGES = [
  { code: "en", label: "English" },
  { code: "zh", label: "中文" },
];

export function SettingsView() {
  const { t, i18n } = useTranslation();
  const providers = useSettingsStore((s) => s.providers);
  const agents = useSettingsStore((s) => s.agents);
  const autoApprove = useSettingsStore((s) => s.autoApprove);
  const updateProvider = useSettingsStore((s) => s.updateProvider);
  const updateAgent = useSettingsStore((s) => s.updateAgent);
  const setAutoApprove = useSettingsStore((s) => s.setAutoApprove);

  const currentLang = i18n.language.startsWith("zh") ? "zh" : "en";

  return (
    <div className="flex-1 overflow-y-auto px-6 py-4">
      <h2 className="text-lg font-semibold text-text-primary mb-6">{t("settings.title")}</h2>

      {/* Language section */}
      <section className="mb-8">
        <h3 className="text-sm text-text-muted uppercase tracking-wider font-medium mb-3">
          {t("settings.language")}
        </h3>
        <div className="flex items-center gap-2">
          {LANGUAGES.map((lang) => (
            <button
              key={lang.code}
              onClick={() => i18n.changeLanguage(lang.code)}
              className={`px-4 py-2 rounded-lg border text-sm font-medium transition-colors duration-100
                focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
                ${
                  currentLang === lang.code
                    ? "border-accent bg-accent/10 text-accent"
                    : "border-border text-text-secondary hover:text-text-primary hover:bg-muted/30"
                }`}
            >
              {lang.label}
            </button>
          ))}
        </div>
      </section>

      {/* Provider section */}
      <section className="mb-8">
        <h3 className="text-sm text-text-muted uppercase tracking-wider font-medium mb-3">
          {t("settings.providers")}
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
                  <label className="text-[12px] text-text-muted uppercase tracking-wider">
                    {t("settings.endpoint")}
                  </label>
                  <input
                    type="text"
                    value={provider.endpoint}
                    onChange={(e) => updateProvider(provider.id, { endpoint: e.target.value })}
                    placeholder="https://..."
                    className="w-full bg-card border border-border rounded px-2 py-1 text-sm text-text-primary
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
        <h3 className="text-sm text-text-muted uppercase tracking-wider font-medium mb-3">
          {t("settings.agents")}
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
                  <span className="text-[12px] text-text-muted uppercase">{agent.role}</span>
                </div>
                <div className="mt-1">
                  <select
                    value={agent.model}
                    onChange={(e) => updateAgent(agent.id, { model: e.target.value })}
                    className="bg-card border border-border rounded px-2 py-0.5 text-[13px] text-text-primary
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
        <h3 className="text-sm text-text-muted uppercase tracking-wider font-medium mb-3">
          {t("settings.approvals")}
        </h3>
        <div className="space-y-2">
          {RISK_LEVELS.map((level) => {
            const color = RISK_COLORS[level];
            const description = RISK_DESC_KEYS[level];
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
                      style={{ backgroundColor: color }}
                    />
                    <span className="text-sm text-text-primary font-medium capitalize">{level}</span>
                  </div>
                  <p className="text-[13px] text-text-muted mt-0.5">{description}</p>
                </div>
              </label>
            );
          })}
        </div>
      </section>
    </div>
  );
}
