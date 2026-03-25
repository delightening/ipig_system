// Section Guidelines 元件
// 第 5 節：相關規範及參考文獻
// 包含：5.1 法源依據、5.2 資料庫搜尋紀錄（A-L）、5.3 引用文獻列表

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import type { SectionProps } from './types'

// A-E 有關鍵字欄位，K/L 有備註欄位
const DB_CODES_WITH_KEYWORDS = ['A', 'B', 'C', 'D', 'E']
const DB_CODES_WITH_NOTE = ['K', 'L']

export function SectionGuidelines({ formData, updateWorkingContent, setFormData: _setFormData, t, isIACUCStaff: _isIACUCStaff }: SectionProps) {
  // 確保 databases 資料存在（向下相容舊資料）
  const databases = formData.working_content.guidelines.databases || []

  const handleDatabaseToggle = (code: string, checked: boolean) => {
    const newDbs = databases.map(db =>
      db.code === code ? { ...db, checked } : db
    )
    updateWorkingContent('guidelines', 'databases', newDbs)
  }

  const handleDatabaseField = (code: string, field: 'keywords' | 'note', value: string) => {
    const newDbs = databases.map(db =>
      db.code === code ? { ...db, [field]: value } : db
    )
    updateWorkingContent('guidelines', 'databases', newDbs)
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('aup.section5')}</CardTitle>
        <CardDescription>{t('aup.guidelines.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* 5.1 法源依據 */}
        <div className="space-y-2">
          <Label>{t('aup.guidelines.contentLabel')} *</Label>
          <Textarea
            value={formData.working_content.guidelines.content}
            onChange={(e) => updateWorkingContent('guidelines', 'content', e.target.value)}
            placeholder={t('aup.guidelines.contentPlaceholder')}
            rows={5}
          />
        </div>

        {/* 5.2 資料庫搜尋紀錄 */}
        <div className="space-y-4 border p-4 rounded-md">
          <h3 className="font-semibold">{t('aup.guidelines.databasesTitle')}</h3>

          <div className="space-y-3">
            {databases.map((db) => {
              const hasKeywords = DB_CODES_WITH_KEYWORDS.includes(db.code)
              const hasNote = DB_CODES_WITH_NOTE.includes(db.code)

              return (
                <div key={db.code} className="space-y-2">
                  <div className={`flex items-start gap-3 ${hasKeywords ? '' : ''}`}>
                    <Checkbox
                      id={`db-${db.code}`}
                      checked={db.checked}
                      onCheckedChange={(checked) => handleDatabaseToggle(db.code, !!checked)}
                      className="mt-0.5"
                    />
                    <div className="flex-1">
                      <label
                        htmlFor={`db-${db.code}`}
                        className="text-sm cursor-pointer select-none leading-relaxed"
                      >
                        {db.code}. {t(`aup.guidelines.databases.${db.code}`)}
                      </label>

                      {/* A-E: 關鍵字行內輸入 */}
                      {hasKeywords && db.checked && (
                        <div className="flex items-center gap-2 mt-2 ml-0">
                          <Label className="text-xs text-muted-foreground whitespace-nowrap">
                            {t('aup.guidelines.keywordsLabel')}:
                          </Label>
                          <Input
                            value={db.keywords || ''}
                            onChange={(e) => handleDatabaseField(db.code, 'keywords', e.target.value)}
                            placeholder={t('aup.guidelines.keywordsLabel')}
                            className="h-8 text-sm"
                          />
                        </div>
                      )}

                      {/* K, L: 備註輸入 */}
                      {hasNote && db.checked && (
                        <div className="mt-2 ml-0">
                          <Textarea
                            value={db.note || ''}
                            onChange={(e) => handleDatabaseField(db.code, 'note', e.target.value)}
                            placeholder={t('aup.guidelines.noteLabel')}
                            rows={2}
                            className="text-sm"
                          />
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        </div>

        {/* 5.3 引用文獻列表 */}
        <div className="space-y-4 border p-4 rounded-md">
          <div className="flex justify-between items-center">
            <h3 className="font-semibold">{t('aup.guidelines.referencesTitle')}</h3>
            <Button
              variant="outline"
              onClick={() => {
                const newRefs = [...formData.working_content.guidelines.references, { citation: '', url: '' }]
                updateWorkingContent('guidelines', 'references', newRefs)
              }}
            >
              {t('aup.guidelines.addReference')}
            </Button>
          </div>
          {formData.working_content.guidelines.references.map((ref, index) => (
            <div key={index} className="grid w-full gap-2 relative">
              <div className="flex gap-2 items-start">
                <span className="text-sm text-muted-foreground mt-2 min-w-[24px]">{index + 1}.</span>
                <div className="grid gap-2 flex-1">
                  <Input
                    placeholder={t('aup.guidelines.citationPlaceholder')}
                    value={ref.citation}
                    onChange={(e) => {
                      const newRefs = [...formData.working_content.guidelines.references]
                      newRefs[index].citation = e.target.value
                      updateWorkingContent('guidelines', 'references', newRefs)
                    }}
                  />
                  <Input
                    placeholder={t('aup.guidelines.urlPlaceholder')}
                    value={ref.url || ''}
                    onChange={(e) => {
                      const newRefs = [...formData.working_content.guidelines.references]
                      newRefs[index].url = e.target.value
                      updateWorkingContent('guidelines', 'references', newRefs)
                    }}
                  />
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 text-destructive mt-1"
                  aria-label="刪除"
                  onClick={() => {
                    const newRefs = [...formData.working_content.guidelines.references]
                    newRefs.splice(index, 1)
                    updateWorkingContent('guidelines', 'references', newRefs)
                  }}
                >
                  X
                </Button>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
