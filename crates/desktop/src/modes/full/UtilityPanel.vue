<template>
  <aside class="utility-panel">
    <!-- Task Info section -->
    <div class="panel-section">
      <button class="section-header" @click="toggle('taskInfo')">
        <span class="section-title">Task Info</span>
        <span class="chevron" :class="{ collapsed: !open.taskInfo }">▾</span>
      </button>
      <div v-if="open.taskInfo" class="section-body">
        <div class="info-row">
          <span class="info-label">Task</span>
          <span class="info-value">—</span>
        </div>
        <div class="info-row">
          <span class="info-label">Status</span>
          <span class="status-badge idle">Idle</span>
        </div>
        <div class="info-row">
          <span class="info-label">Duration</span>
          <span class="info-value">—</span>
        </div>
        <div class="info-row">
          <span class="info-label">Credits est.</span>
          <span class="info-value">—</span>
        </div>
      </div>
    </div>

    <!-- Agents section -->
    <div class="panel-section">
      <button class="section-header" @click="toggle('agents')">
        <span class="section-title">Agents</span>
        <span class="chevron" :class="{ collapsed: !open.agents }">▾</span>
      </button>
      <div v-if="open.agents" class="section-body">
        <div v-if="agentStore.activeAgents.length === 0" class="empty-hint">
          No active agents
        </div>
        <div v-else class="agent-list">
          <div v-for="agent in agentStore.activeAgents" :key="agent" class="agent-row">
            <AgentAvatar :agent="agent" :active="true" />
            <span class="agent-name">{{ agent }}</span>
            <span class="agent-status active">Active</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Permissions section -->
    <div class="panel-section">
      <button class="section-header" @click="toggle('permissions')">
        <span class="section-title">Permissions</span>
        <span class="chevron" :class="{ collapsed: !open.permissions }">▾</span>
      </button>
      <div v-if="open.permissions" class="section-body">
        <div class="empty-hint">No permissions requested</div>
      </div>
    </div>

    <!-- System Health section -->
    <div class="panel-section">
      <button class="section-header" @click="toggle('health')">
        <span class="section-title">System Health</span>
        <span class="chevron" :class="{ collapsed: !open.health }">▾</span>
      </button>
      <div v-if="open.health" class="section-body">
        <div class="info-row">
          <span class="info-label">OS</span>
          <span class="info-value">macOS</span>
        </div>
        <div class="info-row">
          <span class="info-label">Daemon</span>
          <span class="status-badge" :class="daemonStatus === 'Running' ? 'ok' : 'error'">
            {{ daemonStatus }}
          </span>
        </div>
        <div class="health-refresh">
          Last check: {{ lastHealthCheck }}
        </div>
      </div>
    </div>

    <!-- Connection section -->
    <div class="panel-section">
      <button class="section-header" @click="toggle('connection')">
        <span class="section-title">Connection</span>
        <span class="chevron" :class="{ collapsed: !open.connection }">▾</span>
      </button>
      <div v-if="open.connection" class="section-body">
        <div class="info-row">
          <span class="info-label">Backend</span>
          <span
            class="conn-dot"
            :class="backendStatusClass"
          />
          <span class="conn-label">{{ capitalize(agentStore.connectionStatus) }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Gateway</span>
          <span
            class="conn-dot"
            :class="gatewayStatusClass"
          />
          <span class="conn-label">{{ gatewayLabel }}</span>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useAgentStore } from '@/shared/stores/agent'
import AgentAvatar from '@/shared/components/AgentAvatar.vue'

const HEALTH_POLL_INTERVAL_MS = 5_000

const agentStore = useAgentStore()

// Collapse state — all expanded by default, in memory only
const open = ref({
  taskInfo:    true,
  agents:      true,
  permissions: true,
  health:      true,
  connection:  true,
})

function toggle(key: keyof typeof open.value): void {
  open.value[key] = !open.value[key]
}

// System health polling
const daemonStatus = ref<'Running' | 'Stopped'>('Running')
const lastHealthCheck = ref('—')
let healthTimer: ReturnType<typeof setInterval> | null = null

function refreshHealth(): void {
  // Stub: in a real impl, call Tauri to check daemon state
  daemonStatus.value = 'Running'
  lastHealthCheck.value = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

onMounted(() => {
  refreshHealth()
  healthTimer = setInterval(refreshHealth, HEALTH_POLL_INTERVAL_MS)
})

onUnmounted(() => {
  if (healthTimer !== null) {
    clearInterval(healthTimer)
    healthTimer = null
  }
})

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1)
}

const backendStatusClass = computed(() =>
  agentStore.connectionStatus === 'connected'
    ? 'connected'
    : agentStore.connectionStatus === 'connecting'
      ? 'connecting'
      : 'disconnected'
)

const gatewayStatusClass = computed(() =>
  agentStore.connectionStatus === 'connected' ? 'connected' : 'disconnected'
)

const gatewayLabel = computed(() =>
  agentStore.connectionStatus === 'connected' ? 'Online' : 'Offline'
)
</script>

<style scoped>
.utility-panel {
  width: 280px;
  background: rgba(13, 13, 13, 0.78);
  backdrop-filter: blur(30px) saturate(140%);
  -webkit-backdrop-filter: blur(30px) saturate(140%);
  border-left: 1px solid var(--border);
  overflow-y: auto;
  padding: 16px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.utility-panel::-webkit-scrollbar {
  width: 3px;
}

.utility-panel::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}

/* Section */
.panel-section {
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--card);
}

.section-header {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font: 700 10px var(--font-mono);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  cursor: pointer;
  text-align: left;
  transition: background 0.12s;
}

.section-header:hover {
  background: var(--muted);
}

.section-title {
  color: var(--text-muted);
}

.chevron {
  font-size: 12px;
  color: var(--text-disabled);
  transition: transform 0.2s;
  display: inline-block;
}

.chevron.collapsed {
  transform: rotate(-90deg);
}

.section-body {
  padding: 8px 12px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  border-top: 1px solid var(--border);
}

/* Info rows */
.info-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font: 11px var(--font-mono);
}

.info-label {
  color: var(--text-disabled);
  min-width: 72px;
}

.info-value {
  color: var(--text-secondary);
}

/* Status badges */
.status-badge {
  font: 10px var(--font-mono);
  padding: 2px 7px;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
}

.status-badge.idle {
  background: var(--muted);
  color: var(--text-disabled);
  border-color: var(--border);
}

.status-badge.ok {
  background: var(--success-soft);
  color: var(--success);
  border-color: var(--success-border);
}

.status-badge.error {
  background: var(--error-soft);
  color: var(--error);
  border-color: var(--error-border);
}

/* Health refresh hint */
.health-refresh {
  font: 10px var(--font-mono);
  color: var(--text-disabled);
  margin-top: 2px;
}

/* Agent list */
.agent-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.agent-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.agent-name {
  flex: 1;
  font: 11px var(--font-mono);
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.agent-status {
  font: 10px var(--font-mono);
  padding: 2px 7px;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
}

.agent-status.active {
  background: var(--accent-soft);
  color: var(--accent);
  border-color: var(--accent-border);
}

/* Empty hint */
.empty-hint {
  font: 11px var(--font-mono);
  color: var(--text-disabled);
  text-align: center;
  padding: 4px 0;
}

/* Connection dots */
.conn-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.conn-dot.connected   { background: var(--success); }
.conn-dot.connecting  { background: var(--warning); }
.conn-dot.disconnected { background: var(--error); }

.conn-label {
  font: 11px var(--font-mono);
  color: var(--text-secondary);
}
</style>
