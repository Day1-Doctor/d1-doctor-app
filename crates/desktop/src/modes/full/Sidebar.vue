<template>
  <aside class="sidebar">
    <!-- Logo section -->
    <div class="sidebar-logo">
      <div class="logo-circle">D1</div>
      <div class="logo-text">
        <span class="logo-name">Day 1 Doctor</span>
        <span class="logo-version">v2.4.0</span>
      </div>
    </div>

    <!-- Nav section -->
    <nav class="sidebar-nav">
      <button
        v-for="item in navItems"
        :key="item.id"
        class="nav-item"
        :class="{ active: activeNav === item.id }"
        @click="activeNav = item.id"
      >
        <span class="nav-icon">{{ item.icon }}</span>
        <span class="nav-label">{{ item.label }}</span>
      </button>
    </nav>

    <!-- Recent Tasks section -->
    <div class="sidebar-section">
      <div class="section-title">Recent Tasks</div>
      <div class="task-list">
        <div v-for="task in recentTasks" :key="task.id" class="task-item">
          <span class="task-dot" :class="task.status" />
          <span class="task-name">{{ task.name }}</span>
          <span class="task-time">{{ task.ago }}</span>
        </div>
      </div>
    </div>

    <div class="sidebar-spacer" />

    <!-- Credits section -->
    <div class="sidebar-credits">
      <CreditBar
        :credits="agentStore.credits.current"
        :max="agentStore.credits.max"
        variant="full"
        @buy="onBuyCredits"
      />
    </div>

    <!-- User section -->
    <div class="sidebar-user">
      <div class="user-avatar">U</div>
      <div class="user-info">
        <span class="user-name">User</span>
        <span class="user-email">user@example.com</span>
      </div>
      <button class="sign-out-btn" title="Sign Out">↪</button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useAgentStore } from '@/shared/stores/agent'
import CreditBar from '@/shared/components/CreditBar.vue'

interface NavItem {
  id: string
  icon: string
  label: string
}

interface RecentTask {
  id: string
  name: string
  status: 'done' | 'active' | 'pending'
  ago: string
}

const agentStore = useAgentStore()

const activeNav = ref<string>('chat')

const navItems: NavItem[] = [
  { id: 'chat',      icon: '💬', label: 'Chat'      },
  { id: 'tasks',     icon: '✅', label: 'Tasks'     },
  { id: 'knowledge', icon: '📚', label: 'Knowledge' },
  { id: 'settings',  icon: '⚙️', label: 'Settings'  },
]

const recentTasks: RecentTask[] = [
  { id: '1', name: 'Fix login bug',       status: 'done',    ago: '2m ago'  },
  { id: '2', name: 'Review PR #142',      status: 'active',  ago: '15m ago' },
  { id: '3', name: 'Update dependencies', status: 'pending', ago: '1h ago'  },
]

function onBuyCredits(): void {
  // Placeholder — Phase 4 will wire this up
}
</script>

<style scoped>
.sidebar {
  width: 260px;
  background: rgba(10, 10, 10, 0.78);
  backdrop-filter: blur(30px) saturate(140%);
  -webkit-backdrop-filter: blur(30px) saturate(140%);
  border-right: 1px solid var(--border);
  overflow-y: auto;
  padding: 16px 0;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

/* Logo */
.sidebar-logo {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 16px 16px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 8px;
}

.logo-circle {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  background: var(--accent);
  color: #000;
  font: 700 13px/34px var(--font-mono);
  text-align: center;
  flex-shrink: 0;
  user-select: none;
}

.logo-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.logo-name {
  font: 600 12px var(--font-mono);
  color: var(--text-primary);
}

.logo-version {
  font: 10px var(--font-mono);
  color: var(--accent);
  background: var(--accent-soft);
  border: 1px solid var(--accent-border);
  border-radius: var(--radius-sm);
  padding: 1px 5px;
  align-self: flex-start;
}

/* Nav */
.sidebar-nav {
  display: flex;
  flex-direction: column;
  padding: 4px 0;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 16px;
  background: transparent;
  border: none;
  border-left: 2px solid transparent;
  color: var(--text-secondary);
  font: 12px var(--font-mono);
  cursor: pointer;
  text-align: left;
  transition: background 0.12s, color 0.12s;
  width: 100%;
}

.nav-item:hover:not(.active) {
  background: var(--muted);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--accent-soft);
  border-left-color: var(--accent);
  color: var(--accent);
}

.nav-icon {
  font-size: 14px;
  width: 18px;
  text-align: center;
  flex-shrink: 0;
}

.nav-label {
  flex: 1;
}

/* Recent Tasks */
.sidebar-section {
  padding: 12px 16px 8px;
}

.section-title {
  font: 700 10px var(--font-mono);
  color: var(--text-disabled);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  margin-bottom: 8px;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font: 11px var(--font-mono);
  color: var(--text-secondary);
}

.task-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.task-dot.done    { background: var(--success); }
.task-dot.active  { background: var(--accent); }
.task-dot.pending { background: var(--text-disabled); }

.task-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-time {
  color: var(--text-disabled);
  font-size: 10px;
  white-space: nowrap;
}

/* Spacer */
.sidebar-spacer {
  flex: 1;
}

/* Credits */
.sidebar-credits {
  padding: 12px 16px;
  border-top: 1px solid var(--border);
}

/* User */
.sidebar-user {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px 0;
  border-top: 1px solid var(--border);
}

.user-avatar {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background: var(--muted);
  border: 1.5px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  font: 700 11px var(--font-mono);
  color: var(--text-secondary);
  flex-shrink: 0;
  user-select: none;
}

.user-info {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.user-name {
  font: 600 11px var(--font-mono);
  color: var(--text-primary);
}

.user-email {
  font: 10px var(--font-mono);
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sign-out-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 14px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  transition: color 0.15s, background 0.15s;
  flex-shrink: 0;
}

.sign-out-btn:hover {
  color: var(--error);
  background: var(--error-soft);
}
</style>
