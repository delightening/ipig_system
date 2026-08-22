// Section Basic 元件（研究資料）
// title / 期程 / study_no / PI 留本檔；GLP / 類型 / 資金 / 委託單位 / 試驗機構
// 抽至 ResearchBasicFields 與匯入頁共用（R64-5c C2）。

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { SectionProps } from './types'
import { ResearchBasicFields } from './ResearchBasicFields'

/** SD（計劃負責人）欄位呈現模式：執秘/admin 下拉指派 / staff 本人自動帶入唯讀 / 客戶隱藏 */
export type SdFieldMode = 'select' | 'self' | 'hidden'

interface SectionBasicProps extends SectionProps {
  /** C3：編輯頁鎖定匯入計劃的研究資料（整段灰階唯讀） */
  disabled?: boolean
  /** C2/D5：匯入頁由 PiSelector 提供 PI，隱藏 PI 段避免重複 */
  hidePi?: boolean
  /** SD 欄位模式（由上層依角色計算） */
  sdMode?: SdFieldMode
  /** 目前 SD 使用者 id（select 模式） */
  sdUserId?: string
  /** SD 變更 callback（select 模式） */
  onSdChange?: (userId: string) => void
  /** SD 候選清單（select 模式，僅 EXPERIMENT_STAFF） */
  sdOptions?: { id: string; label: string }[]
  /** SD 本人顯示名（self 模式唯讀顯示） */
  sdSelfLabel?: string
  /** GLP 鎖：已指派 SD 的 GLP 計劃編輯時禁止變更 */
  sdLocked?: boolean
  /** SD 候選清單載入中 */
  sdLoading?: boolean
}

