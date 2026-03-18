import { useState, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useCostStore } from "../../stores/costStore";

/**
 * CreditMeter — Progress bar showing DD credit balance with color transitions.
 * Green (>50%) -> Yellow (25-50%) -> Red (<25%).
 * On hover/click: popover with per-agent cost breakdown.
 */
export function CreditMeter() {
  const { t } = useTranslation();
  const balance = useCostStore((s) => s.balance);
  const limit = useCostStore((s) => s.limit);
  const sessionCost = useCostStore((s) => s.sessionCost);
  const agentCosts = useCostStore((s) => s.agentCosts);
  const [showPopover, setShowPopover] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const pct = limit > 0 ? (balance / limit) * 100 : 0;

  const barColor =
    pct > 50 ? "#22C55E" : pct > 25 ? "#F59E0B" : "#EF4444";

  useEffect(() => {
    if (!showPopover) return;
    function handleClick(e: MouseEvent) {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        setShowPopover(false);
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [showPopover]);

  const agentEntries = Object.entries(agentCosts);

  return (
    <div ref={containerRef} className="relative">
      <button
        onClick={() => setShowPopover((v) => !v)}
        className="flex items-center gap-1.5 text-sm text-text-secondary
          hover:text-text-primary transition-colors duration-100
          focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
          rounded px-1.5 py-0.5"
        aria-label={`DD Credits: ${balance} of ${limit} remaining`}
        aria-expanded={showPopover}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-warning shrink-0" aria-hidden="true">
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
        </svg>
        <div className="w-16 h-1.5 rounded-full bg-muted overflow-hidden" aria-hidden="true">
          <div className="h-full rounded-full transition-all duration-300" style={{ width: `${pct}%`, backgroundColor: barColor }} />
        </div>
        <span>{balance} / {limit} DD</span>
      </button>

      {showPopover && (
        <div
          className="absolute right-0 top-full mt-2 z-50 min-w-[200px]
            rounded-lg border border-border bg-card/95 p-3
            shadow-lg shadow-black/40"
          style={{ backdropFilter: "blur(16px)", WebkitBackdropFilter: "blur(16px)" }}
          role="dialog"
          aria-label={t("credits.sessionCostBreakdown")}
        >
          <p className="text-[12px] text-text-muted uppercase tracking-wider mb-2">{t("credits.sessionCostBreakdown")}</p>
          <div className="flex justify-between text-sm mb-1">
            <span className="text-text-secondary">{t("credits.sessionTotal")}</span>
            <span className="text-text-primary">{sessionCost} DD</span>
          </div>
          <div className="flex justify-between text-sm mb-2">
            <span className="text-text-secondary">{t("credits.balance")}</span>
            <span style={{ color: barColor }}>{balance} / {limit} DD</span>
          </div>
          {agentEntries.length > 0 && (
            <>
              <div className="h-px bg-border my-2" />
              <p className="text-[12px] text-text-muted uppercase tracking-wider mb-1.5">{t("credits.perAgent")}</p>
              {agentEntries.map(([name, cost]) => (
                <div key={name} className="flex justify-between text-sm mb-0.5">
                  <span className="text-text-secondary">{name}</span>
                  <span className="text-text-primary">{cost} DD</span>
                </div>
              ))}
            </>
          )}
        </div>
      )}
    </div>
  );
}
