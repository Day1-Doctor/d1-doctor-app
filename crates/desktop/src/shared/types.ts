// CONTRACT FILE — read-only after Phase 1 creates it.
export type UIMode = 'full' | 'copilot' | 'ninja'

export interface Message {
  id: string
  role: 'agent' | 'user'
  content: string
  timestamp: number
}

export interface Step {
  id: string
  label: string
  state: 'pending' | 'active' | 'done' | 'error'
  index: number
}

export interface Plan {
  steps: Step[]
  approved: boolean | null
}

export type AgentEventType =
  | 'plan_proposed'
  | 'step_started'
  | 'step_completed'
  | 'step_failed'
  | 'agent_message'
  | 'result_ready'
  | 'credits_updated'
  | 'permission_requested'

export interface AgentEvent {
  type: AgentEventType
  payload: Record<string, unknown>
}

export interface CreditInfo {
  current: number
  max: number
}

export interface Config {
  ui_mode: UIMode
  last_copilot_x: number
  last_copilot_y: number
  theme: 'dark'
}
