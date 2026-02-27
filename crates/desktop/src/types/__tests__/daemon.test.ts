import { describe, it, expect } from 'vitest'
import {
  PROTOCOL_VERSION,
  createMessage,
  type DaemonMessage,
  type ClientMessage,
} from '../daemon'

describe('daemon types', () => {
  it('PROTOCOL_VERSION is 1', () => {
    expect(PROTOCOL_VERSION).toBe(1)
  })

  it('createMessage("task.submit", ...) produces valid envelope', () => {
    const msg = createMessage('task.submit', {
      task_id: 'tsk_001',
      input: 'test task',
      context: {},
    })
    expect(msg.v).toBe(1)
    expect(msg.type).toBe('task.submit')
    expect(msg.payload.task_id).toBe('tsk_001')
    expect(msg.id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i)
    const now = Date.now()
    expect(msg.ts).toBeGreaterThan(now - 1000)
    expect(msg.ts).toBeLessThanOrEqual(now + 1)
  })

  it('createMessage("heartbeat", ...) produces heartbeat', () => {
    const msg = createMessage('heartbeat', { ping: true })
    expect(msg.type).toBe('heartbeat')
    expect(msg.payload.ping).toBe(true)
  })
})
