import { AnimalSacrifice } from '@/lib/api'
import { sanitizeSvg } from '@/lib/sanitize'
import { HandwrittenSignaturePad } from '@/components/ui/handwritten-signature-pad'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { FileUpload } from '@/components/ui/file-upload'
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from '@/components/ui/dialog'
import { Loader2, PenLine, CheckCircle2 } from 'lucide-react'
import { useSacrificeForm } from './hooks/useSacrificeForm'

const SAMPLING_OPTIONS = [
  { value: '心', label: '心' }, { value: '肝', label: '肝' },
  { value: '脾', label: '脾' }, { value: '肺', label: '肺' },
  { value: '腎', label: '腎' }, { value: '眼', label: '眼' },
  { value: '耳', label: '耳' }, { value: '舌', label: '舌' },
  { value: '腦', label: '腦' }, { value: '骨組織', label: '骨組織' },
  { value: '脂肪', label: '脂肪' }, { value: '肌肉', label: '肌肉' },
  { value: '皮膚', label: '皮膚' }, { value: '其他', label: '其他' },
]

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  animalId: string
  earTag: string
  sacrifice?: AnimalSacrifice
}

export function SacrificeFormDialog({ open, onOpenChange, animalId, earTag, sacrifice }: Props) {
  const sf = useSacrificeForm({ open, animalId, sacrifice, onOpenChange })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>{sf.isEdit ? '編輯犧牲/採樣紀錄' : '新增犧牲/採樣紀錄'}</DialogTitle>
          <DialogDescription>耳號：{earTag}</DialogDescription>
        </DialogHeader>

        <form onSubmit={sf.handleSubmit} className="space-y-6">
          <div className="space-y-2">
            <Label htmlFor="sacrifice_date">犧牲日期 *</Label>
            <Input id="sacrifice_date" type="date" value={sf.formData.sacrifice_date}
              onChange={(e) => sf.setFormData({ ...sf.formData, sacrifice_date: e.target.value })} required />
          </div>

          <div className="space-y-4">
            <Label>犧牲方式</Label>
            <div className="space-y-2">
              <Label htmlFor="zoletil_dose" className="text-sm font-normal">Zoletil-50 (ml)</Label>
              <Input id="zoletil_dose" type="text" value={sf.formData.zoletil_dose}
                onChange={(e) => sf.setFormData({ ...sf.formData, zoletil_dose: e.target.value })} placeholder="請輸入劑量" />
            </div>
            <div className="space-y-2">
              <div className="flex flex-wrap gap-4">
                <Checkbox label="220V電擊" checked={sf.formData.method_electrocution}
                  onCheckedChange={(checked) => sf.setFormData({ ...sf.formData, method_electrocution: checked })} />
                <Checkbox label="放血" checked={sf.formData.method_bloodletting}
                  onCheckedChange={(checked) => sf.setFormData({ ...sf.formData, method_bloodletting: checked })} />
                <Checkbox label="其他" checked={sf.formData.method_other_enabled}
                  onCheckedChange={(checked) => sf.setFormData({
                    ...sf.formData, method_other_enabled: checked,
                    method_other: checked ? (sf.formData.method_other || '') : ''
                  })} />
              </div>
              {sf.formData.method_other_enabled && (
                <Input type="text" value={sf.formData.method_other}
                  onChange={(e) => sf.setFormData({ ...sf.formData, method_other: e.target.value })}
                  placeholder="請輸入其他方式" className="mt-2" />
              )}
            </div>
          </div>

          <div className="space-y-4">
            <Label>採樣部位</Label>
            <div className="grid grid-cols-4 gap-3">
              {SAMPLING_OPTIONS.map((option) => (
                <Checkbox key={option.value} label={option.label}
                  checked={sf.formData.sampling.includes(option.value)}
                  onCheckedChange={(checked) => sf.handleSamplingChange(option.value, checked)} />
              ))}
            </div>
            {sf.hasOtherSampling && (
              <div className="mt-2">
                <Input type="text" value={sf.formData.sampling_other}
                  onChange={(e) => sf.setFormData({ ...sf.formData, sampling_other: e.target.value })}
                  placeholder="請輸入其他採樣部位說明" />
              </div>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="blood_volume_ml">採樣血液 (ml)</Label>
            <Input id="blood_volume_ml" type="number" step="0.1" value={sf.formData.blood_volume_ml}
              onChange={(e) => sf.setFormData({ ...sf.formData, blood_volume_ml: e.target.value })}
              placeholder="請輸入血液採樣量" />
          </div>

          <div className="space-y-2">
            <Checkbox label="確定犧牲" checked={sf.formData.confirmed_sacrifice}
              onCheckedChange={(checked) => sf.setFormData({ ...sf.formData, confirmed_sacrifice: checked })} />
          </div>

          {sf.formData.confirmed_sacrifice && (
            <div className="space-y-3 pt-2 border-t">
              <div className="flex items-center gap-2">
                <PenLine className="w-4 h-4 text-primary" />
                <Label className="text-base font-medium">{sf.t('signature.handwriting', '手寫簽名')}</Label>
                {sf.signatureStatus?.is_signed && (
                  <span className="signature-status-badge signature-status-signed">
                    <CheckCircle2 className="w-3.5 h-3.5" />
                    {sf.t('signature.signed', '已簽署')}
                  </span>
                )}
              </div>

              {sf.signatureStatus?.is_signed && sf.signatureStatus.signatures.length > 0 ? (
                <div className="space-y-2">
                  {sf.signatureStatus.signatures.map((sig) => (
                    <div key={sig.id} className="rounded-lg border bg-status-success-bg/50 p-3">
                      {sig.handwriting_svg && (
                        <div className="signature-preview-image mb-2" style={{ height: '120px' }}
                          dangerouslySetInnerHTML={{ __html: sanitizeSvg(sig.handwriting_svg) }} />
                      )}
                      <p className="text-xs text-muted-foreground">
                        {sf.t('signature.signedBy', '由 {{name}} 簽署於 {{date}}', {
                          name: sig.signer_name || '—',
                          date: new Date(sig.signed_at).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei' }),
                        })}
                      </p>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="space-y-2">
                  <p className="text-sm text-muted-foreground">{sf.t('signature.signRequired', '需要簽名確認')}</p>
                  <HandwrittenSignaturePad onSignatureChange={sf.setSignatureData} height={180}
                    disabled={sf.signatureStatus?.is_locked} />
                  {sf.isEdit && sacrifice?.id && sf.signatureData && (
                    <Button type="button" size="sm" className="bg-primary hover:bg-primary/90"
                      disabled={sf.signMutation.isPending}
                      onClick={() => sf.signMutation.mutate(sf.signatureData!)}>
                      {sf.signMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />}
                      <CheckCircle2 className="w-4 h-4 mr-1" />
                      {sf.t('signature.confirmSign', '確認簽署')}
                    </Button>
                  )}
                </div>
              )}
            </div>
          )}

          <div className="space-y-2">
            <Label>上傳照片</Label>
            <FileUpload value={sf.formData.photos}
              onChange={(photos) => sf.setFormData({ ...sf.formData, photos })}
              onUpload={sf.handlePhotoUpload} accept="image/*"
              placeholder="拖曳照片到此處，或點擊選擇照片" maxSize={10} maxFiles={10} />
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>取消</Button>
            <Button type="submit" disabled={sf.mutation.isPending}
              className="bg-status-success-solid hover:bg-status-success-solid/90">
              {sf.mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
