import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useViewStore } from "../../stores/viewStore";
import { useLayoutStore } from "../../stores/layoutStore";
import { useOfficeStore } from "../../stores/officeStore";
import { useBillingStore } from "../../stores/billingStore";
import { useApprovalStore } from "../../stores/approvalStore";

const MAX_OFFICES = 11;

// ── Icons ──────────────────────────────────────────────────────────────────

function BuildingIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <rect x="4" y="2" width="16" height="20" rx="2" ry="2" />
      <path d="M9 22V12h6v10" />
      <path d="M8 6h.01M12 6h.01M16 6h.01M8 10h.01M12 10h.01M16 10h.01" />
    </svg>
  );
}

function ChecklistIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M9 11l3 3L22 4" />
      <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
    </svg>
  );
}

function FolderIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
    </svg>
  );
}

function ChartBarIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <line x1="18" y1="20" x2="18" y2="10" />
      <line x1="12" y1="20" x2="12" y2="4" />
      <line x1="6" y1="20" x2="6" y2="14" />
      <line x1="2" y1="20" x2="22" y2="20" />
    </svg>
  );
}

function TerminalIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <polyline points="4 17 10 11 4 5" />
      <line x1="12" y1="19" x2="20" y2="19" />
    </svg>
  );
}

function SettingsGearIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}

function ChevronDownIcon({ expanded }: { expanded: boolean }) {
  return (
    <svg
      width="10"
      height="10"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      className={`shrink-0 transition-transform duration-150 ${expanded ? "rotate-180" : ""}`}
    >
      <polyline points="6 9 12 15 18 9" />
    </svg>
  );
}

// ── Collapsed icon-only button ─────────────────────────────────────────────

function CollapsedIconButton({
  icon,
  label,
  isActive,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  isActive: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`
        w-full flex items-center justify-center py-2 rounded-md
        transition-colors duration-100 relative
        focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
        ${isActive ? "text-text-primary bg-muted" : "text-text-secondary hover:text-text-primary hover:bg-muted/50"}
      `}
      aria-label={label}
      title={label}
    >
      {isActive && (
        <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-4 bg-accent rounded-r" aria-hidden="true" />
      )}
      <span className={isActive ? "text-accent" : ""}>{icon}</span>
    </button>
  );
}

// ── Section header ─────────────────────────────────────────────────────────

function SectionHeader({
  label,
  icon,
  expanded,
  isActive,
  onClick,
  badge,
}: {
  label: string;
  icon: React.ReactNode;
  expanded: boolean;
  isActive: boolean;
  onClick: () => void;
  badge?: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={`
        w-full flex items-center gap-2 px-2 py-1.5 rounded-md
        transition-colors duration-100 relative group
        focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
        ${isActive ? "text-text-primary" : "text-text-muted hover:text-text-secondary"}
      `}
      aria-expanded={expanded}
    >
      {isActive && (
        <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-4 bg-accent rounded-r" aria-hidden="true" />
      )}
      <span className={`shrink-0 ${isActive ? "text-accent" : ""}`}>{icon}</span>
      <span className="flex-1 text-left text-[10px] uppercase tracking-wider font-semibold">
        {label}
      </span>
      {badge}
      <ChevronDownIcon expanded={expanded} />
    </button>
  );
}

// ── Sub-item ───────────────────────────────────────────────────────────────

function SubItem({
  label,
  isActive,
  onClick,
  badge,
  badgeType = "count",
  accentDot = false,
  muted = false,
}: {
  label: string;
  isActive?: boolean;
  onClick?: () => void;
  badge?: string;
  badgeType?: "count" | "warning" | "muted";
  accentDot?: boolean;
  muted?: boolean;
}) {
  const badgeCls =
    badgeType === "warning"
      ? "bg-yellow-500/20 text-yellow-400 border border-yellow-500/30"
      : badgeType === "muted"
        ? "bg-muted text-text-muted"
        : "bg-muted text-text-secondary";

  return (
    <button
      onClick={onClick}
      disabled={!onClick}
      className={`
        w-full flex items-center gap-2 pl-6 pr-2 py-1 rounded-md text-[11px]
        transition-colors duration-100 relative
        focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
        ${isActive ? "text-text-primary bg-muted/60 border-l-2 border-accent" : ""}
        ${muted ? "text-text-muted cursor-default" : !isActive ? "text-text-secondary hover:text-text-primary hover:bg-muted/40" : ""}
        ${!onClick ? "cursor-default" : ""}
      `}
    >
      {accentDot && (
        <span className="w-1.5 h-1.5 rounded-full bg-accent shrink-0" aria-hidden="true" />
      )}
      <span className="flex-1 text-left truncate">{label}</span>
      {badge && (
        <span className={`text-[9px] font-medium px-1.5 py-0.5 rounded-full shrink-0 ${badgeCls}`}>
          {badge}
        </span>
      )}
    </button>
  );
}

// ── COWORK VALLEY section ──────────────────────────────────────────────────

