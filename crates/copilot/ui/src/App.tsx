import React, { Suspense, lazy, memo } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useViewStore } from "./stores/viewStore";
import { useEventStream } from "./hooks/useEventStream";
import { useAgentStore } from "./stores/agentStore";
import { TopBar } from "./components/shared/TopBar";
import { Sidebar } from "./components/shared/Sidebar";
import { StatusBar } from "./components/shared/StatusBar";
import { RightPanel } from "./components/shared/RightPanel";
import { OfficeView } from "./views/OfficeView";
import { TaskView } from "./views/TaskView";
import { ChatView } from "./views/ChatView";
import {
  OnboardingWizard,
  useOnboarding,
} from "./components/shared/OnboardingWizard";

// D1D-240: Lazy load heavy views
const DebugView = lazy(() =>
  import("./views/DebugView").then((m) => ({ default: m.DebugView })),
);
const SettingsView = lazy(() =>
  import("./views/SettingsView").then((m) => ({ default: m.SettingsView })),
);

function LazyFallback() {
  return (
    <div className="flex-1 flex items-center justify-center">
      <span className="text-text-muted text-xs">Loading...</span>
    </div>
  );
}

const viewComponents = {
  office: OfficeView,
  task: TaskView,
  debug: DebugView,
  chat: ChatView,
  settings: SettingsView,
} as const;

// D1D-239: Smooth Framer Motion transitions with fade + slide
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

// D1D-240: Memoize the main content area to avoid unnecessary re-renders
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
  useEventStream(); // Connect to Station Runtime Event Bus
  const activeView = useViewStore((s) => s.activeView);
  const { hasOnboarded, completeOnboarding } = useOnboarding();
  const fetchAgents = useAgentStore((s) => s.fetchAgents);

  // Fetch real agent data from Tauri runtime on mount
  React.useEffect(() => {
    // Small delay to let the runtime initialize
    const timer = setTimeout(() => fetchAgents(), 500);
    return () => clearTimeout(timer);
  }, [fetchAgents]);

  return (
    <div className="h-screen flex flex-col bg-background font-mono overflow-hidden">
      {/* D1D-255: Onboarding wizard on first launch */}
      {!hasOnboarded && <OnboardingWizard onComplete={completeOnboarding} />}

      <TopBar />

      <div className="flex flex-1 min-h-0">
        <Sidebar />

        {/* Main Canvas */}
        <main className="flex-1 flex min-w-0 relative" role="main">
          <MainContent activeView={activeView} />
        </main>

        <RightPanel />
      </div>

      <StatusBar />
    </div>
  );
}

export default App;
