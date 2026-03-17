import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, X, Send } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

interface ReviewCommentPanelProps {
  onClose: () => void
  onSubmit: (content: string) => void
  isSubmitting: boolean
  currentSection?: string
  sectionOptions: string[]
}

export function ReviewCommentPanel({
  onClose,
  onSubmit,
  isSubmitting,
  currentSection,
  sectionOptions,
}: ReviewCommentPanelProps) {
  const { t } = useTranslation()
  const [content, setContent] = useState('')
  const [selectedSection, setSelectedSection] = useState('')

  useEffect(() => {
    if (currentSection) {
      setSelectedSection(currentSection)
    }
  }, [currentSection])

  const handleSubmit = () => {
    if (!content.trim()) return
    const prefix = selectedSection ? `[${selectedSection}] ` : ''
    onSubmit(`${prefix}${content.trim()}`)
    setContent('')
  }

  return (
    <div className="bg-background border rounded-lg shadow-sm flex flex-col">
      <div className="flex items-center justify-between px-4 py-3 border-b">
        <h3 className="font-semibold text-sm">審查意見</h3>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X className="h-4 w-4" />
        </Button>
      </div>

      <div className="p-4 space-y-4">
        <div className="space-y-2">
          <Label className="text-xs">針對章節</Label>
          <select
            className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            value={selectedSection}
            onChange={(e) => setSelectedSection(e.target.value)}
          >
            <option value="">（不指定章節）</option>
            {sectionOptions.map((name) => (
              <option key={name} value={name}>{name}</option>
            ))}
          </select>
        </div>

        <div className="space-y-2">
          <Label className="text-xs">意見內容</Label>
          <Textarea
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="請輸入審查意見..."
            rows={6}
            className="resize-y"
          />
        </div>

        <Button
          className="w-full"
          size="sm"
          onClick={handleSubmit}
          disabled={!content.trim() || isSubmitting}
        >
          {isSubmitting ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Send className="mr-2 h-4 w-4" />
          )}
          {t('protocols.detail.dialogs.comment.submit')}
        </Button>
      </div>
    </div>
  )
}
