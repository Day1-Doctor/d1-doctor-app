import React, { Suspense, lazy, memo, useState, useCallback } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useViewStore } from "./stores/viewStore";
import { useEventStream } from "./hooks/useEventStream";
import { useAgentStore } from "./stores/agentStore";
import { useAuthStore } from "./stores/authStore";
import { useSettingsStore } from "./stores/settingsStore";
import { TopBar } from "./components/shared/TopBar";
import { Sidebar } from "./components/shared/Sidebar";
import { StatusBar } from "./components/shared/StatusBar";
import { RightPanel } from "./components/shared/RightPanel";
import { AuthWall } from "./components/shared/AuthWall";
import { ValleyView } from "./views/ValleyView";
import { OfficeView } from "./views/OfficeView";
import { TaskView } from "./views/TaskView";
import {
  OnboardingWizard,
  useOnboarding,
} from "./components/shared/OnboardingWizard";
import { ApprovalDialog } from "./components/shared/ApprovalDialog";
import { UpgradePrompt } from "./components/shared/UpgradePrompt";
import { useDeepLink } from "./hooks/useDeepLink";

const DebugView = lazy(() =>
  import("./views/DebugView").then((m) => ({ default: m.DebugView })),
);
const SettingsView = lazy(() =>
  import("./views/SettingsView").then((m) => ({ default: m.SettingsView })),
);
const WorkspaceView = lazy(() =>
  import("./views/WorkspaceView").then((m) => ({ default: m.WorkspaceView })),
);
const MetricsView = lazy(() =>
  import("./views/MetricsView").then((m) => ({ default: m.MetricsView })),
);

function LazyFallback() {
  const { t } = useTranslation();
  return (
    <div className="flex-1 flex items-center justify-center">
      <span className="text-text-muted text-sm">{t("common.loading")}</span>
    </div>
  );
}

const viewComponents = {
  valley: ValleyView,
  office: OfficeView,
  task: TaskView,
  workspace: WorkspaceView,
  metrics: MetricsView,
  debug: DebugView,
  chat: ValleyView, // Chat is now in Command Center; fallback to valley
  settings: SettingsView,
} as const;

const motionVariants = {
  initial: { opacity: 0, x: 10 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -10 },
};

const reducedMotion =
  typeof window !== "undefined" &&
  window.matchMedia("(prefers-reduced-motion: reduce)").matches;

const transitionConfig = {
  duration: reducedMotion ? 0 : 0.15,
  ease: "easeOut" as const,
};

const MainContent = memo(function MainContent({
  activeView,
}: {
  activeView: keyof typeof viewComponents;
}) {
  const ActiveViewComponent = viewComponents[activeView];

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={activeView}
        variants={motionVariants}
        initial="initial"
        animate="animate"
        exit="exit"
        transition={transitionConfig}
        className="flex-1 flex"
        style={{ willChange: "opacity, transform" }}
      >
        <Suspense fallback={<LazyFallback />}>
          <ActiveViewComponent />
        </Suspense>
      </motion.div>
    </AnimatePresence>
  );
});

function App() {
  useEventStream();
  useDeepLink();
  const activeView = useViewStore((s) => s.activeView);
  const { hasOnboarded, completeOnboarding } = useOnboarding();
  const fetchAgents = useAgentStore((s) => s.fetchAgents);
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const checkStoredAuth = useAuthStore((s) => s.checkStoredAuth);
  const refreshBalance = useAuthStore((s) => s.refreshBalance);
  const [showAuthWall, setShowAuthWall] = useState(false);

  // Apply persisted zoom level on mount
  const zoomLevel = useSettingsStore((s) => s.zoomLevel);
  React.useEffect(() => {
    document.documentElement.style.fontSize = `${zoomLevel}%`;
  }, [zoomLevel]);

  // Check for stored auth on mount
  React.useEffect(() => {
    checkStoredAuth();
  }, [checkStoredAuth]);

  React.useEffect(() => {
    const timer = setTimeout(() => fetchAgents(), 500);
    return () => clearTimeout(timer);
  }, [fetchAgents]);

  // Auto-dismiss auth wall when authenticated (e.g. via deep link callback)
  React.useEffect(() => {
    if (isAuthenticated && showAuthWall) {
      setShowAuthWall(false);
      fetchAgents();
    }
  }, [isAuthenticated, showAuthWall, fetchAgents]);

  // Refresh balance periodically when authenticated
  React.useEffect(() => {
    if (!isAuthenticated) return;
    const interval = setInterval(() => refreshBalance(), 60_000);
    return () => clearInterval(interval);
  }, [isAuthenticated, refreshBalance]);

  const handleAuthRequired = useCallback(() => {
    if (!isAuthenticated) {
      setShowAuthWall(true);
    }
  }, [isAuthenticated]);

  const handleAuthenticated = useCallback(() => {
    setShowAuthWall(false);
    fetchAgents();
  }, [fetchAgents]);

  return (
    <div className="h-screen flex flex-col bg-background font-mono overflow-hidden">
      {!hasOnboarded && <OnboardingWizard onComplete={completeOnboarding} />}
      {/* Auth wall modal */}
      {showAuthWall && <AuthWall onAuthenticated={handleAuthenticated} />}

      {/* Approval dialog for risky agent tool calls (PRD 10.4) */}
      <ApprovalDialog />

      {/* Upgrade prompt when tier limit exceeded */}
      <UpgradePrompt />

      <TopBar onAuthRequired={handleAuthRequired} />

      <div className="flex flex-1 min-h-0">
        <Sidebar />
        <main className="flex-1 flex min-w-0 relative" role="main">
          <MainContent activeView={activeView} />
        </main>
        <RightPanel onAuthRequired={handleAuthRequired} />
      </div>

      <StatusBar />
    </div>
  );
}

export default App;
