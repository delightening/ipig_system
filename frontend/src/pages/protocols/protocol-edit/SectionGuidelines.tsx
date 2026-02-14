// Section Guidelines 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import type { SectionProps } from './types'

export function SectionGuidelines({ formData, updateWorkingContent, setFormData, t, isIACUCStaff }: SectionProps) {

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('aup.section5')}</CardTitle>
        <CardDescription>{t('aup.guidelines.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-2">
          <Label>{t('aup.guidelines.contentLabel')} *</Label>
          <Textarea
            value={formData.working_content.guidelines.content}
            onChange={(e) => updateWorkingContent('guidelines', 'content', e.target.value)}
            placeholder={t('aup.guidelines.contentPlaceholder')}
            rows={5}
          />
        </div>
        <div className="space-y-4 border p-4 rounded-md">
          <div className="flex justify-between items-center">
            <h3 className="font-semibold">{t('aup.guidelines.referencesTitle')}</h3>
            <Button
              variant="outline"
              size="sm"
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
                  className="h-8 w-8 text-red-500 mt-1"
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
