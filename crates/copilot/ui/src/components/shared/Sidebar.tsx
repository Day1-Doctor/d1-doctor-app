import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useViewStore, type ViewType } from "../../stores/viewStore";
import { useLayoutStore } from "../../stores/layoutStore";
import { useOfficeStore } from "../../stores/officeStore";
import { useBillingStore } from "../../stores/billingStore";

interface NavItem {
  id: string;
  labelKey: string;
  icon: React.ReactNode;
  view?: ViewType;
}

const MAX_OFFICES = 11;

const navItems: NavItem[] = [
  {
    id: "task",
    labelKey: "nav.tasks",
    view: "task",
    icon: (
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <path d="M9 11l3 3L22 4" />
        <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
      </svg>
    ),
  },
  {
    id: "debug",
    labelKey: "nav.debug",
    view: "debug",
    icon: (
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
    ),
  },
  {
    id: "chat",
    labelKey: "nav.chat",
    view: "chat",
    icon: (
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
      </svg>
    ),
  },
];

const secondaryItems: NavItem[] = [
  {
    id: "agents",
    labelKey: "nav.agents",
    icon: (
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
        <circle cx="9" cy="7" r="4" />
        <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
        <path d="M16 3.13a4 4 0 0 1 0 7.75" />
      </svg>
    ),
  },
  {
    id: "memory",
    labelKey: "nav.memory",
    icon: (
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <path d="M9.5 2A2.5 2.5 0 0 1 12 4.5v15a2.5 2.5 0 0 1-4.96.44 2.5 2.5 0 0 1-2.96-3.08 3 3 0 0 1-.34-5.58 2.5 2.5 0 0 1 1.32-4.24 2.5 2.5 0 0 1 1.98-3A2.5 2.5 0 0 1 9.5 2z" />
        <path d="M14.5 2A2.5 2.5 0 0 0 12 4.5v15a2.5 2.5 0 0 0 4.96.44 2.5 2.5 0 0 0 2.96-3.08 3 3 0 0 0 .34-5.58 2.5 2.5 0 0 0-1.32-4.24 2.5 2.5 0 0 0-1.98-3A2.5 2.5 0 0 0 14.5 2z" />
      </svg>
    ),
  },
  {
    id: "settings",
    labelKey: "nav.settings",
    view: "settings",
    icon: (
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
      </svg>
    ),
  },
];

function NavButton({
  item,
  isActive,
  collapsed,
  onClick,
}: {
  item: NavItem;
  isActive: boolean;
  collapsed: boolean;
  onClick: () => void;
}) {
  const { t } = useTranslation();
  const label = t(item.labelKey);

  return (
    <button
      onClick={onClick}
      className={`
        w-full flex items-center gap-3 px-3 py-2 rounded-md text-xs
        transition-colors duration-100 relative
        focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
        ${
          isActive
            ? "text-text-primary bg-muted"
            : "text-text-secondary hover:text-text-primary hover:bg-muted/50"
        }
        ${collapsed ? "justify-center px-0" : ""}
      `}
      aria-label={label}
      aria-current={isActive ? "page" : undefined}
      title={collapsed ? label : undefined}
      tabIndex={0}
    >
      {isActive && (
        <span
          className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-4 bg-accent rounded-r"
          aria-hidden="true"
        />
      )}
      <span className={`shrink-0 ${isActive ? "text-accent" : ""}`}>
        {item.icon}
      </span>
      {!collapsed && <span>{label}</span>}
    </button>
  );
}

