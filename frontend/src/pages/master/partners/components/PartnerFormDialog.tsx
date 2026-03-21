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
  formData: PartnerFormData
  setFormData: React.Dispatch<React.SetStateAction<PartnerFormData>>
  isEditing: boolean
  isGeneratingCode: boolean
  isPending: boolean
  onPartnerTypeChange: (value: 'supplier' | 'customer') => void
  onSupplierCategoryChange: (category: SupplierCategory) => void
  onSubmit: (e: React.FormEvent) => void
  onClose: () => void
}

export function PartnerFormDialog({
  open,
  onOpenChange,
  formData,
  setFormData,
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
                onChange={(v) => setFormData(prev => ({ ...prev, customer_category: v }))}
                disabled={isEditing}
              />
            )}
            <CodeField code={formData.code} isGenerating={isGeneratingCode} />
            <TextField
              id="name"
              label="名稱"
              value={formData.name}
              onChange={(v) => setFormData(prev => ({ ...prev, name: v }))}
              required
              disabled={isEditing}
            />
            <TextField
              id="tax_id"
              label="統編"
              value={formData.tax_id}
              onChange={(v) => setFormData(prev => ({ ...prev, tax_id: v }))}
              placeholder="留白或 8 碼數字"
              disabled={isEditing}
            />
            <PhoneField
              phone={formData.phone}
              phoneExt={formData.phone_ext}
              onPhoneChange={(v) => setFormData(prev => ({ ...prev, phone: v }))}
              onPhoneExtChange={(v) => setFormData(prev => ({ ...prev, phone_ext: v }))}
            />
            <TextField
              id="email"
              label="Email"
              value={formData.email}
              onChange={(v) => setFormData(prev => ({ ...prev, email: v }))}
              placeholder="留白或正確 Email 格式"
              type="email"
            />
            <TextField
              id="address"
              label="地址"
              value={formData.address}
              onChange={(v) => setFormData(prev => ({ ...prev, address: v }))}
              placeholder="留白或地址字串"
            />
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

function TextField({
  id,
  label,
  value,
  onChange,
  placeholder,
  required,
  disabled,
  type,
}: {
  id: string
  label: string
  value: string
  onChange: (v: string) => void
  placeholder?: string
  required?: boolean
  disabled?: boolean
  type?: string
}) {
  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label htmlFor={id} className="text-right">{label}</Label>
      <Input
        id={id}
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="col-span-3"
        placeholder={placeholder}
        required={required}
        disabled={disabled}
      />
    </div>
  )
}

function PhoneField({
  phone,
  phoneExt,
  onPhoneChange,
  onPhoneExtChange,
}: {
  phone: string
  phoneExt: string
  onPhoneChange: (v: string) => void
  onPhoneExtChange: (v: string) => void
}) {
  return (
    <div className="grid grid-cols-4 items-center gap-4">
      <Label htmlFor="phone" className="text-right">電話</Label>
      <div className="col-span-3 flex gap-2">
        <Input
          id="phone"
          className="flex-1"
          value={phone}
          onChange={(e) => onPhoneChange(e.target.value)}
          placeholder="留白或 9-10 碼數字"
        />
        <div className="flex items-center gap-2 shrink-0">
          <span className="text-muted-foreground">#</span>
          <Input
            className="w-24"
            placeholder="分機"
            value={phoneExt}
            onChange={(e) => onPhoneExtChange(e.target.value)}
          />
        </div>
      </div>
    </div>
  )
}
