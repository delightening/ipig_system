import { UseFormRegister, FieldErrors } from 'react-hook-form'

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

function FieldError({ message }: { message?: string }) {
  if (!message) return null
  return <p className="text-sm text-destructive col-start-2 col-span-3">{message}</p>
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
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isEditing ? '編輯夥伴' : '新增夥伴'}</DialogTitle>
          <DialogDescription>
            {isEditing ? '修改夥伴資料' : '建立新的供應商或客戶'}
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
              <Label htmlFor="name" className="text-right">名稱</Label>
              <Input
                id="name"
                {...register('name')}
                className="col-span-3"
                disabled={isEditing}
              />
              <FieldError message={errors.name?.message} />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="tax_id" className="text-right">統編</Label>
              <Input
                id="tax_id"
                {...register('tax_id')}
                className="col-span-3"
                placeholder="留白或 8 碼數字"
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
                placeholder="留白或正確 Email 格式"
              />
              <FieldError message={errors.email?.message} />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="address" className="text-right">地址</Label>
              <Input
                id="address"
                {...register('address')}
                className="col-span-3"
                placeholder="留白或地址字串"
              />
              <FieldError message={errors.address?.message} />
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>
              取消
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {isEditing ? '更新' : '建立'}
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
  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label className="text-right">類型</Label>
      <Select value={value} onValueChange={onChange} disabled={disabled}>
        <SelectTrigger className="col-span-3">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="supplier">供應商</SelectItem>
          <SelectItem value="customer">客戶</SelectItem>
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
  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label className="text-right">提供商類型</Label>
      <Select value={value} onValueChange={onChange as (v: string) => void} disabled={disabled}>
        <SelectTrigger className="col-span-3">
          <SelectValue placeholder="請選擇提供商類型" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="drug">藥物</SelectItem>
          <SelectItem value="consumable">耗材</SelectItem>
          <SelectItem value="feed">飼料</SelectItem>
          <SelectItem value="equipment">儀器</SelectItem>
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
  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label className="text-right">客戶分類</Label>
      <Select value={value} onValueChange={onChange as (v: string) => void} disabled={disabled}>
        <SelectTrigger className="col-span-3">
          <SelectValue placeholder="請選擇客戶分類" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="internal">內部單位</SelectItem>
          <SelectItem value="external">外部客戶</SelectItem>
          <SelectItem value="research">研究計畫</SelectItem>
          <SelectItem value="other">其他</SelectItem>
        </SelectContent>
      </Select>
    </div>
  )
}

function CodeField({ code, isGenerating }: { code: string; isGenerating: boolean }) {
  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label htmlFor="code" className="text-right">代碼</Label>
      <div className="col-span-3 flex gap-2">
        <Input
          id="code"
          value={code}
          disabled
          required
          placeholder={isGenerating ? '生成中...' : '系統自動編號'}
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
  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label htmlFor="phone" className="text-right">電話</Label>
      <div className="col-span-3 flex gap-2">
        <Input
          id="phone"
          className="flex-1"
          {...register('phone')}
          placeholder="留白或 9-10 碼數字"
        />
        <div className="flex items-center gap-2 shrink-0">
          <span className="text-muted-foreground">#</span>
          <Input
            className="w-24"
            placeholder="分機"
            {...register('phone_ext')}
          />
        </div>
      </div>
      <FieldError message={errors.phone?.message} />
    </div>
  )
}