function ValleySection({ collapsed }: { collapsed: boolean }) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(true);
  const activeView = useViewStore((s) => s.activeView);
  const setActiveView = useViewStore((s) => s.setActiveView);
  const offices = useOfficeStore((s) => s.offices);
  const addOffice = useOfficeStore((s) => s.addOffice);
  const maxOffices = useBillingStore((s) => s.maxAgents);
  const openUpgradePrompt = useBillingStore((s) => s.openUpgradePrompt);

  const isActive = activeView === "valley" || activeView === "office";
  const label = t("sidebar.coworkValley");
  const billingLockedCount = Math.max(0, MAX_OFFICES - Math.min(maxOffices, MAX_OFFICES));
  const lockedCount = Math.max(0, MAX_OFFICES - offices.length);

  if (collapsed) {
    return (
      <CollapsedIconButton
        icon={<BuildingIcon />}
        label={label}
        isActive={isActive}
        onClick={() => setActiveView("valley")}
      />
    );
  }

  const handleHeaderClick = () => {
    setActiveView("valley");
    setExpanded((prev) => !prev);
  };

  const handleAddOffice = (e: React.MouseEvent) => {
    e.stopPropagation();
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
  };

  return (
    <div>
      <SectionHeader
        label={label}
        icon={<BuildingIcon />}
        expanded={expanded}
        isActive={isActive}
        onClick={handleHeaderClick}
      />
      {expanded && (
        <div className="mt-0.5 space-y-0.5">
          {offices.map((office) => (
            <SubItem
              key={office.id}
              label={office.name}
              isActive={activeView === "office"}
              onClick={() => setActiveView("office")}
              accentDot={activeView === "office"}
            />
          ))}
          {/* Add Office button */}
          <button
            onClick={handleAddOffice}
            className="w-full flex items-center gap-2 pl-6 pr-2 py-1 rounded-md text-[11px]
              text-accent hover:text-accent hover:bg-accent/10
              transition-colors duration-100
              focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
          >
            <span className="font-bold text-sm leading-none">+</span>
            <span>{t("sidebar.addOffice")}</span>
          </button>
          {/* Locked count */}
          {lockedCount > 0 && (
            <div className="pl-6 pr-2 py-1 text-[10px] text-text-muted select-none">
              {t("sidebar.locked", { count: billingLockedCount > 0 ? billingLockedCount : lockedCount })}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// ── TASKS section ──────────────────────────────────────────────────────────

function TasksSection({ collapsed }: { collapsed: boolean }) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(true);
  const activeView = useViewStore((s) => s.activeView);
  const setActiveView = useViewStore((s) => s.setActiveView);
  const pendingApprovals = useApprovalStore((s) => s.pendingApprovals);
  const blockCount = pendingApprovals.length;

  const isActive = activeView === "task";
  const label = t("sidebar.tasks");

  if (collapsed) {
    return (
      <CollapsedIconButton
        icon={<ChecklistIcon />}
        label={label}
        isActive={isActive}
        onClick={() => setActiveView("task")}
      />
    );
  }

  const handleHeaderClick = () => {
    setActiveView("task");
    setExpanded((prev) => !prev);
  };

  return (
    <div>
      <SectionHeader
        label={label}
        icon={<ChecklistIcon />}
        expanded={expanded}
        isActive={isActive}
        onClick={handleHeaderClick}
        badge={
          blockCount > 0 ? (
            <span className="text-[9px] font-medium px-1.5 py-0.5 rounded-full bg-yellow-500/20 text-yellow-400 border border-yellow-500/30 shrink-0">
              ⚠
            </span>
          ) : undefined
        }
      />
      {expanded && (
        <div className="mt-0.5 space-y-0.5">
          <SubItem
            label={t("sidebar.taskList")}
            isActive={activeView === "task"}
            onClick={() => setActiveView("task")}
          />
          {blockCount > 0 && (
            <SubItem
              label={t("sidebar.blocks", { count: blockCount })}
              onClick={() => setActiveView("task")}
              badge={String(blockCount)}
              badgeType="warning"
            />
          )}
        </div>
      )}
    </div>
  );
}

// ── WORKSPACE section ──────────────────────────────────────────────────────

function WorkspaceSection({ collapsed }: { collapsed: boolean }) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(true);
  const activeView = useViewStore((s) => s.activeView);
  const setActiveView = useViewStore((s) => s.setActiveView);

  const isActive = activeView === "workspace";
  const label = t("sidebar.workspace");

  if (collapsed) {
    return (
      <CollapsedIconButton
        icon={<FolderIcon />}
        label={label}
        isActive={isActive}
        onClick={() => setActiveView("workspace")}
      />
    );
  }

  const handleHeaderClick = () => {
    setActiveView("workspace");
    setExpanded((prev) => !prev);
  };

  return (
    <div>
      <SectionHeader
        label={label}
        icon={<FolderIcon />}
        expanded={expanded}
        isActive={isActive}
        onClick={handleHeaderClick}
      />
      {expanded && (
        <div className="mt-0.5 space-y-0.5">
          <SubItem
            label={t("sidebar.filesArtifacts")}
            isActive={activeView === "workspace"}
            onClick={() => setActiveView("workspace")}
          />
          <SubItem
            label={t("sidebar.memory")}
            onClick={() => setActiveView("workspace")}
          />
        </div>
      )}
    </div>
  );
}

// ── METRICS section ────────────────────────────────────────────────────────

function MetricsSection({ collapsed }: { collapsed: boolean }) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(true);
  const activeView = useViewStore((s) => s.activeView);
  const setActiveView = useViewStore((s) => s.setActiveView);

  const isActive = activeView === "metrics";
  const label = t("sidebar.metrics");

  if (collapsed) {
    return (
      <CollapsedIconButton
        icon={<ChartBarIcon />}
        label={label}
        isActive={isActive}
        onClick={() => setActiveView("metrics")}
      />
    );
  }

  const handleHeaderClick = () => {
    setActiveView("metrics");
    setExpanded((prev) => !prev);
  };

  return (
    <div>
      <SectionHeader
        label={label}
        icon={<ChartBarIcon />}
        expanded={expanded}
        isActive={isActive}
        onClick={handleHeaderClick}
      />
      {expanded && (
        <div className="mt-0.5 space-y-0.5">
          <SubItem
            label={t("sidebar.usageCosts")}
            isActive={activeView === "metrics"}
            onClick={() => setActiveView("metrics")}
          />
        </div>
      )}
    </div>
  );
}

// ── DEVELOPER section ──────────────────────────────────────────────────────

function DeveloperSection({ collapsed }: { collapsed: boolean }) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(false);
  const activeView = useViewStore((s) => s.activeView);
  const setActiveView = useViewStore((s) => s.setActiveView);

  const isActive = activeView === "debug";
  const label = t("sidebar.developer");

  if (collapsed) {
    return (
      <CollapsedIconButton
        icon={<TerminalIcon />}
        label={label}
        isActive={isActive}
        onClick={() => setActiveView("debug")}
      />
    );
  }

  return (
    <div>
      <SectionHeader
        label={label}
        icon={<TerminalIcon />}
        expanded={expanded}
        isActive={isActive}
        onClick={() => setExpanded((prev) => !prev)}
      />
      {expanded && (
        <div className="mt-0.5 space-y-0.5">
          <SubItem
            label={t("sidebar.debugConsole")}
            isActive={activeView === "debug"}
            onClick={() => setActiveView("debug")}
          />
        </div>
      )}
    </div>
  );
}

// ── Main Sidebar ───────────────────────────────────────────────────────────

export function Sidebar() {
  const { t } = useTranslation();
  const activeView = useViewStore((s) => s.activeView);
  const setActiveView = useViewStore((s) => s.setActiveView);
  const collapsed = useLayoutStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useLayoutStore((s) => s.toggleSidebar);

  return (
    <aside
      className={`
        flex flex-col border-r border-border shrink-0
        backdrop-blur-xl saturate-150 bg-black/60
        transition-[width] duration-150 ease-out
        ${collapsed ? "w-14" : "w-[220px]"}
      `}
      role="navigation"
      aria-label={t("nav.office")}
    >
      {/* Collapse toggle */}
      <div className={`flex items-center h-10 px-3 ${collapsed ? "justify-center" : "justify-end"}`}>
        <button
          onClick={toggleSidebar}
          className="p-1 rounded hover:bg-muted text-text-muted hover:text-text-secondary
            transition-colors duration-100 focus:outline-none focus-visible:ring-2
            focus-visible:ring-accent/50"
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
            className={`transition-transform duration-150 ${collapsed ? "rotate-180" : ""}`}
          >
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <line x1="9" y1="3" x2="9" y2="21" />
          </svg>
        </button>
      </div>

      {/* Primary nav sections */}
      <nav className="flex-1 px-2 space-y-0.5 overflow-y-auto" aria-label="Primary">
        <ValleySection collapsed={collapsed} />
        <TasksSection collapsed={collapsed} />
        <WorkspaceSection collapsed={collapsed} />
        <MetricsSection collapsed={collapsed} />

        {/* Separator */}
        <div className="my-2 border-t border-border" role="separator" aria-hidden="true" />

        <DeveloperSection collapsed={collapsed} />
      </nav>

      {/* Settings at bottom */}
      <div className="px-2 pb-3">
        {collapsed ? (
          <CollapsedIconButton
            icon={<SettingsGearIcon />}
            label={t("nav.settings")}
            isActive={activeView === "settings"}
            onClick={() => setActiveView("settings")}
          />
        ) : (
          <button
            onClick={() => setActiveView("settings")}
            className={`
              w-full flex items-center gap-2 px-2 py-2 rounded-md text-[11px]
              transition-colors duration-100 relative
              focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
              ${activeView === "settings" ? "text-text-primary bg-muted" : "text-text-secondary hover:text-text-primary hover:bg-muted/50"}
            `}
          >
            {activeView === "settings" && (
              <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-4 bg-accent rounded-r" aria-hidden="true" />
            )}
            <span className={activeView === "settings" ? "text-accent" : ""}><SettingsGearIcon /></span>
            <span>{t("nav.settings")}</span>
          </button>
        )}
      </div>
    </aside>
  );
}
