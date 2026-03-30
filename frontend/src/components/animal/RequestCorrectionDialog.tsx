import { useState } from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { animalBreedNames, animalGenderNames, CORRECTABLE_FIELDS, type CorrectableField } from '@/lib/api'
import { Loader2 } from 'lucide-react'
import type { Animal } from '@/lib/api'

const FIELD_LABELS: Record<CorrectableField, string> = {
  ear_tag: '耳號',
  birth_date: '出生日期',
  gender: '性別',
  breed: '品種',
}

interface RequestCorrectionDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  animal: Animal
  onSubmit: (data: { field_name: string; new_value: string; reason: string }) => Promise<void>
}

export function RequestCorrectionDialog({
  open,
  onOpenChange,
  animal,
  onSubmit,
}: RequestCorrectionDialogProps) {
  const [field, setField] = useState<CorrectableField>('ear_tag')
  const [newValue, setNewValue] = useState('')
  const [reason, setReason] = useState('')
  const [isPending, setIsPending] = useState(false)

  const getCurrentValue = () => {
    switch (field) {
      case 'ear_tag':
        return animal.ear_tag
      case 'birth_date':
        return animal.birth_date ? new Date(animal.birth_date).toISOString().split('T')[0] : '-'
      case 'gender':
        return animalGenderNames[animal.gender]
      case 'breed':
        return animal.breed === 'other' ? (animal.breed_other || '其他') : animalBreedNames[animal.breed]
      default:
        return '-'
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newValue.trim() || !reason.trim()) return
    setIsPending(true)
    try {
      await onSubmit({
        field_name: field,
        new_value: newValue.trim(),
        reason: reason.trim(),
      })
      onOpenChange(false)
      setField('ear_tag')
      setNewValue('')
      setReason('')
    } finally {
      setIsPending(false)
    }
  }

  const handleOpenChange = (next: boolean) => {
    if (!next) {
      setField('ear_tag')
      setNewValue('')
      setReason('')
    }
    onOpenChange(next)
  }

  const renderFieldInput = () => {
    switch (field) {
      case 'ear_tag':
        return (
          <Input
            value={newValue}
            onChange={(e) => setNewValue(e.target.value)}
            placeholder="如：001"
            maxLength={10}
          />
        )
      case 'birth_date':
        return (
          <Input
            type="date"
            value={newValue}
            onChange={(e) => setNewValue(e.target.value)}
          />
        )
      case 'gender':
        return (
          <Select value={newValue} onValueChange={setNewValue}>
            <SelectTrigger>
              <SelectValue placeholder="選擇性別" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="male">公</SelectItem>
              <SelectItem value="female">母</SelectItem>
            </SelectContent>
          </Select>
        )
      case 'breed':
        return (
          <Select value={newValue} onValueChange={setNewValue}>
            <SelectTrigger>
              <SelectValue placeholder="選擇品種" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="miniature">迷你豬</SelectItem>
              <SelectItem value="white">白豬</SelectItem>
              <SelectItem value="LYD">LYD</SelectItem>
              <SelectItem value="other">其他</SelectItem>
            </SelectContent>
          </Select>
        )
      default:
        return null
    }
  }

  // 切換欄位時重置 newValue 並設定預設
  const handleFieldChange = (v: string) => {
    setField(v as CorrectableField)
    switch (v) {
      case 'ear_tag':
        setNewValue(animal.ear_tag)
        break
      case 'birth_date':
        setNewValue(animal.birth_date ? new Date(animal.birth_date).toISOString().split('T')[0] : '')
        break
      case 'gender':
        setNewValue(animal.gender)
        break
      case 'breed':
        setNewValue(animal.breed === 'other' ? 'other' : animal.breed === 'minipig' ? 'miniature' : animal.breed)
        break
      default:
        setNewValue('')
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>申請修正欄位</DialogTitle>
          <DialogDescription>
            耳號、出生日期、性別、品種等欄位建立後不可直接修改。若輸入錯誤，可提交修正申請，經系統管理員批准後套用。耳號：{animal.ear_tag}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>選擇要修正的欄位</Label>
              <Select value={field} onValueChange={handleFieldChange}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {CORRECTABLE_FIELDS.map((f) => (
                    <SelectItem key={f} value={f}>
                      {FIELD_LABELS[f]}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>目前值</Label>
              <Input value={getCurrentValue()} disabled className="bg-muted" />
            </div>
            <div className="space-y-2">
              <Label>修正後的值 *</Label>
              {renderFieldInput()}
            </div>
            <div className="space-y-2">
              <Label>修正原因 *</Label>
              <Textarea
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                placeholder="請說明為何需要修正（例如：建檔時誤植）"
                rows={3}
                required
              />
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => handleOpenChange(false)}>
              取消
            </Button>
            <Button
              type="submit"
              disabled={isPending || !newValue.trim() || !reason.trim()}
              className="bg-purple-600 hover:bg-purple-700"
            >
              {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              提交申請
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
