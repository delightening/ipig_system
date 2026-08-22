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
  if (!isBlocked) return null

  return (
    <AlertDialog open>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>離開此頁面？</AlertDialogTitle>
          <AlertDialogDescription>
            您有未儲存的變更。離開後將遺失這些修改。
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={onReset}>
            繼續編輯
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={onProceed}
            className="bg-destructive hover:bg-destructive/90"
          >
            離開
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
