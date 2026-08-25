import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, X, Send } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

/**
 * 「無意見」時送出的固定內容。
 *
 * 必須與 migration 005 回填時比對的字串一字不差——回填是 `content = '無意見'`
 * 的精準比對，這裡改字（哪怕只是多一個標點）都會讓新舊資料在任何以內容為準的
 * 統計裡分成兩堆。真正的判別依據是 `comment_type`，這個字串只是給人看的呈現。
 */
const NO_OBJECTION_CONTENT = '無意見'

// 這個面板同一畫面只會出現一份（由 showCommentPanel 控制），所以固定 id 就夠，
// 不需要 useId。改成可同時開多份時要換掉，否則 htmlFor 會指到第一份。
const SECTION_SELECT_ID = 'review-comment-section'
const CONTENT_TEXTAREA_ID = 'review-comment-content'

interface ReviewCommentPanelProps {
  onClose: () => void
  /**
   * 送出一則意見。回傳 Promise 時，面板會等它 resolve 才清空欄位——
   * reject 就保留使用者的輸入。回傳 void 的呼叫端維持舊行為（送出即清空）。
   */
  onSubmit: (
    content: string,
    commentType: 'COMMENT' | 'NO_OBJECTION'
  ) => void | Promise<unknown>
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

  /**
   * ⚠️ 只有在送出**成功**之後才清空欄位。
   *
   * onSubmit 底下是 React Query 的 mutation。若在呼叫後立刻清空，一旦請求失敗
   * （網路斷線、後端 4xx），使用者剛打完的意見就沒了，「無意見」的勾選也被還原，
   * 得整個重來一次——而畫面上只會看到一個 toast 說失敗。
   *
   * 失敗時什麼都不動，讓他們直接按第二次就好。錯誤訊息由 mutation 的 onError
   * 以 toast 呈現，這裡不重複處理，所以 catch 是刻意留空的。
   */
  const handleSubmit = async () => {
    try {
      if (noObjection) {
        // 「無意見」不掛章節前綴：它是對整份計畫書表態，不是針對某一節。
        await onSubmit(NO_OBJECTION_CONTENT, 'NO_OBJECTION')
        setContent('')
        setNoObjection(false)
        return
      }
      if (!content.trim()) return
      const prefix = selectedSection ? `[${selectedSection}] ` : ''
      await onSubmit(`${prefix}${content.trim()}`, 'COMMENT')
      setContent('')
    } catch {
      // 保留使用者的輸入與已選型別，讓他們可以直接重試
    }
  }

  return (
    <div className="bg-background border rounded-lg shadow-xs flex flex-col">
      <div className="flex items-center justify-between px-4 py-3 border-b">
        <h3 className="font-semibold text-sm">{t('protocols.detail.dialogs.comment.panel.title')}</h3>
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
            disabled={isSubmitting}
            onChange={(e) => setNoObjection(e.target.checked)}
          />
          <span className="text-sm">
            <span className="font-medium">{t('protocols.detail.dialogs.comment.panel.noObjectionLabel')}</span>
            <span className="block text-xs text-muted-foreground mt-0.5">
              {t('protocols.detail.dialogs.comment.panel.noObjectionHint')}
            </span>
          </span>
        </label>

        <div className={`space-y-4 ${noObjection ? 'opacity-50 pointer-events-none' : ''}`}>
          <div className="space-y-2">
            {/* htmlFor / id 成對：沒有這組關聯，螢幕閱讀器讀到的是一個沒有名稱的控制項 */}
            <Label htmlFor={SECTION_SELECT_ID} className="text-xs">
              {t('protocols.detail.dialogs.comment.panel.targetSection')}
            </Label>
            <select
              id={SECTION_SELECT_ID}
              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
              value={selectedSection}
              onChange={(e) => setSelectedSection(e.target.value)}
              disabled={noObjection}
            >
              <option value="">{t('protocols.detail.dialogs.comment.panel.noSection')}</option>
              {sectionOptions.map((name) => (
                <option key={name} value={name}>{name}</option>
              ))}
            </select>
          </div>

          <div className="space-y-2">
            <Label htmlFor={CONTENT_TEXTAREA_ID} className="text-xs">
              {t('protocols.detail.dialogs.comment.panel.contentLabel')}
            </Label>
            <Textarea
              id={CONTENT_TEXTAREA_ID}
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder={t('protocols.detail.dialogs.comment.panel.placeholder')}
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
          {noObjection
            ? t('protocols.detail.dialogs.comment.panel.noObjectionSubmit')
            : t('protocols.detail.dialogs.comment.submit')}
        </Button>
      </div>
    </div>
  )
}
