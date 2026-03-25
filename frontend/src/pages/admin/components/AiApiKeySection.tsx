import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Copy, Key, Plus, Trash2, Loader2, AlertCircle, CheckCircle2 } from 'lucide-react'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { toast } from '@/components/ui/use-toast'
import { aiApi } from '@/lib/api/ai'
import type { AiApiKeyInfo, CreateAiApiKeyResponse } from '@/lib/api/ai'
import { getErrorMessage } from '@/types/error'
import { formatDateTime } from '@/lib/utils'
import { CreateAiKeyDialog } from './CreateAiKeyDialog'
import { EmptyState } from '@/components/ui/empty-state'

const SCOPE_LABELS: Record<string, string> = { read: '唯讀查詢' }

export function AiApiKeySection() {
  const queryClient = useQueryClient()
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [createdKey, setCreatedKey] = useState<CreateAiApiKeyResponse | null>(null)
  const [copied, setCopied] = useState(false)

  const { data: keys, isLoading, error } = useQuery({
    queryKey: ['ai-api-keys'],
    queryFn: aiApi.listKeys,
    staleTime: 30_000,
  })

  const toggleMutation = useMutation({
    mutationFn: ({ id, is_active }: { id: string; is_active: boolean }) =>
      aiApi.toggleKey(id, is_active),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['ai-api-keys'] }),
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getErrorMessage(err), variant: 'destructive' })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => aiApi.deleteKey(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ai-api-keys'] })
      toast({ title: '成功', description: 'API Key 已刪除' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getErrorMessage(err), variant: 'destructive' })
    },
  })

  const handleCopyKey = async (key: string) => {
    await navigator.clipboard.writeText(key)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Key className="h-5 w-5" />
              AI API Key 管理
            </CardTitle>
            <CardDescription className="mt-1">
              管理外部 AI 系統存取 iPig 資料的 API 金鑰。金鑰僅在建立時顯示一次，請妥善保存。
            </CardDescription>
          </div>
          <Button size="sm" onClick={() => setShowCreateDialog(true)}>
            <Plus className="mr-1 h-4 w-4" />
            建立金鑰
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading && (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {error && (
          <div className="flex items-center gap-2 text-destructive py-4">
            <AlertCircle className="h-4 w-4" />
            <span>無法載入 API Key 清單</span>
          </div>
        )}

        {!isLoading && !error && (
          <>
            <KeyTable
              keys={keys ?? []}
              onToggle={(id, is_active) => toggleMutation.mutate({ id, is_active })}
              onDelete={(id, name) => {
                if (confirm(`確定要刪除 "${name}" 嗎？此操作無法復原。`)) {
                  deleteMutation.mutate(id)
                }
              }}
            />
            <UsageGuide />
          </>
        )}
      </CardContent>

      <CreateAiKeyDialog
        open={showCreateDialog}
        onClose={() => setShowCreateDialog(false)}
        onCreated={(resp) => {
          setShowCreateDialog(false)
          setCreatedKey(resp)
          queryClient.invalidateQueries({ queryKey: ['ai-api-keys'] })
        }}
      />

      <CreatedKeyDialog
        createdKey={createdKey}
        copied={copied}
        onCopy={handleCopyKey}
        onClose={() => setCreatedKey(null)}
      />
    </Card>
  )
}

