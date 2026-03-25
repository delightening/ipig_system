import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { Loader2 } from 'lucide-react'

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
import { toast } from '@/components/ui/use-toast'
import { aiApi } from '@/lib/api/ai'
import type { CreateAiApiKeyResponse } from '@/lib/api/ai'
import { getErrorMessage } from '@/types/error'
import { createAiKeySchema, type CreateAiKeyFormData } from '@/lib/validation'

const AVAILABLE_SCOPES = [
  { value: 'read', label: '唯讀查詢', description: '查詢動物、計畫、觀察、手術、體重、設施、庫存、人資資料' },
]

interface CreateAiKeyDialogProps {
  open: boolean
  onClose: () => void
  onCreated: (resp: CreateAiApiKeyResponse) => void
}

export function CreateAiKeyDialog({ open, onClose, onCreated }: CreateAiKeyDialogProps) {
  const { t } = useTranslation()
  const { register, handleSubmit, setValue, watch, reset, formState: { errors } } = useForm<CreateAiKeyFormData>({
    resolver: zodResolver(createAiKeySchema),
    defaultValues: { name: '', scopes: ['read'], rateLimit: 60, expiresInDays: '' },
  })

  const scopes = watch('scopes')

  const createMutation = useMutation({
    mutationFn: (data: CreateAiKeyFormData) => {
      const expires_at = data.expiresInDays
        ? new Date(Date.now() + parseInt(data.expiresInDays) * 86400000).toISOString()
        : null
      return aiApi.createKey({ name: data.name, scopes: data.scopes, rate_limit_per_minute: data.rateLimit, expires_at })
    },
    onSuccess: (resp) => {
      onCreated(resp)
      reset()
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getErrorMessage(err), variant: 'destructive' })
    },
  })

  const onValid = (data: CreateAiKeyFormData) => {
    createMutation.mutate(data)
  }

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>建立 AI API Key</DialogTitle>
          <DialogDescription>
            建立一組金鑰供外部 AI 系統（如 Claude、ChatGPT）查詢 iPig 資料。
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onValid)} className="space-y-4">
          <div>
            <Label>金鑰名稱 *</Label>
            <Input
              placeholder="例如：Claude Desktop 查詢用"
              {...register('name')}
            />
            {errors.name && (
              <p className="text-sm text-destructive">{errors.name.message}</p>
            )}
          </div>
          <div>
            <Label>權限範圍</Label>
            <div className="mt-2 space-y-2">
              {AVAILABLE_SCOPES.map(scope => (
                <label key={scope.value} className="flex items-start gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={scopes.includes(scope.value)}
                    onChange={(e) => {
                      const current = scopes
                      if (e.target.checked) {
                        setValue('scopes', [...current, scope.value], { shouldValidate: true })
                      } else {
                        setValue('scopes', current.filter(s => s !== scope.value), { shouldValidate: true })
                      }
                    }}
                    className="mt-1"
                  />
                  <div>
                    <span className="font-medium text-sm">{scope.label}</span>
                    <p className="text-xs text-muted-foreground">{scope.description}</p>
                  </div>
                </label>
              ))}
            </div>
            {errors.scopes && (
              <p className="text-sm text-destructive">{errors.scopes.message}</p>
            )}
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>速率限制（次/分鐘）</Label>
              <Input
                type="number"
                min={1}
                max={600}
                {...register('rateLimit', { valueAsNumber: true })}
              />
              {errors.rateLimit && (
                <p className="text-sm text-destructive">{errors.rateLimit.message}</p>
              )}
            </div>
            <div>
              <Label>有效天數（空白=永不過期）</Label>
              <Input
                type="number"
                min={1}
                placeholder="例如：365"
                {...register('expiresInDays')}
              />
            </div>
          </div>
        </form>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>{t('common.cancel')}</Button>
          <Button
            onClick={handleSubmit(onValid)}
            disabled={createMutation.isPending}
          >
            {createMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            建立金鑰
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
