import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation()
  return (
    <AlertDialog open={state.open} onOpenChange={(open) => { if (!open) state.onCancel() }}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{state.title}</AlertDialogTitle>
          <AlertDialogDescription>{state.description}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={state.onCancel}>{t('common.cancel')}</AlertDialogCancel>
          <AlertDialogAction
            onClick={state.onConfirm}
            className={state.variant === 'destructive' ? 'bg-red-600 hover:bg-red-700' : ''}
          >
            {state.confirmLabel || t('common.confirm')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
