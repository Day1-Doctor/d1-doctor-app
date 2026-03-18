import { useState, useRef, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  useEventLogStore,
  type EventCategory,
  type EventLogEntry,
} from "../../stores/eventLogStore";

const CATEGORY_COLORS: Record<EventCategory, string> = {
  agent: "#3B82F6",
  tool: "#F97316",
  task: "#22C55E",
  cost: "#F59E0B",
  system: "#878787",
};

const ALL_CATEGORIES: EventCategory[] = ["agent", "tool", "task", "cost", "system"];

function formatTime(iso: string): string {
  const d = new Date(iso);
  const h = d.getHours().toString().padStart(2, "0");
  const m = d.getMinutes().toString().padStart(2, "0");
  const s = d.getSeconds().toString().padStart(2, "0");
  return `${h}:${m}:${s}`;
}

function truncatePayload(payload: Record<string, unknown>, maxLen = 60): string {
  const str = JSON.stringify(payload);
  return str.length > maxLen ? str.slice(0, maxLen) + "..." : str;
}

function EventRow({ event }: { event: EventLogEntry }) {
  const color = CATEGORY_COLORS[event.category];
  return (
    <div className="flex items-start gap-2 px-3 py-1.5 hover:bg-muted/30 text-[13px] group">
      <span className="text-text-disabled tabular-nums shrink-0 w-[60px]">
        {formatTime(event.timestamp)}
      </span>
      <span
        className="shrink-0 px-1.5 py-0.5 rounded text-[13px] font-medium uppercase tracking-wider"
        style={{
          backgroundColor: color + "15",
          color: color,
          border: `1px solid ${color}30`,
        }}
      >
        {event.category}
      </span>
      <span className="text-text-secondary shrink-0 w-[80px] truncate" title={event.agentName}>
        {event.agentName || "--"}
      </span>
      <span className="text-text-primary truncate flex-1">{event.type}</span>
      <span className="text-text-disabled truncate max-w-[200px] hidden group-hover:inline">
        {truncatePayload(event.payload)}
      </span>
    </div>
  );
}

export function EventLog() {
  const { t } = useTranslation();
  const events = useEventLogStore((s) => s.events);
  const [categoryFilter, setCategoryFilter] = useState<EventCategory | "all">("all");
  const [agentFilter, setAgentFilter] = useState<string>("all");
  const [searchText, setSearchText] = useState("");
  const [isPaused, setIsPaused] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  const agentNames = useMemo(() => {
    const names = new Set<string>();
    for (const e of events) {
      if (e.agentName) names.add(e.agentName);
    }
    return Array.from(names).sort();
  }, [events]);

  const filtered = useMemo(() => {
    return events.filter((e) => {
      if (categoryFilter !== "all" && e.category !== categoryFilter) return false;
      if (agentFilter !== "all" && e.agentName !== agentFilter) return false;
      if (searchText) {
        const lower = searchText.toLowerCase();
        const match =
          e.type.toLowerCase().includes(lower) ||
          e.agentName.toLowerCase().includes(lower) ||
          JSON.stringify(e.payload).toLowerCase().includes(lower);
        if (!match) return false;
      }
      return true;
    });
  }, [events, categoryFilter, agentFilter, searchText]);

  const scrollToBottom = useCallback(() => {
    if (!isPaused && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [isPaused]);

  useEffect(() => {
    scrollToBottom();
  }, [filtered.length, scrollToBottom]);

  return (
    <div className="flex flex-col h-full min-h-0">
      {/* Filter bar */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-border shrink-0 flex-wrap">
        <label className="text-[12px] text-text-muted uppercase tracking-wider">{t("debug.filterByType")}</label>
        <select
          value={categoryFilter}
          onChange={(e) => setCategoryFilter(e.target.value as EventCategory | "all")}
          className="bg-card border border-border rounded px-2 py-0.5 text-[13px] text-text-primary
            focus:outline-none focus:ring-1 focus:ring-accent/50 font-mono cursor-pointer"
        >
          <option value="all">{t("common.all")}</option>
          {ALL_CATEGORIES.map((c) => (
            <option key={c} value={c}>{c}.*</option>
          ))}
        </select>

        <label className="text-[12px] text-text-muted uppercase tracking-wider ml-2">{t("debug.filterByAgent")}</label>
        <select
          value={agentFilter}
          onChange={(e) => setAgentFilter(e.target.value)}
          className="bg-card border border-border rounded px-2 py-0.5 text-[13px] text-text-primary
            focus:outline-none focus:ring-1 focus:ring-accent/50 font-mono cursor-pointer"
        >
          <option value="all">{t("common.all")}</option>
          {agentNames.map((n) => (
            <option key={n} value={n}>{n}</option>
          ))}
        </select>

        <input
          type="text"
          value={searchText}
          onChange={(e) => setSearchText(e.target.value)}
          placeholder={t("debug.search")}
          className="bg-card border border-border rounded px-2 py-0.5 text-[13px] text-text-primary
            placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent/50
            font-mono ml-2 w-[140px]"
        />

        <span className="ml-auto text-[12px] text-text-muted tabular-nums">
          {filtered.length} {t("debug.events")}
        </span>
      </div>

      {/* Event list */}
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto min-h-0"
        onMouseEnter={() => setIsPaused(true)}
        onMouseLeave={() => setIsPaused(false)}
      >
        {filtered.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <p className="text-text-disabled text-sm">{t("debug.noMatchingEvents")}</p>
          </div>
        ) : (
          filtered.map((event) => <EventRow key={event.id} event={event} />)
        )}
      </div>

      {/* Paused indicator */}
      {isPaused && (
        <div className="shrink-0 text-center py-1 bg-warning/10 text-warning text-[12px] border-t border-border">
          {t("debug.autoScrollPaused")}
        </div>
      )}
    </div>
  );
}
