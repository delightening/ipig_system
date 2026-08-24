import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, X, Send } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

/** 「無意見」時送出的固定內容——與 migration 005 回填既有 12 筆時比對的字串一致。 */
const NO_OBJECTION_CONTENT = '無意見'

interface ReviewCommentPanelProps {
  onClose: () => void
  onSubmit: (content: string, commentType: 'COMMENT' | 'NO_OBJECTION') => void
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
  const [noObjection, setNoObjection] = useState(false)

  useEffect(() => {
    if (currentSection) {
      setSelectedSection(currentSection)
    }
  }, [currentSection])

  const handleSubmit = () => {
    if (noObjection) {
      // 「無意見」不掛章節前綴：它是對整份計畫書表態，不是針對某一節。
      onSubmit(NO_OBJECTION_CONTENT, 'NO_OBJECTION')
      setContent('')
      setNoObjection(false)
      return
    }
    if (!content.trim()) return
    const prefix = selectedSection ? `[${selectedSection}] ` : ''
    onSubmit(`${prefix}${content.trim()}`, 'COMMENT')
    setContent('')
  }

  return (
    <div className="bg-background border rounded-lg shadow-xs flex flex-col">
      <div className="flex items-center justify-between px-4 py-3 border-b">
        <h3 className="font-semibold text-sm">審查意見</h3>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X className="h-4 w-4" />
        </Button>
      </div>

      <div className="p-4 space-y-4">
        {/*
          「無意見」是一個選項，不是要打進自由文字欄的一句話。
          審查委員沒有意見時仍必須發表一則意見才能讓計畫進入核准
          （後端 status.rs 的閘門：每位被指派的委員都要有 top-level 意見），
          所以這裡送出的仍是一筆真實的意見，只是型別為 NO_OBJECTION、
          申請人不需要回覆。
        */}
        <label className="flex items-start gap-2 rounded-md border border-input p-3 cursor-pointer hover:bg-muted/50">
          <input
            type="checkbox"
            className="mt-0.5"
            checked={noObjection}
            onChange={(e) => setNoObjection(e.target.checked)}
          />
          <span className="text-sm">
            <span className="font-medium">無意見</span>
            <span className="block text-xs text-muted-foreground mt-0.5">
              對本計畫書沒有需要修改之處。送出後仍計為已完成審查，申請人不需回覆。
            </span>
          </span>
        </label>

        <div className={`space-y-4 ${noObjection ? 'opacity-50 pointer-events-none' : ''}`}>
          <div className="space-y-2">
            <Label className="text-xs">針對章節</Label>
            <select
              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
              value={selectedSection}
              onChange={(e) => setSelectedSection(e.target.value)}
              disabled={noObjection}
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
              disabled={noObjection}
            />
          </div>
        </div>

        <Button
          className="w-full"
          size="sm"
          onClick={handleSubmit}
          disabled={(!noObjection && !content.trim()) || isSubmitting}
        >
          {isSubmitting ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Send className="mr-2 h-4 w-4" />
          )}
          {noObjection ? '送出「無意見」' : t('protocols.detail.dialogs.comment.submit')}
        </Button>
      </div>
    </div>
  )
}
