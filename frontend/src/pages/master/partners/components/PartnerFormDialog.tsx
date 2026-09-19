import { UseFormRegister, FieldErrors } from 'react-hook-form'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Loader2 } from 'lucide-react'
import {
  PartnerFormData,
  CustomerCategory,
  SupplierCategory,
} from '../constants'

interface PartnerFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  formData: Pick<PartnerFormData, 'partner_type' | 'supplier_category' | 'customer_category' | 'code'>
  register: UseFormRegister<PartnerFormData>
  setValue: (field: keyof PartnerFormData, value: string) => void
  errors: FieldErrors<PartnerFormData>
  isEditing: boolean
  isGeneratingCode: boolean
  isPending: boolean
  onPartnerTypeChange: (value: 'supplier' | 'customer') => void
  onSupplierCategoryChange: (category: SupplierCategory) => void
  onSubmit: (e: React.FormEvent) => void
  onClose: () => void
}

// message 是 i18n 鍵（見 usePartnerForm 的 FIELD_RULES），在此顯示時才翻譯
function FieldError({ message }: { message?: string }) {
  const { t } = useTranslation()
  if (!message) return null
  return <p className="text-sm text-destructive col-start-2 col-span-3">{t(message)}</p>
}

export function PartnerFormDialog({
  open,
  onOpenChange,
  formData,
  register,
  setValue,
  errors,
  isEditing,
  isGeneratingCode,
  isPending,
  onPartnerTypeChange,
  onSupplierCategoryChange,
  onSubmit,
  onClose,
}: PartnerFormDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isEditing ? t('erpMaster.partners.form.editTitle') : t('erpMaster.partners.form.addTitle')}</DialogTitle>
          <DialogDescription>
            {isEditing ? t('erpMaster.partners.form.editDescription') : t('erpMaster.partners.form.addDescription')}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={onSubmit}>
          <div className="grid gap-4 py-4">
            <PartnerTypeField
              value={formData.partner_type}
              onChange={onPartnerTypeChange}
              disabled={isEditing}
            />
            {formData.partner_type === 'supplier' && (
              <SupplierCategoryField
                value={formData.supplier_category}
                onChange={onSupplierCategoryChange}
                disabled={isEditing || isGeneratingCode}
              />
            )}
            {formData.partner_type === 'customer' && (
              <CustomerCategoryField
                value={formData.customer_category}
                onChange={(v) => setValue('customer_category', v)}
                disabled={isEditing}
              />
            )}
            <CodeField code={formData.code} isGenerating={isGeneratingCode} />
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="name" className="text-right">{t('erpMaster.common.name')}</Label>
              <Input
                id="name"
                {...register('name')}
                className="col-span-3"
                disabled={isEditing}
              />
              <FieldError message={errors.name?.message} />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="tax_id" className="text-right">{t('erpMaster.partners.taxId')}</Label>
              <Input
                id="tax_id"
                {...register('tax_id')}
                className="col-span-3"
                placeholder={t('erpMaster.partners.form.taxIdPlaceholder')}
                disabled={isEditing}
              />
              <FieldError message={errors.tax_id?.message} />
            </div>
            <PhoneField
              register={register}
              errors={errors}
            />
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="email" className="text-right">Email</Label>
              <Input
                id="email"
                {...register('email')}
                className="col-span-3"
                placeholder={t('erpMaster.partners.form.emailPlaceholder')}
              />
              <FieldError message={errors.email?.message} />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="address" className="text-right">{t('erpMaster.partners.address')}</Label>
              <Input
                id="address"
                {...register('address')}
                className="col-span-3"
                placeholder={t('erpMaster.partners.form.addressPlaceholder')}
              />
              <FieldError message={errors.address?.message} />
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {isEditing ? t('common.update') : t('erpMaster.common.createSubmit')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

/* ---- Internal sub-components ---- */

function PartnerTypeField({
  value,
  onChange,
  disabled,
}: {
  value: string
  onChange: (v: 'supplier' | 'customer') => void
  disabled: boolean
}) {
  const { t } = useTranslation()

  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label className="text-right">{t('erpMaster.partners.table.type')}</Label>
      <Select value={value} onValueChange={onChange} disabled={disabled}>
        <SelectTrigger className="col-span-3">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="supplier">{t('erpMaster.partners.type.supplier')}</SelectItem>
          <SelectItem value="customer">{t('erpMaster.partners.type.customer')}</SelectItem>
        </SelectContent>
      </Select>
    </div>
  )
}

