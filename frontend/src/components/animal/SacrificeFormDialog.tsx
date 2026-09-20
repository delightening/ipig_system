import { useTranslation } from 'react-i18next'

import { AnimalSacrifice } from '@/lib/api'
import { uiLocale } from '@/lib/utils'
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

// value 為送後端的採樣部位字面值（維持中文），只有 labelKey 是顯示用
const SAMPLING_OPTIONS = [
  { value: '心', labelKey: 'animalActions.sacrifice.sampling.heart' },
  { value: '肝', labelKey: 'animalActions.sacrifice.sampling.liver' },
  { value: '脾', labelKey: 'animalActions.sacrifice.sampling.spleen' },
  { value: '肺', labelKey: 'animalActions.sacrifice.sampling.lung' },
  { value: '腎', labelKey: 'animalActions.sacrifice.sampling.kidney' },
  { value: '眼', labelKey: 'animalActions.sacrifice.sampling.eye' },
  { value: '耳', labelKey: 'animalActions.sacrifice.sampling.ear' },
  { value: '舌', labelKey: 'animalActions.sacrifice.sampling.tongue' },
  { value: '腦', labelKey: 'animalActions.sacrifice.sampling.brain' },
  { value: '骨組織', labelKey: 'animalActions.sacrifice.sampling.boneTissue' },
  { value: '脂肪', labelKey: 'animalActions.sacrifice.sampling.fat' },
  { value: '肌肉', labelKey: 'animalActions.sacrifice.sampling.muscle' },
  { value: '皮膚', labelKey: 'animalActions.sacrifice.sampling.skin' },
  { value: '其他', labelKey: 'animalActions.common.other' },
]

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  animalId: string
  earTag: string
  sacrifice?: AnimalSacrifice
}

