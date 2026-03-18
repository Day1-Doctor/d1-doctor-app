import { useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useViewStore } from "../stores/viewStore";
import { useBillingStore } from "../stores/billingStore";
import { useOfficeStore } from "../stores/officeStore";
import { useCanvasSize } from "../hooks/useCanvasSize";
import { ValleyCanvas } from "../components/valley/ValleyCanvas";

const TIER_MAX_OFFICES: Record<string, number> = {
  free_man: 1,
  mini_shop: 3,
  rocket_inc: 8,
};

const TIER_LABELS: Record<string, string> = {
  free_man: "Free",
  mini_shop: "Mini",
  rocket_inc: "Rocket",
};

/**
 * ValleyView — Cowork Campus parcel grid showing 11 office slots.
 *
 * Title: "COWORK CAMPUS" (large monospace)
 * Tier badge beside title
 * Bottom: "N/M offices active" count
 */
export function ValleyView() {
  const { t } = useTranslation();
  const containerRef = useRef<HTMLDivElement>(null);
  const { width, height } = useCanvasSize(containerRef);

  const setActiveView = useViewStore((s) => s.setActiveView);
  const openUpgradePrompt = useBillingStore((s) => s.openUpgradePrompt);
  const tier = useBillingStore((s) => s.tier);
  const offices = useOfficeStore((s) => s.offices);
  const addOffice = useOfficeStore((s) => s.addOffice);

  const maxOffices = TIER_MAX_OFFICES[tier] ?? 1;
  const activeCount = offices.length;

  const handleParcelClick = useCallback(
    (parcelIndex: number, state: "running" | "empty" | "locked") => {
      if (state === "running") {
        setActiveView("office");
      } else if (state === "empty") {
        // If user has unlocked slots available, create a new office; otherwise show upgrade
        if (offices.length < maxOffices) {
          addOffice({
            id: `office-${Date.now()}`,
            name: `Office #${offices.length + 1}`,
            agentCount: 0,
            skillCount: 0,
            fileCount: 0,
            taskProgress: 0,
          });
        } else {
          openUpgradePrompt(t("campus.upgradePlan"));
        }
      } else {
        openUpgradePrompt(t("campus.upgradePlan"));
      }
      void parcelIndex;
    },
    [setActiveView, openUpgradePrompt, addOffice, offices.length, maxOffices, t],
  );

  const handleParcelDoubleClick = useCallback((_parcelIndex: number) => {
    // double-click on running parcel opens inline rename — handled inside ValleyCanvas
  }, []);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        flex: 1,
        minHeight: 0,
        overflow: "hidden",
        background: "#050508",
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 10,
          padding: "10px 16px 6px",
          borderBottom: "1px solid #1A1A1A",
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
            fontWeight: "bold",
            fontSize: 15,
            letterSpacing: 2,
            color: "#E5E5E5",
            textTransform: "uppercase",
          }}
        >
          {t("campus.title")}
        </span>

        {/* Tier badge */}
        <span
          style={{
            fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
            fontSize: 9,
            fontWeight: "bold",
            letterSpacing: 1,
            color: "#F97316",
            background: "rgba(249,115,22,0.12)",
            border: "1px solid rgba(249,115,22,0.3)",
            borderRadius: 4,
            padding: "1px 6px",
            textTransform: "uppercase",
          }}
        >
          {TIER_LABELS[tier] ?? tier}
        </span>
      </div>

      {/* Canvas area */}
      <div
        ref={containerRef}
        style={{ flex: 1, position: "relative", minHeight: 0, overflow: "hidden" }}
      >
        {width > 0 && height > 0 && (
          <ValleyCanvas
            width={width}
            height={height}
            onParcelClick={handleParcelClick}
            onParcelDoubleClick={handleParcelDoubleClick}
          />
        )}
      </div>

      {/* Footer: active count */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          padding: "6px 16px",
          borderTop: "1px solid #1A1A1A",
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontFamily: 'ui-monospace, "SF Mono", Menlo, monospace',
            fontSize: 10,
            color: "#666666",
            letterSpacing: 0.5,
          }}
        >
          {t("campus.activeCount", { active: activeCount, max: maxOffices })}
        </span>
      </div>
    </div>
  );
}
