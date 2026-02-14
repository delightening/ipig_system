// Section Attachments 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { FileUpload, FileInfo } from '@/components/ui/file-upload'
import type { SectionProps } from './types'

export function SectionAttachments({ formData, updateWorkingContent, setFormData, t, isIACUCStaff }: SectionProps) {

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('aup.section9')}</CardTitle>
        <CardDescription>{t('aup.attachments.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label>{t('aup.attachments.label')}</Label>
          <FileUpload
            value={formData.working_content.attachments || []}
            onChange={(attachments) => {
              setFormData((prev) => ({
                ...prev,
                working_content: {
                  ...prev.working_content,
                  attachments
                }
              }))
            }}
            accept="application/pdf,.pdf"
            placeholder={t('aup.attachments.placeholder')}
            maxSize={20}
            maxFiles={10}
            showPreview={false}
            hint={t('aup.attachments.hint')}
          />
        </div>
      </CardContent>
    </Card>
  )
}