function SupplierCategoryField({
  value,
  onChange,
  disabled,
}: {
  value: string
  onChange: (v: SupplierCategory) => void
  disabled: boolean
}) {
  const { t } = useTranslation()

  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label className="text-right">{t('erpMaster.partners.form.supplierCategoryLabel')}</Label>
      <Select value={value} onValueChange={onChange as (v: string) => void} disabled={disabled}>
        <SelectTrigger className="col-span-3">
          <SelectValue placeholder={t('erpMaster.partners.form.supplierCategoryPlaceholder')} />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="drug">{t('erpMaster.partners.supplierCategory.drug')}</SelectItem>
          <SelectItem value="consumable">{t('erpMaster.partners.supplierCategory.consumable')}</SelectItem>
          <SelectItem value="feed">{t('erpMaster.partners.supplierCategory.feed')}</SelectItem>
          <SelectItem value="equipment">{t('erpMaster.partners.supplierCategory.equipment')}</SelectItem>
        </SelectContent>
      </Select>
    </div>
  )
}

function CustomerCategoryField({
  value,
  onChange,
  disabled,
}: {
  value: string
  onChange: (v: CustomerCategory) => void
  disabled: boolean
}) {
  const { t } = useTranslation()

  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label className="text-right">{t('erpMaster.partners.customerCategoryLabel')}</Label>
      <Select value={value} onValueChange={onChange as (v: string) => void} disabled={disabled}>
        <SelectTrigger className="col-span-3">
          <SelectValue placeholder={t('erpMaster.partners.form.customerCategoryPlaceholder')} />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="internal">{t('erpMaster.partners.customerCategory.internal')}</SelectItem>
          <SelectItem value="external">{t('erpMaster.partners.customerCategory.external')}</SelectItem>
          <SelectItem value="research">{t('erpMaster.partners.customerCategory.research')}</SelectItem>
          <SelectItem value="other">{t('erpMaster.partners.customerCategory.other')}</SelectItem>
        </SelectContent>
      </Select>
    </div>
  )
}

function CodeField({ code, isGenerating }: { code: string; isGenerating: boolean }) {
  const { t } = useTranslation()

  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label htmlFor="code" className="text-right">{t('erpMaster.common.code')}</Label>
      <div className="col-span-3 flex gap-2">
        <Input
          id="code"
          value={code}
          disabled
          required
          placeholder={isGenerating ? t('erpMaster.partners.form.generating') : t('erpMaster.partners.form.codePlaceholder')}
        />
        {isGenerating && (
          <Loader2 className="h-4 w-4 animate-spin text-muted-foreground self-center" />
        )}
      </div>
    </div>
  )
}

function PhoneField({
  register,
  errors,
}: {
  register: UseFormRegister<PartnerFormData>
  errors: FieldErrors<PartnerFormData>
}) {
  const { t } = useTranslation()

  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label htmlFor="phone" className="text-right">{t('erpMaster.partners.phone')}</Label>
      <div className="col-span-3 flex gap-2">
        <Input
          id="phone"
          className="flex-1"
          {...register('phone')}
          placeholder={t('erpMaster.partners.form.phonePlaceholder')}
        />
        <div className="flex items-center gap-2 shrink-0">
          <span className="text-muted-foreground">#</span>
          <Input
            className="w-24"
            placeholder={t('erpMaster.partners.form.extension')}
            {...register('phone_ext')}
          />
        </div>
      </div>
      <FieldError message={errors.phone?.message} />
    </div>
  )
}
