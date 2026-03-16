import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

interface InlineCommentDialogProps {
  open: boolean
  sectionName: string
  onClose: () => void
  onSubmit: (content: string) => void
  isSubmitting: boolean
}

export function InlineCommentDialog({
  open,
  sectionName,
  onClose,
  onSubmit,
  isSubmitting,
}: InlineCommentDialogProps) {
  const { t } = useTranslation()
  const [content, setContent] = useState('')

  const handleSubmit = () => {
    if (!content.trim()) return
    onSubmit(`[${sectionName}] ${content.trim()}`)
  }

  const handleOpenChange = (isOpen: boolean) => {
    if (!isOpen) {
      setContent('')
      onClose()
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('protocols.detail.dialogs.comment.title')}</DialogTitle>
          <DialogDescription>
            針對「{sectionName}」章節新增審查意見
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label>{t('protocols.detail.dialogs.comment.placeholder')}</Label>
            <Textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder={`請輸入對「${sectionName}」的審查意見...`}
              rows={5}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => handleOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={!content.trim() || isSubmitting}
          >
            {isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t('protocols.detail.dialogs.comment.submit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
