import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
import { Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { FormField } from '@/components/ui/form-field'
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
// R57-2: 改 React Hook Form 原生 validation rules（避開 Zod 4 CSP eval probe）
type CreateAiKeyFormData = {
  name: string
  scopes: string[]
  rateLimit: number
  expiresInDays: string
}

const SCOPE_VALUES = ['read'] as const

interface CreateAiKeyDialogProps {
  open: boolean
  onClose: () => void
  onCreated: (resp: CreateAiApiKeyResponse) => void
}

export function CreateAiKeyDialog({ open, onClose, onCreated }: CreateAiKeyDialogProps) {
  const { t } = useTranslation()
  const availableScopes = SCOPE_VALUES.map(value => ({
    value,
    label: t(`admin.createAiKeyDialog.scope.${value}.label`),
    description: t(`admin.createAiKeyDialog.scope.${value}.description`),
  }))
  const { register, handleSubmit, setValue, watch, reset, setError, clearErrors, formState: { errors } } = useForm<CreateAiKeyFormData>({
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
      toast({ title: t('common.error'), description: getErrorMessage(err), variant: 'destructive' })
    },
  })

  const onValid = (data: CreateAiKeyFormData) => {
    if (!data.scopes || data.scopes.length === 0) {
      setError('scopes', { message: t('admin.createAiKeyDialog.scopeRequired') })
      return
    }
    clearErrors('scopes')
    createMutation.mutate(data)
  }

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('admin.createAiKeyDialog.title')}</DialogTitle>
          <DialogDescription>
            {t('admin.createAiKeyDialog.description')}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onValid)} className="space-y-4">
          <FormField label={t('admin.createAiKeyDialog.nameLabel')} required error={errors.name?.message}>
            <Input
              placeholder={t('admin.createAiKeyDialog.namePlaceholder')}
              {...register('name', { required: t('admin.createAiKeyDialog.nameRequired') })}
            />
          </FormField>
          <FormField label={t('admin.createAiKeyDialog.scopesLabel')} error={errors.scopes?.message}>
            <div className="mt-2 space-y-2">
              {availableScopes.map(scope => (
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
          </FormField>
          <div className="grid grid-cols-2 gap-4">
            <FormField label={t('admin.createAiKeyDialog.rateLimitLabel')} error={errors.rateLimit?.message}>
              <Input
                type="number"
                min={1}
                max={600}
                {...register('rateLimit', {
                  required: t('admin.createAiKeyDialog.rateLimitRequired'),
                  valueAsNumber: true,
                  min: { value: 1, message: t('admin.createAiKeyDialog.rateLimitMin') },
                  max: { value: 600, message: t('admin.createAiKeyDialog.rateLimitMax') },
                })}
              />
            </FormField>
            <FormField label={t('admin.createAiKeyDialog.expiresLabel')}>
              <Input
                type="number"
                min={1}
                placeholder={t('admin.createAiKeyDialog.expiresPlaceholder')}
                {...register('expiresInDays')}
              />
            </FormField>
          </div>
        </form>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>{t('common.cancel')}</Button>
          <Button
            onClick={handleSubmit(onValid)}
            disabled={createMutation.isPending}
          >
            {createMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t('admin.createAiKeyDialog.submit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