function KeyTable({ keys, onToggle, onDelete }: {
  keys: AiApiKeyInfo[]
  onToggle: (id: string, is_active: boolean) => void
  onDelete: (id: string, name: string) => void
}) {
  if (keys.length === 0) {
    return (
      <EmptyState icon={Key} title="尚未建立任何 API Key" description="點擊「建立金鑰」來產生第一組 AI 存取金鑰" />
    )
  }

  return (
    <div className="border rounded-md overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>名稱</TableHead>
            <TableHead>金鑰前綴</TableHead>
            <TableHead>權限</TableHead>
            <TableHead className="text-center">速率</TableHead>
            <TableHead className="text-center">使用次數</TableHead>
            <TableHead>最後使用</TableHead>
            <TableHead>到期時間</TableHead>
            <TableHead className="text-center">啟用</TableHead>
            <TableHead className="text-center w-16">操作</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {keys.map((key) => (
            <TableRow key={key.id} className={key.is_active ? '' : 'opacity-50'}>
              <TableCell className="font-medium">{key.name}</TableCell>
              <TableCell>
                <code className="text-xs bg-muted px-1.5 py-0.5 rounded">{key.key_prefix}...</code>
              </TableCell>
              <TableCell>
                <div className="flex flex-wrap gap-1">
                  {key.scopes.map(s => (
                    <Badge key={s} variant="secondary" className="text-xs">{SCOPE_LABELS[s] || s}</Badge>
                  ))}
                </div>
              </TableCell>
              <TableCell className="text-center">{key.rate_limit_per_minute}/min</TableCell>
              <TableCell className="text-center">{key.usage_count.toLocaleString()}</TableCell>
              <TableCell className="text-sm">{key.last_used_at ? formatDateTime(key.last_used_at) : '-'}</TableCell>
              <TableCell className="text-sm">{key.expires_at ? formatDateTime(key.expires_at) : '永不過期'}</TableCell>
              <TableCell className="text-center">
                <Switch checked={key.is_active} onCheckedChange={(c) => onToggle(key.id, c)} />
              </TableCell>
              <TableCell className="text-center">
                <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive hover:text-destructive/80"
                  onClick={() => onDelete(key.id, key.name)} aria-label="刪除">
                  <Trash2 className="h-4 w-4" />
                </Button>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

function CreatedKeyDialog({ createdKey, copied, onCopy, onClose }: {
  createdKey: CreateAiApiKeyResponse | null
  copied: boolean
  onCopy: (key: string) => void
  onClose: () => void
}) {
  return (
    <Dialog open={!!createdKey} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <CheckCircle2 className="h-5 w-5 text-status-success-text" />
            API Key 建立成功
          </DialogTitle>
          <DialogDescription>
            請立即複製並妥善保存此金鑰，關閉後將無法再次查看完整金鑰。
          </DialogDescription>
        </DialogHeader>
        {createdKey && (
          <div className="space-y-3">
            <div>
              <Label className="text-sm font-medium">金鑰名稱</Label>
              <p className="text-sm">{createdKey.name}</p>
            </div>
            <div>
              <Label className="text-sm font-medium">API Key</Label>
              <div className="flex items-center gap-2 mt-1">
                <code className="flex-1 text-xs bg-muted p-2 rounded break-all font-mono">
                  {createdKey.api_key}
                </code>
                <Button size="sm" variant="outline" onClick={() => onCopy(createdKey.api_key)}>
                  {copied ? <CheckCircle2 className="h-4 w-4 text-status-success-text" /> : <Copy className="h-4 w-4" />}
                </Button>
              </div>
            </div>
            <div className="text-xs text-amber-600 bg-amber-50 p-2 rounded">
              此金鑰僅顯示一次。若遺失需刪除並重新建立。
            </div>
          </div>
        )}
        <DialogFooter>
          <Button onClick={onClose}>確認已複製，關閉</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function UsageGuide() {
  return (
    <div className="mt-6 p-4 bg-muted rounded-lg border text-sm space-y-3">
      <h4 className="font-semibold">使用說明</h4>
      <div className="space-y-2 text-muted-foreground">
        <p><strong>1. 建立金鑰：</strong>點擊「建立金鑰」，設定名稱與權限後取得 API Key。</p>
        <p><strong>2. 認證方式：</strong>在 HTTP 請求中加入 Header：</p>
        <code className="block bg-white p-2 rounded text-xs font-mono border">
          X-AI-API-Key: ipig_ai_xxxxxxxxxxxxxxxx
        </code>
        <p><strong>3. 可用端點：</strong></p>
        <ul className="list-disc list-inside space-y-1 ml-2">
          <li><code className="text-xs">GET /api/ai/overview</code> — 系統概覽</li>
          <li><code className="text-xs">GET /api/ai/schema</code> — 資料結構描述</li>
          <li><code className="text-xs">POST /api/ai/query</code> — 資料查詢</li>
        </ul>
        <p className="text-xs">
          查詢領域：animals / observations / surgeries / weights / protocols / facilities / stock / hr_summary
        </p>
      </div>
    </div>
  )
}
