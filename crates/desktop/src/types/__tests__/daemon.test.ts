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
    expect(typeof msg.id).toBe('string')
    expect(typeof msg.ts).toBe('number')
  })

  it('createMessage("heartbeat", ...) produces heartbeat', () => {
    const msg = createMessage('heartbeat', { ping: true })
    expect(msg.type).toBe('heartbeat')
    expect(msg.payload.ping).toBe(true)
  })
})
