import { AnimatePresence, motion } from "framer-motion";
import { useViewStore } from "./stores/viewStore";
import { useEventStream } from "./hooks/useEventStream";
import { TopBar } from "./components/shared/TopBar";
import { Sidebar } from "./components/shared/Sidebar";
import { StatusBar } from "./components/shared/StatusBar";
import { RightPanel } from "./components/shared/RightPanel";
import { OfficeView } from "./views/OfficeView";
import { TaskView } from "./views/TaskView";
import { DebugView } from "./views/DebugView";
import { ChatView } from "./views/ChatView";

const viewComponents = {
  office: OfficeView,
  task: TaskView,
  debug: DebugView,
  chat: ChatView,
} as const;

const motionVariants = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
};

const reducedMotion =
  typeof window !== "undefined" &&
  window.matchMedia("(prefers-reduced-motion: reduce)").matches;

const transitionDuration = reducedMotion ? 0 : 0.15;

function App() {
  useEventStream(); // Connect to Station Runtime Event Bus
  const activeView = useViewStore((s) => s.activeView);
  const ActiveViewComponent = viewComponents[activeView];

  return (
    <div className="h-screen flex flex-col bg-background font-mono overflow-hidden">
      <TopBar />

      <div className="flex flex-1 min-h-0">
        <Sidebar />

        {/* Main Canvas */}
        <main className="flex-1 flex min-w-0 relative" role="main">
          <AnimatePresence mode="wait">
            <motion.div
              key={activeView}
              variants={motionVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              transition={{ duration: transitionDuration }}
              className="flex-1 flex"
            >
              <ActiveViewComponent />
            </motion.div>
          </AnimatePresence>
        </main>

        <RightPanel />
      </div>

      <StatusBar />
    </div>
  );
}

export default App;
