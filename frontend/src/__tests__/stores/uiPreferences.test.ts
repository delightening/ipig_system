import { describe, it, expect, beforeEach } from 'vitest'
import { useUIPreferences } from '@/stores/uiPreferences'

describe('useUIPreferences', () => {
  beforeEach(() => {
    useUIPreferences.setState({ developerMode: false })
  })

  it('defaults developer mode to false', () => {
    expect(useUIPreferences.getState().developerMode).toBe(false)
  })

  it('toggles developer mode', () => {
    useUIPreferences.getState().toggleDeveloperMode()
    expect(useUIPreferences.getState().developerMode).toBe(true)

    useUIPreferences.getState().toggleDeveloperMode()
    expect(useUIPreferences.getState().developerMode).toBe(false)
  })

  it('sets developer mode directly', () => {
    useUIPreferences.getState().setDeveloperMode(true)
    expect(useUIPreferences.getState().developerMode).toBe(true)

    useUIPreferences.getState().setDeveloperMode(false)
    expect(useUIPreferences.getState().developerMode).toBe(false)
  })
})
