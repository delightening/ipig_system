import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import type { ConfirmDialogState } from '@/hooks/useConfirmDialog'

interface Props {
  state: ConfirmDialogState
}

export function ConfirmDialog({ state }: Props) {
  return (
    <AlertDialog open={state.open} onOpenChange={(open) => { if (!open) state.onCancel() }}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{state.title}</AlertDialogTitle>
          <AlertDialogDescription>{state.description}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={state.onCancel}>取消</AlertDialogCancel>
          <AlertDialogAction
            onClick={state.onConfirm}
            className={state.variant === 'destructive' ? 'bg-red-600 hover:bg-red-700' : ''}
          >
            {state.confirmLabel || '確認'}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
