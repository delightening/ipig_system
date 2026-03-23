import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
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

const AVAILABLE_SCOPES = [
  { value: 'read', label: '唯讀查詢', description: '查詢動物、計畫、觀察、手術、體重、設施、庫存、人資資料' },
]

interface CreateAiKeyDialogProps {
  open: boolean
  onClose: () => void
  onCreated: (resp: CreateAiApiKeyResponse) => void
}

export function CreateAiKeyDialog({ open, onClose, onCreated }: CreateAiKeyDialogProps) {
  const [name, setName] = useState('')
  const [scopes, setScopes] = useState<string[]>(['read'])
  const [rateLimit, setRateLimit] = useState(60)
  const [expiresInDays, setExpiresInDays] = useState<string>('')

  const createMutation = useMutation({
    mutationFn: () => {
      const expires_at = expiresInDays
        ? new Date(Date.now() + parseInt(expiresInDays) * 86400000).toISOString()
        : null
      return aiApi.createKey({ name, scopes, rate_limit_per_minute: rateLimit, expires_at })
    },
    onSuccess: (resp) => {
      onCreated(resp)
      setName('')
      setScopes(['read'])
      setRateLimit(60)
      setExpiresInDays('')
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getErrorMessage(err), variant: 'destructive' })
    },
  })

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>建立 AI API Key</DialogTitle>
          <DialogDescription>
            建立一組金鑰供外部 AI 系統（如 Claude、ChatGPT）查詢 iPig 資料。
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          <div>
            <Label>金鑰名稱 *</Label>
            <Input
              placeholder="例如：Claude Desktop 查詢用"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
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
                      if (e.target.checked) {
                        setScopes([...scopes, scope.value])
                      } else {
                        setScopes(scopes.filter(s => s !== scope.value))
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
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>速率限制（次/分鐘）</Label>
              <Input
                type="number"
                min={1}
                max={600}
                value={rateLimit}
                onChange={(e) => setRateLimit(parseInt(e.target.value) || 60)}
              />
            </div>
            <div>
              <Label>有效天數（空白=永不過期）</Label>
              <Input
                type="number"
                min={1}
                placeholder="例如：365"
                value={expiresInDays}
                onChange={(e) => setExpiresInDays(e.target.value)}
              />
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>取消</Button>
          <Button
            onClick={() => createMutation.mutate()}
            disabled={!name.trim() || scopes.length === 0 || createMutation.isPending}
          >
            {createMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            建立金鑰
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
