// Section Signature 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { FileUpload } from '@/components/ui/file-upload'
import type { SectionProps } from './types'

export function SectionSignature({ formData, setFormData, t }: SectionProps) {

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('aup.section10')}</CardTitle>
        <CardDescription>{t('aup.signature.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label>{t('aup.signature.label')}</Label>
          <FileUpload
            value={formData.working_content.signature || []}
            onChange={(signature) => {
              setFormData((prev) => ({
                ...prev,
                working_content: {
                  ...prev.working_content,
                  signature
                }
              }))
            }}
            accept="image/*,.png,.jpg,.jpeg,.gif,.bmp"
            placeholder={t('aup.signature.placeholder')}
            maxSize={5}
            maxFiles={5}
            showPreview={true}
            hint={t('aup.signature.hint')}
          />
        </div>
      </CardContent>
    </Card>
  )
}
