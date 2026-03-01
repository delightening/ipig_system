import { describe, it, expect } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'

describe('useConfirmDialog', () => {
  it('starts with closed state', () => {
    const { result } = renderHook(() => useConfirmDialog())
    expect(result.current.dialogState.open).toBe(false)
    expect(result.current.dialogState.title).toBe('')
  })

  it('opens dialog when confirm is called', async () => {
    const { result } = renderHook(() => useConfirmDialog())

    // Don't await - it returns a promise that resolves on user action
    let resolved = false
    act(() => {
      result.current.confirm({
        title: 'Delete?',
        description: 'Are you sure?',
        variant: 'destructive',
      }).then(() => { resolved = true })
    })

    expect(result.current.dialogState.open).toBe(true)
    expect(result.current.dialogState.title).toBe('Delete?')
    expect(result.current.dialogState.description).toBe('Are you sure?')
    expect(result.current.dialogState.variant).toBe('destructive')
    expect(resolved).toBe(false)
  })

  it('resolves true on confirm', async () => {
    const { result } = renderHook(() => useConfirmDialog())

    let resolvedValue: boolean | undefined
    act(() => {
      result.current.confirm({
        title: 'Test',
        description: 'Confirm?',
      }).then(v => { resolvedValue = v })
    })

    // Simulate user clicking confirm
    act(() => {
      result.current.dialogState.onConfirm()
    })

    // Wait for promise resolution
    await vi.waitFor(() => {
      expect(resolvedValue).toBe(true)
    })
    expect(result.current.dialogState.open).toBe(false)
  })

  it('resolves false on cancel', async () => {
    const { result } = renderHook(() => useConfirmDialog())

    let resolvedValue: boolean | undefined
    act(() => {
      result.current.confirm({
        title: 'Test',
        description: 'Cancel?',
      }).then(v => { resolvedValue = v })
    })

    // Simulate user clicking cancel
    act(() => {
      result.current.dialogState.onCancel()
    })

    await vi.waitFor(() => {
      expect(resolvedValue).toBe(false)
    })
    expect(result.current.dialogState.open).toBe(false)
  })
})
