import { useState } from "react";
import { useTranslation } from "react-i18next";

const STORAGE_KEY = "d1-copilot-has-onboarded";

const AGENTS = [
  { name: "Dr. Bob", role: "Orchestrator", descKey: "Coordinates the team and plans tasks" },
  { name: "Scout", role: "Researcher", descKey: "Searches the web and gathers information" },
  { name: "Sage", role: "Analyst", descKey: "Analyzes data and synthesizes findings" },
  { name: "Quill", role: "Writer", descKey: "Writes reports, docs, and deliverables" },
  { name: "Pixel", role: "Coder", descKey: "Writes and tests code" },
  { name: "Atlas", role: "Operator", descKey: "Manages files, deploys, and system ops" },
];

const ROLE_COLORS: Record<string, string> = {
  Orchestrator: "#F97316",
  Researcher: "#3B82F6",
  Analyst: "#8B5CF6",
  Writer: "#10B981",
  Coder: "#EC4899",
  Operator: "#F59E0B",
};

interface OnboardingWizardProps {
  onComplete: () => void;
}

export function useOnboarding() {
  const [hasOnboarded, setHasOnboarded] = useState(() => {
    return localStorage.getItem(STORAGE_KEY) === "true";
  });

  const completeOnboarding = () => {
    localStorage.setItem(STORAGE_KEY, "true");
    setHasOnboarded(true);
  };

  return { hasOnboarded, completeOnboarding };
}

export function OnboardingWizard({ onComplete }: OnboardingWizardProps) {
  const { t } = useTranslation();
  const [step, setStep] = useState(0);

  const totalSteps = 3;

  function handleNext() {
    if (step < totalSteps - 1) {
      setStep(step + 1);
    } else {
      onComplete();
    }
  }

  function handleBack() {
    if (step > 0) setStep(step - 1);
  }

  function handleSkip() {
    onComplete();
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm">
      <div
        className="w-[480px] max-h-[90vh] rounded-xl border border-border overflow-hidden"
        style={{ backgroundColor: "#0D0D0D" }}
      >
        {/* Step indicator */}
        <div className="flex items-center justify-center gap-2 py-4">
          {Array.from({ length: totalSteps }).map((_, i) => (
            <div
              key={i}
              className={`w-2 h-2 rounded-full transition-colors duration-200 ${
                i === step ? "bg-accent" : i < step ? "bg-accent/50" : "bg-border"
              }`}
            />
          ))}
        </div>

        {/* Step content */}
        <div className="px-8 pb-6">
          {step === 0 && <StepWelcome />}
          {step === 1 && <StepMeetTheTeam />}
          {step === 2 && <StepTryATask />}
        </div>

        {/* Navigation buttons */}
        <div className="flex items-center justify-between px-8 pb-6">
          <button
            onClick={handleBack}
            disabled={step === 0}
            className="px-4 py-1.5 rounded text-sm text-text-secondary hover:text-text-primary
              disabled:opacity-30 disabled:cursor-not-allowed transition-colors duration-100
              focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
          >
            {t("onboarding.back")}
          </button>
          <div className="flex items-center gap-2">
            <button
              onClick={handleSkip}
              className="px-4 py-1.5 rounded text-sm text-text-muted hover:text-text-secondary
                transition-colors duration-100 focus:outline-none focus-visible:ring-2
                focus-visible:ring-accent/50"
            >
              {t("onboarding.skip")}
            </button>
            <button
              onClick={handleNext}
              className="px-4 py-1.5 rounded text-sm bg-accent hover:bg-accent-hover text-background
                font-medium transition-colors duration-100 focus:outline-none focus-visible:ring-2
                focus-visible:ring-accent/50"
            >
              {step === totalSteps - 1 ? t("onboarding.done") : t("onboarding.next")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function StepWelcome() {
  const { t } = useTranslation();
  return (
    <div className="text-center">
      <div className="text-accent text-2xl font-bold mb-2">{t("onboarding.welcome")}</div>
      <p className="text-text-secondary text-sm mb-4">
        {t("onboarding.welcomeDesc")}
      </p>
      <div className="w-12 h-0.5 bg-accent/30 rounded-full mx-auto mb-4" />
      <p className="text-text-muted text-sm leading-relaxed">
        {t("onboarding.welcomeBody")}
      </p>
    </div>
  );
}

function StepMeetTheTeam() {
  const { t } = useTranslation();
  return (
    <div>
      <h3 className="text-sm text-text-primary font-medium mb-3 text-center">
        {t("onboarding.meetTeam")}
      </h3>
      <div className="grid grid-cols-2 gap-2">
        {AGENTS.map((agent) => {
          const color = ROLE_COLORS[agent.role] ?? "#6B7280";
          return (
            <div
              key={agent.name}
              className="rounded-lg border border-border p-2.5"
              style={{ backgroundColor: "rgba(15, 15, 15, 0.6)" }}
            >
              <div className="flex items-center gap-2 mb-1">
                <span
                  className="inline-block w-2.5 h-2.5 rounded-full"
                  style={{ backgroundColor: color }}
                />
                <span className="text-sm text-text-primary font-medium">{agent.name}</span>
              </div>
              <span
                className="inline-flex items-center px-1.5 py-0.5 rounded text-[13px]
                  font-medium uppercase tracking-wider mb-1"
                style={{
                  backgroundColor: color + "15",
                  color: color,
                  border: `1px solid ${color}30`,
                }}
              >
                {agent.role}
              </span>
              <p className="text-[12px] text-text-muted leading-snug">{agent.descKey}</p>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function StepTryATask() {
  const { t } = useTranslation();
  const suggestions = t("onboarding.tryTaskSuggestions", { returnObjects: true }) as string[];

  return (
    <div className="text-center">
      <h3 className="text-sm text-text-primary font-medium mb-3">{t("onboarding.tryTask")}</h3>
      <p className="text-text-muted text-sm mb-4 leading-relaxed">
        {t("onboarding.tryTaskDesc")}
      </p>
      <div className="space-y-2 text-left">
        {suggestions.map((suggestion, i) => (
          <div
            key={i}
            className="rounded-lg border border-border px-3 py-2 text-sm text-text-secondary"
            style={{ backgroundColor: "rgba(15, 15, 15, 0.6)" }}
          >
            &ldquo;{suggestion}&rdquo;
          </div>
        ))}
      </div>
    </div>
  );
}