export function SacrificeFormDialog({ open, onOpenChange, animalId, earTag, sacrifice }: Props) {
  const { t } = useTranslation()
  const sf = useSacrificeForm({ open, animalId, sacrifice, onOpenChange })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="lg">
        <DialogHeader>
          <DialogTitle>{sf.isEdit ? t('animalActions.sacrifice.form.editTitle') : t('animalActions.sacrifice.form.createTitle')}</DialogTitle>
          <DialogDescription>{t('animalActions.common.earTagLabel', { earTag })}</DialogDescription>
        </DialogHeader>

        <form onSubmit={sf.handleSubmit} className="space-y-6">
          <div className="space-y-2">
            <Label htmlFor="sacrifice_date">{t('animalActions.sacrifice.form.sacrificeDateRequired')}</Label>
            <Input id="sacrifice_date" type="date" value={sf.formData.sacrifice_date}
              onChange={(e) => sf.setFormData({ ...sf.formData, sacrifice_date: e.target.value })} required />
          </div>

          <div className="space-y-4">
            <Label>{t('animalActions.sacrifice.form.method')}</Label>
            <div className="space-y-2">
              <Label htmlFor="zoletil_dose" className="text-sm font-normal">Zoletil-50 (ml)</Label>
              <Input id="zoletil_dose" type="text" value={sf.formData.zoletil_dose}
                onChange={(e) => sf.setFormData({ ...sf.formData, zoletil_dose: e.target.value })} placeholder={t('animalActions.sacrifice.form.doseHint')} />
            </div>
            <div className="space-y-2">
              <div className="flex flex-wrap gap-4">
                <Checkbox label={t('animalActions.sacrifice.form.electrocution220')} checked={sf.formData.method_electrocution}
                  onCheckedChange={(checked) => sf.setFormData({ ...sf.formData, method_electrocution: checked })} />
                <Checkbox label={t('animalActions.sacrifice.bloodletting')} checked={sf.formData.method_bloodletting}
                  onCheckedChange={(checked) => sf.setFormData({ ...sf.formData, method_bloodletting: checked })} />
                <Checkbox label={t('animalActions.common.other')} checked={sf.formData.method_other_enabled}
                  onCheckedChange={(checked) => sf.setFormData({
                    ...sf.formData, method_other_enabled: checked,
                    method_other: checked ? (sf.formData.method_other || '') : ''
                  })} />
              </div>
              {sf.formData.method_other_enabled && (
                <Input type="text" value={sf.formData.method_other}
                  onChange={(e) => sf.setFormData({ ...sf.formData, method_other: e.target.value })}
                  placeholder={t('animalActions.sacrifice.form.otherMethodHint')} className="mt-2" />
              )}
            </div>
          </div>

          <div className="space-y-4">
            <Label>{t('animalActions.sacrifice.form.samplingSites')}</Label>
            <div className="grid grid-cols-4 gap-3">
              {SAMPLING_OPTIONS.map((option) => (
                <Checkbox key={option.value} label={t(option.labelKey)}
                  checked={sf.formData.sampling.includes(option.value)}
                  onCheckedChange={(checked) => sf.handleSamplingChange(option.value, checked)} />
              ))}
            </div>
            {sf.hasOtherSampling && (
              <div className="mt-2">
                <Input type="text" value={sf.formData.sampling_other}
                  onChange={(e) => sf.setFormData({ ...sf.formData, sampling_other: e.target.value })}
                  placeholder={t('animalActions.sacrifice.form.otherSamplingHint')} />
              </div>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="blood_volume_ml">{t('animalActions.sacrifice.form.bloodSampled')}</Label>
            <Input id="blood_volume_ml" type="number" step="0.1" value={sf.formData.blood_volume_ml}
              onChange={(e) => sf.setFormData({ ...sf.formData, blood_volume_ml: e.target.value })}
              placeholder={t('animalActions.sacrifice.form.bloodVolumeHint')} />
          </div>

          <div className="space-y-2">
            <Checkbox label={t('animalActions.sacrifice.confirmedSacrifice')} checked={sf.formData.confirmed_sacrifice}
              onCheckedChange={(checked) => sf.setFormData({ ...sf.formData, confirmed_sacrifice: checked })} />
          </div>

          {sf.formData.confirmed_sacrifice && (
            <div className="space-y-3 pt-2 border-t">
              <div className="flex items-center gap-2">
                <PenLine className="w-4 h-4 text-primary" />
                <Label className="text-base font-medium">{sf.t('signature.handwriting')}</Label>
                {sf.signatureStatus?.is_signed && (
                  <span className="signature-status-badge signature-status-signed">
                    <CheckCircle2 className="w-3.5 h-3.5" />
                    {sf.t('signature.signed')}
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
                        {sf.t('signature.signedBy', {
                          name: sig.signer_name || '—',
                          date: new Date(sig.signed_at).toLocaleString(uiLocale(), { timeZone: 'Asia/Taipei' }),
                        })}
                      </p>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="space-y-2">
                  <p className="text-sm text-muted-foreground">{sf.t('signature.signRequired')}</p>
                  <HandwrittenSignaturePad onSignatureChange={sf.setSignatureData} height={180}
                    disabled={sf.signatureStatus?.is_locked} />
                  {sf.isEdit && sacrifice?.id && sf.signatureData && (
                    <Button type="button" size="sm" className="bg-primary hover:bg-primary/90"
                      disabled={sf.signMutation.isPending}
                      onClick={() => sf.signMutation.mutate(sf.signatureData!)}>
                      {sf.signMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />}
                      <CheckCircle2 className="w-4 h-4 mr-1" />
                      {sf.t('signature.confirmSign')}
                    </Button>
                  )}
                </div>
              )}
            </div>
          )}

          <div className="space-y-2">
            <Label>{t('animalActions.common.uploadPhotos')}</Label>
            <FileUpload value={sf.formData.photos}
              onChange={(photos) => sf.setFormData({ ...sf.formData, photos })}
              onUpload={sf.handlePhotoUpload} accept="image/*"
              placeholder={t('animalActions.sacrifice.form.photoPlaceholder')} maxSize={10} maxFiles={10} />
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>{t('common.cancel')}</Button>
            <Button type="submit" disabled={sf.mutation.isPending}
              className="bg-status-success-solid hover:bg-status-success-solid/90">
              {sf.mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('common.save')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
