<template>
  <aside class="utility-panel">
    <!-- Task Info section -->
    <div class="panel-section">
      <button class="section-header" @click="toggle('taskInfo')" :aria-expanded="open.taskInfo" aria-controls="section-task-info">
        <span class="section-title">Task Info</span>
        <span class="chevron" :class="{ collapsed: !open.taskInfo }">▾</span>
      </button>
      <div v-if="open.taskInfo" id="section-task-info" class="section-body">
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
      <button class="section-header" @click="toggle('agents')" :aria-expanded="open.agents" aria-controls="section-agents">
        <span class="section-title">Agents</span>
        <span class="chevron" :class="{ collapsed: !open.agents }">▾</span>
      </button>
      <div v-if="open.agents" id="section-agents" class="section-body">
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
      <button class="section-header" @click="toggle('permissions')" :aria-expanded="open.permissions" aria-controls="section-permissions">
        <span class="section-title">Permissions</span>
        <span class="chevron" :class="{ collapsed: !open.permissions }">▾</span>
      </button>
      <div v-if="open.permissions" id="section-permissions" class="section-body">
        <div class="empty-hint">No permissions requested</div>
      </div>
    </div>

    <!-- System Health section -->
    <div class="panel-section">
      <button class="section-header" @click="toggle('health')" :aria-expanded="open.health" aria-controls="section-health">
        <span class="section-title">System Health</span>
        <span class="chevron" :class="{ collapsed: !open.health }">▾</span>
      </button>
      <div v-if="open.health" id="section-health" class="section-body">
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
      <button class="section-header" @click="toggle('connection')" :aria-expanded="open.connection" aria-controls="section-connection">
        <span class="section-title">Connection</span>
        <span class="chevron" :class="{ collapsed: !open.connection }">▾</span>
      </button>
      <div v-if="open.connection" id="section-connection" class="section-body">
        <div class="info-row">
          <span class="info-label">Backend</span>
          <span
            class="conn-dot"
            :class="backendStatusClass"
          />
          <span class="conn-label">{{ backendLabel }}</span>
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
import { useDaemonStore } from '@/shared/stores/daemon'
import AgentAvatar from '@/shared/components/AgentAvatar.vue'

const HEALTH_POLL_INTERVAL_MS = 5_000

const agentStore = useAgentStore()
const daemonStore = useDaemonStore()

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

const backendStatusClass = computed(() =>
  daemonStore.status === 'connected'
    ? 'connected'
    : daemonStore.status === 'connecting'
      ? 'connecting'
      : 'disconnected'
)

const backendLabel = computed(() => {
  switch (daemonStore.status) {
    case 'connected':    return 'Connected'
    case 'connecting':   return 'Connecting\u2026'
    case 'disconnected': return 'Disconnected'
    default:             return 'Disconnected'
  }
})

const gatewayStatusClass = computed(() =>
  daemonStore.status === 'connected' && daemonStore.orchestratorConnected
    ? 'connected'
    : 'disconnected'
)

const gatewayLabel = computed(() =>
  daemonStore.status === 'connected' && daemonStore.orchestratorConnected
    ? 'Online'
    : 'Offline'
)
</script>

<style scoped>
.utility-panel {
  width: 280px;
  background: rgba(13, 13, 13, 0.78);
  backdrop-filter: var(--backdrop-md);
  -webkit-backdrop-filter: var(--backdrop-md);
  border-left: 1px solid var(--border);
  overflow-y: auto;
  padding: var(--space-lg);
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
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
  padding: var(--space-sm) var(--space-md);
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font: var(--font-weight-bold) var(--font-size-xs) var(--font-mono);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  cursor: pointer;
  text-align: left;
  transition: background var(--duration-fast);
}

.section-header:hover {
  background: var(--muted);
}

.section-title {
  color: var(--text-muted);
}

.chevron {
  font-size: var(--font-size-base);
  color: var(--text-disabled);
  transition: transform var(--duration-fast);
  display: inline-block;
}

.chevron.collapsed {
  transform: rotate(-90deg);
}

.section-body {
  padding: var(--space-sm) var(--space-md) var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  border-top: 1px solid var(--border);
}

/* Info rows */
.info-row {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font: var(--font-size-sm) var(--font-mono);
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
  font: var(--font-size-xs) var(--font-mono);
  padding: var(--space-2xs) var(--space-sm);
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
  font: var(--font-size-xs) var(--font-mono);
  color: var(--text-disabled);
  margin-top: 2px;
}

/* Agent list */
.agent-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.agent-row {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.agent-name {
  flex: 1;
  font: var(--font-size-sm) var(--font-mono);
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.agent-status {
  font: var(--font-size-xs) var(--font-mono);
  padding: var(--space-2xs) var(--space-sm);
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
  font: var(--font-size-sm) var(--font-mono);
  color: var(--text-disabled);
  text-align: center;
  padding: var(--space-xs) 0;
}

/* Connection dots */
.conn-dot {
  width: var(--space-sm);
  height: var(--space-sm);
  border-radius: 50%;
  flex-shrink: 0;
}

.conn-dot.connected   { background: var(--success); }
.conn-dot.connecting  { background: var(--warning); }
.conn-dot.disconnected { background: var(--error); }

.conn-label {
  font: var(--font-size-sm) var(--font-mono);
  color: var(--text-secondary);
}
</style>
