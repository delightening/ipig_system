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

interface UnsavedChangesDialogProps {
  isBlocked: boolean
  onProceed: () => void
  onReset: () => void
}

export function UnsavedChangesDialog({ isBlocked, onProceed, onReset }: UnsavedChangesDialogProps) {
  const { t } = useTranslation()
  if (!isBlocked) return null

  return (
    <AlertDialog open>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t('unsavedChanges.title')}</AlertDialogTitle>
          <AlertDialogDescription>
            {t('unsavedChanges.description')}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={onReset}>
            {t('unsavedChanges.continueEditing')}
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={onProceed}
            className="bg-destructive hover:bg-destructive/90"
          >
            {t('unsavedChanges.leave')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