/** Cowork Valley collapsible section with nested office items */
function ValleySection({
  collapsed,
}: {
  collapsed: boolean;
}) {
  const { t } = useTranslation();
  const [isExpanded, setIsExpanded] = useState(true);
  const activeView = useViewStore((s) => s.activeView);
  const setActiveView = useViewStore((s) => s.setActiveView);
  const offices = useOfficeStore((s) => s.offices);
  const addOffice = useOfficeStore((s) => s.addOffice);
  const maxOffices = useBillingStore((s) => s.maxAgents);
  const openUpgradePrompt = useBillingStore((s) => s.openUpgradePrompt);

  const isValleyActive = activeView === "valley" || activeView === "office";
  const lockedCount = Math.max(0, MAX_OFFICES - offices.length);
  // How many locked slots beyond what billing allows
  const billingLockedCount = Math.max(0, MAX_OFFICES - Math.min(maxOffices, MAX_OFFICES));

  const handleValleyClick = () => {
    setActiveView("valley");
    setIsExpanded((prev) => !prev);
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

  const valleyLabel = t("nav.valley");

  if (collapsed) {
    // When sidebar is collapsed, just show the valley icon
    return (
      <button
        onClick={() => setActiveView("valley")}
        className={`
          w-full flex items-center justify-center py-2 rounded-md text-xs
          transition-colors duration-100 relative
          focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
          ${isValleyActive ? "text-text-primary bg-muted" : "text-text-secondary hover:text-text-primary hover:bg-muted/50"}
        `}
        aria-label={valleyLabel}
        title={valleyLabel}
        tabIndex={0}
      >
        {isValleyActive && (
          <span
            className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-4 bg-accent rounded-r"
            aria-hidden="true"
          />
        )}
        <span className={`shrink-0 ${isValleyActive ? "text-accent" : ""}`}>
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <path d="M2 22L7 7l5 8 4-5 6 12H2z" />
            <path d="M17 3l2 2-2 2" />
            <circle cx="6" cy="4" r="1.5" />
          </svg>
        </span>
      </button>
    );
  }

  return (
    <div>
      {/* Cowork Valley parent item */}
      <button
        onClick={handleValleyClick}
        className={`
          w-full flex items-center gap-3 px-3 py-2 rounded-md text-xs
          transition-colors duration-100 relative
          focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
          ${isValleyActive ? "text-text-primary bg-muted" : "text-text-secondary hover:text-text-primary hover:bg-muted/50"}
        `}
        aria-label={valleyLabel}
        aria-expanded={isExpanded}
        tabIndex={0}
      >
        {isValleyActive && (
          <span
            className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-4 bg-accent rounded-r"
            aria-hidden="true"
          />
        )}
        <span className={`shrink-0 ${isValleyActive ? "text-accent" : ""}`}>
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <path d="M2 22L7 7l5 8 4-5 6 12H2z" />
            <path d="M17 3l2 2-2 2" />
            <circle cx="6" cy="4" r="1.5" />
          </svg>
        </span>
        <span className="flex-1 text-left">{valleyLabel}</span>
        {/* Expand/collapse chevron */}
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
          className={`shrink-0 transition-transform duration-150 ${isExpanded ? "rotate-180" : ""}`}
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </button>

      {/* Nested office items */}
      {isExpanded && (
        <div className="mt-0.5 space-y-0.5">
          {offices.map((office) => {
            const isOfficeActive = activeView === "office";
            const isEmpty = office.agentCount === 0;
            return (
              <button
                key={office.id}
                onClick={() => setActiveView("office")}
                className={`
                  w-full flex items-center gap-2 pl-8 pr-3 py-1.5 rounded-md text-xs
                  transition-colors duration-100 relative
                  focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
                  ${isOfficeActive ? "text-text-primary bg-muted border-l-2 border-accent" : ""}
                  ${isEmpty ? "text-text-muted hover:text-text-secondary hover:bg-muted/30" : "text-text-secondary hover:text-text-primary hover:bg-muted/50"}
                `}
                aria-label={office.name}
                tabIndex={0}
              >
                {/* Tree connector icon */}
                <span className="text-text-muted shrink-0" aria-hidden="true">├</span>
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  aria-hidden="true"
                  className={`shrink-0 ${isOfficeActive ? "text-accent" : isEmpty ? "text-text-muted" : ""}`}
                >
                  <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
                  <polyline points="9 22 9 12 15 12 15 22" />
                </svg>
                <span className="truncate">{office.name}</span>
                {isEmpty && (
                  <span className="ml-auto text-text-muted text-[10px]">empty</span>
                )}
              </button>
            );
          })}

          {/* Add office "+" button */}
          <div className="flex items-center gap-2 pl-8 pr-3 py-1">
            <button
              onClick={handleAddOffice}
              className="flex items-center justify-center w-5 h-5 rounded-full bg-accent/20 hover:bg-accent/40
                text-accent text-xs font-bold transition-colors duration-100
                focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
              aria-label="Add new office"
              title={offices.length < maxOffices ? "Add new office" : "Upgrade to add more offices"}
            >
              +
            </button>
          </div>

          {/* Locked count */}
          {lockedCount > 0 && (
            <div className="flex items-center gap-2 pl-8 pr-3 py-1 text-[10px] text-text-muted select-none">
              <span aria-hidden="true">└</span>
              <span>+ {billingLockedCount > 0 ? billingLockedCount : lockedCount} locked</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

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
      <div
        className={`flex items-center h-10 px-3 ${collapsed ? "justify-center" : "justify-end"}`}
      >
        <button
          onClick={toggleSidebar}
          className="p-1 rounded hover:bg-muted text-text-muted hover:text-text-secondary
            transition-colors duration-100 focus:outline-none focus-visible:ring-2
            focus-visible:ring-accent/50"
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          tabIndex={0}
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

      {/* Primary nav */}
      <nav className="flex-1 px-2 space-y-0.5 overflow-y-auto" aria-label="Primary">
        {/* Cowork Valley collapsible section */}
        <ValleySection collapsed={collapsed} />

        {/* Separator */}
        <div
          className="my-2 border-t border-border"
          role="separator"
          aria-hidden="true"
        />

        {/* Other primary nav items */}
        {navItems.map((item) => (
          <NavButton
            key={item.id}
            item={item}
            isActive={item.view === activeView}
            collapsed={collapsed}
            onClick={() => {
              if (item.view) {
                setActiveView(item.view);
              }
            }}
          />
        ))}

        {/* Separator */}
        <div
          className="my-2 border-t border-border"
          role="separator"
          aria-hidden="true"
        />

        {/* Secondary nav */}
        {secondaryItems.map((item) => (
          <NavButton
            key={item.id}
            item={item}
            isActive={item.view === activeView}
            collapsed={collapsed}
            onClick={() => {
              if (item.view) {
                setActiveView(item.view);
              }
            }}
          />
        ))}
      </nav>
    </aside>
  );
}
