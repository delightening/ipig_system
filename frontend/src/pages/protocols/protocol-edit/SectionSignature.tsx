// Section Signature 元件
// 支援檔案上傳 + 手寫簽名兩種模式

import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { FileUpload } from '@/components/ui/file-upload'
import { Button } from '@/components/ui/button'
import { HandwrittenSignaturePad, type SignatureData } from '@/components/ui/handwritten-signature-pad'
import { PenLine, Upload } from 'lucide-react'
import type { SectionProps } from './types'

type SignMode = 'upload' | 'handwriting'

export function SectionSignature({ formData, setFormData, t }: SectionProps) {
  const [signMode, setSignMode] = useState<SignMode>('upload')

  const handleSignatureChange = (sigData: SignatureData | null) => {
    if (sigData) {
      // 將手寫簽名 SVG 儲存到 formData
      setFormData((prev) => ({
        ...prev,
        working_content: {
          ...prev.working_content,
          handwriting_svg: sigData.svg,
          stroke_data: sigData.strokeData,
        }
      }))
    } else {
      setFormData((prev) => ({
        ...prev,
        working_content: {
          ...prev.working_content,
          handwriting_svg: undefined,
          stroke_data: undefined,
        }
      }))
    }
  }

  return (
    <Card className="min-w-0">
      <CardHeader>
        <CardTitle>{t('aup.section10')}</CardTitle>
        <CardDescription>{t('aup.signature.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4 min-w-0">
        {/* 簽名模式切換 */}
        <div className="flex gap-2">
          <Button
            type="button"
            variant={signMode === 'upload' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setSignMode('upload')}
          >
            <Upload className="h-4 w-4 mr-1" />
            {t('aup.signature.uploadMode', '上傳簽名檔')}
          </Button>
          <Button
            type="button"
            variant={signMode === 'handwriting' ? 'default' : 'outline'}
            size="sm"
            onClick={() => setSignMode('handwriting')}
          >
            <PenLine className="h-4 w-4 mr-1" />
            {t('signature.handwriting', '手寫簽名')}
          </Button>
        </div>

        {/* 上傳模式 */}
        {signMode === 'upload' && (
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
        )}

        {/* 手寫簽名模式 */}
        {signMode === 'handwriting' && (
          <div className="space-y-2 min-w-0 max-w-full">
            <Label>{t('signature.handwriting', '手寫簽名')}</Label>
            <HandwrittenSignaturePad
              onSignatureChange={handleSignatureChange}
              height={200}
              className="max-w-full"
            />
            {formData.working_content.handwriting_svg && (
              <p className="text-sm text-green-600">
                ✓ {t('signature.signed', '已簽署')}
              </p>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
