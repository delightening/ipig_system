import { useState, useCallback } from 'react'

export interface ConfirmDialogState {
  open: boolean
  title: string
  description: string
  variant?: 'default' | 'destructive'
  confirmLabel?: string
  onConfirm: () => void
  onCancel: () => void
}

const INITIAL: ConfirmDialogState = {
  open: false,
  title: '',
  description: '',
  onConfirm: () => {},
  onCancel: () => {},
}

export function useConfirmDialog() {
  const [state, setState] = useState<ConfirmDialogState>(INITIAL)

  const confirm = useCallback(
    (opts: {
      title: string
      description: string
      variant?: 'default' | 'destructive'
      confirmLabel?: string
    }): Promise<boolean> =>
      new Promise((resolve) => {
        setState({
          open: true,
          ...opts,
          onConfirm: () => {
            setState(INITIAL)
            resolve(true)
          },
          onCancel: () => {
            setState(INITIAL)
            resolve(false)
          },
        })
      }),
    []
  )

  return { dialogState: state, confirm }
}
