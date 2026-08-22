import { describe, it, expect } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useDialogSet } from '@/hooks/useDialogSet'

describe('useDialogSet', () => {
  it('initializes all dialogs as closed', () => {
    const { result } = renderHook(() =>
      useDialogSet(['create', 'edit', 'delete'] as const)
    )
    expect(result.current.isOpen('create')).toBe(false)
    expect(result.current.isOpen('edit')).toBe(false)
    expect(result.current.isOpen('delete')).toBe(false)
  })

  it('opens a specific dialog', () => {
    const { result } = renderHook(() =>
      useDialogSet(['create', 'edit'] as const)
    )
    act(() => { result.current.open('create') })
    expect(result.current.isOpen('create')).toBe(true)
    expect(result.current.isOpen('edit')).toBe(false)
  })

  it('closes a specific dialog', () => {
    const { result } = renderHook(() =>
      useDialogSet(['create', 'edit'] as const)
    )
    act(() => { result.current.open('create') })
    act(() => { result.current.close('create') })
    expect(result.current.isOpen('create')).toBe(false)
  })

  it('setOpen returns a setter function', () => {
    const { result } = renderHook(() =>
      useDialogSet(['modal'] as const)
    )
    const setter = result.current.setOpen('modal')
    act(() => { setter(true) })
    expect(result.current.isOpen('modal')).toBe(true)

    act(() => { setter(false) })
    expect(result.current.isOpen('modal')).toBe(false)
  })

  it('can open multiple dialogs independently', () => {
    const { result } = renderHook(() =>
      useDialogSet(['a', 'b', 'c'] as const)
    )
    act(() => { result.current.open('a') })
    act(() => { result.current.open('c') })
    expect(result.current.isOpen('a')).toBe(true)
    expect(result.current.isOpen('b')).toBe(false)
    expect(result.current.isOpen('c')).toBe(true)
  })
})