export function SectionBasic({
  formData,
  updateWorkingContent,
  setFormData,
  t,
  isIACUCStaff,
  isNew,
  formVersion,
  disabled,
  hidePi,
  sdMode = 'hidden',
  sdUserId = '',
  onSdChange,
  sdOptions = [],
  sdSelfLabel = '',
  sdLocked = false,
  sdLoading = false,
}: SectionBasicProps) {
  const basic = formData.working_content.basic

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('aup.section1')}</CardTitle>
        <CardDescription>{t('aup.basic.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <fieldset disabled={disabled} className="space-y-6 border-0 p-0 m-0 disabled:opacity-70">
          {/* Title */}
          <div className="space-y-2">
            <Label htmlFor="title">{t('aup.basic.studyTitle')} *</Label>
            <Input
              id="title"
              value={formData.title}
              onChange={(e) => setFormData((prev) => ({ ...prev, title: e.target.value }))}
              placeholder={t('aup.basic.studyTitlePlaceholder')}
            />
          </div>

          {/* IDs and Dates */}
          <div className={`grid gap-4 ${isNew || !isIACUCStaff ? 'md:grid-cols-1' : 'md:grid-cols-2'}`}>
            {!isNew && isIACUCStaff && (
              <div className="space-y-2">
                <Label htmlFor="apply_study_number">{t('aup.basic.studyNo')}</Label>
                <Input
                  id="apply_study_number"
                  value={basic.apply_study_number || ''}
                  onChange={(e) => updateWorkingContent('basic', 'apply_study_number', e.target.value)}
                  placeholder={t('aup.basic.studyNoPlaceholder')}
                />
              </div>
            )}
            <div className="space-y-2">
              <Label>{t('aup.basic.expectedPeriod')} *</Label>
              <div className="flex gap-2 items-center">
                <Input
                  type="date"
                  value={formData.start_date || ''}
                  onChange={(e) => setFormData((prev) => ({ ...prev, start_date: e.target.value }))}
                  required
                />
                <span className="self-center">{t('aup.basic.to')}</span>
                <Input
                  type="date"
                  value={formData.end_date || ''}
                  onChange={(e) => setFormData((prev) => ({ ...prev, end_date: e.target.value }))}
                  required
                />
              </div>
            </div>
          </div>

          {/* PI */}
          {!hidePi && (
            <>
              <div className="h-px bg-border my-4" />
              <div className="space-y-4">
                <h3 className="font-semibold">{t('aup.basic.pi')}</h3>
                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-2">
                    <Label>{t('aup.basic.name')} *</Label>
                    <Input
                      value={basic.pi.name}
                      onChange={(e) => updateWorkingContent('basic', 'pi.name', e.target.value)}
                      placeholder={t('aup.basic.piNamePlaceholder')}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.basic.position')}</Label>
                    <Input value={basic.pi.position || ''} onChange={(e) => updateWorkingContent('basic', 'pi.position', e.target.value)} />
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.basic.email')} *</Label>
                    <Input value={basic.pi.email} onChange={(e) => updateWorkingContent('basic', 'pi.email', e.target.value)} />
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.basic.phone')} *</Label>
                    <div className="flex gap-2">
                      <Input
                        className="flex-1"
                        value={basic.pi.phone}
                        onChange={(e) => updateWorkingContent('basic', 'pi.phone', e.target.value)}
                      />
                      <div className="flex items-center gap-2 shrink-0">
                        <span className="text-muted-foreground">#</span>
                        <Input
                          className="w-24"
                          placeholder={t('aup.basic.piExtension')}
                          value={basic.pi.phone_ext || ''}
                          onChange={(e) => updateWorkingContent('basic', 'pi.phone_ext', e.target.value)}
                        />
                      </div>
                    </div>
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.basic.address')} *</Label>
                    <Input value={basic.pi.address} onChange={(e) => updateWorkingContent('basic', 'pi.address', e.target.value)} />
                  </div>
                </div>
              </div>
            </>
          )}

          {/* SD（計劃負責人）：由執行秘書/admin 指定內部 EXPERIMENT_STAFF，不由客戶填寫。
              select=執秘/admin 下拉指派；self=staff 本人自動帶入唯讀；hidden=客戶/PI 不顯示。 */}
          {sdMode !== 'hidden' && (
            <>
              <div className="h-px bg-border my-4" />
              <div className="space-y-2">
                <h3 className="font-semibold">{t('aup.basic.sd')}</h3>
                {sdMode === 'select' ? (
                  <div className="space-y-2 md:max-w-md">
                    <Label>{t('aup.basic.sdName')}</Label>
                    <Select value={sdUserId} onValueChange={(v) => onSdChange?.(v)} disabled={sdLoading || sdLocked}>
                      <SelectTrigger>
                        <SelectValue placeholder={sdLoading ? t('common.loading') : t('aup.basic.sdSelectPlaceholder')} />
                      </SelectTrigger>
                      <SelectContent>
                        {sdOptions.map((o) => (
                          <SelectItem key={o.id} value={o.id}>{o.label}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <p className="text-sm text-muted-foreground">
                      {sdLocked ? t('aup.basic.sdGlpLocked') : t('aup.basic.sdHint')}
                    </p>
                  </div>
                ) : (
                  <div className="space-y-2 md:max-w-md">
                    <Label>{t('aup.basic.sdName')}</Label>
                    <Input value={sdSelfLabel} readOnly disabled />
                    {/* 新增時才是「自動帶入本人」；編輯既有計畫 SD 由執秘指定（可能未指派或為他人） */}
                    <p className="text-sm text-muted-foreground">
                      {isNew ? t('aup.basic.sdSelfHint') : t('aup.basic.sdHint')}
                    </p>
                  </div>
                )}
              </div>
            </>
          )}

          <div className="h-px bg-border my-4" />
        </fieldset>

        <ResearchBasicFields
          basic={basic}
          onUpdate={(path, value) => updateWorkingContent('basic', path, value)}
          disabled={disabled}
          formVersion={formVersion}
        />
      </CardContent>
    </Card>
  )
}
