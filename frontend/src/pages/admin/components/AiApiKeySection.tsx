import { useState } from 'react'
import { useTranslation } from 'react-i18next'
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
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { toast } from '@/components/ui/use-toast'
import { aiApi } from '@/lib/api/ai'
import type { AiApiKeyInfo, CreateAiApiKeyResponse } from '@/lib/api/ai'
import { getErrorMessage } from '@/types/error'
import { useTableSort } from '@/hooks/useTableSort'
import { formatDateTime } from '@/lib/utils'
import { CreateAiKeyDialog } from './CreateAiKeyDialog'
import { EmptyState } from '@/components/ui/empty-state'

const SCOPE_LABELS: Record<string, string> = { read: 'admin.aiApiKeySection.scopeRead' }

export function AiApiKeySection() {
  const { t } = useTranslation()
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
      toast({ title: t('common.error'), description: getErrorMessage(err), variant: 'destructive' })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => aiApi.deleteKey(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ai-api-keys'] })
      toast({ title: t('common.success'), description: t('admin.aiApiKeySection.deleteSuccess') })
    },
    onError: (err: unknown) => {
      toast({ title: t('common.error'), description: getErrorMessage(err), variant: 'destructive' })
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
              {t('admin.aiApiKeySection.title')}
            </CardTitle>
            <CardDescription className="mt-1">
              {t('admin.aiApiKeySection.description')}
            </CardDescription>
          </div>
          <Button size="sm" onClick={() => setShowCreateDialog(true)}>
            <Plus className="mr-1 h-4 w-4" />
            {t('admin.aiApiKeySection.createKey')}
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
            <span>{t('admin.aiApiKeySection.loadError')}</span>
          </div>
        )}

        {!isLoading && !error && (
          <>
            <KeyTable
              keys={keys ?? []}
              onToggle={(id, is_active) => toggleMutation.mutate({ id, is_active })}
              onDelete={(id, name) => {
                if (confirm(t('admin.aiApiKeySection.confirmDelete', { name }))) {
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
  const { t } = useTranslation()
  const { sortedData, sort, toggleSort } = useTableSort(keys)

  if (keys.length === 0) {
    return (
      <EmptyState icon={Key} title={t('admin.aiApiKeySection.emptyTitle')} description={t('admin.aiApiKeySection.emptyDescription')} />
    )
  }

  return (
    <div className="border rounded-md overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow>
            <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.aiApiKeySection.colName')}</SortableTableHead>
            <TableHead>{t('admin.aiApiKeySection.colKeyPrefix')}</TableHead>
            <TableHead>{t('admin.aiApiKeySection.colScopes')}</TableHead>
            <SortableTableHead sortKey="rate_limit_per_minute" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-center">{t('admin.aiApiKeySection.colRate')}</SortableTableHead>
            <SortableTableHead sortKey="usage_count" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-center">{t('admin.aiApiKeySection.colUsageCount')}</SortableTableHead>
            <SortableTableHead sortKey="last_used_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.aiApiKeySection.colLastUsed')}</SortableTableHead>
            <SortableTableHead sortKey="expires_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.aiApiKeySection.colExpiresAt')}</SortableTableHead>
            <TableHead className="text-center">{t('admin.aiApiKeySection.colActive')}</TableHead>
            <TableHead className="text-center w-16">{t('common.actions')}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {(sortedData ?? keys).map((key) => (
            <TableRow key={key.id} className={key.is_active ? '' : 'opacity-50'}>
              <TableCell className="font-medium">{key.name}</TableCell>
              <TableCell>
                <code className="text-xs bg-muted px-1.5 py-0.5 rounded">{key.key_prefix}...</code>
              </TableCell>
              <TableCell>
                <div className="flex flex-wrap gap-1">
                  {key.scopes.map(s => (
                    <Badge key={s} variant="secondary" className="text-xs">{SCOPE_LABELS[s] ? t(SCOPE_LABELS[s]) : s}</Badge>
                  ))}
                </div>
              </TableCell>
              <TableCell className="text-center">{t('admin.aiApiKeySection.ratePerMinute', { count: key.rate_limit_per_minute })}</TableCell>
              <TableCell className="text-center">{key.usage_count.toLocaleString()}</TableCell>
              <TableCell className="text-sm">{key.last_used_at ? formatDateTime(key.last_used_at) : '-'}</TableCell>
              <TableCell className="text-sm">{key.expires_at ? formatDateTime(key.expires_at) : t('admin.aiApiKeySection.neverExpires')}</TableCell>
              <TableCell className="text-center">
                <Switch checked={key.is_active} onCheckedChange={(c) => onToggle(key.id, c)} />
              </TableCell>
              <TableCell className="text-center">
                <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive hover:text-destructive/80"
                  onClick={() => onDelete(key.id, key.name)} aria-label={t('admin.aiApiKeySection.deleteKeyAria', { name: key.name })}>
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
  const { t } = useTranslation()
  return (
    <Dialog open={!!createdKey} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <CheckCircle2 className="h-5 w-5 text-status-success-text" />
            {t('admin.aiApiKeySection.createdTitle')}
          </DialogTitle>
          <DialogDescription>
            {t('admin.aiApiKeySection.createdDescription')}
          </DialogDescription>
        </DialogHeader>
        {createdKey && (
          <div className="space-y-3">
            <div>
              <Label className="text-sm font-medium">{t('admin.aiApiKeySection.keyNameLabel')}</Label>
              <p className="text-sm">{createdKey.name}</p>
            </div>
            <div>
              <Label className="text-sm font-medium">{t('admin.aiApiKeySection.apiKeyLabel')}</Label>
              <div className="flex items-center gap-2 mt-1">
                <code className="flex-1 text-xs bg-muted p-2 rounded break-all font-mono">
                  {createdKey.api_key}
                </code>
                <Button size="sm" variant="outline" onClick={() => onCopy(createdKey.api_key)} aria-label={t('admin.aiApiKeySection.copyKey')}>
                  {copied ? <CheckCircle2 className="h-4 w-4 text-status-success-text" /> : <Copy className="h-4 w-4" />}
                </Button>
              </div>
            </div>
            <div className="text-xs text-status-warning-text bg-status-warning-bg p-2 rounded">
              {t('admin.aiApiKeySection.showOnceWarning')}
            </div>
          </div>
        )}
        <DialogFooter>
          <Button onClick={onClose}>{t('admin.aiApiKeySection.confirmCopiedClose')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function UsageGuide() {
  const { t } = useTranslation()
  return (
    <div className="mt-6 p-4 bg-muted rounded-lg border text-sm space-y-3">
      <h4 className="font-semibold">{t('admin.aiApiKeySection.guideTitle')}</h4>
      <div className="space-y-2 text-muted-foreground">
        <p><strong>{t('admin.aiApiKeySection.guideStep1Label')}</strong>{t('admin.aiApiKeySection.guideStep1Text')}</p>
        <p><strong>{t('admin.aiApiKeySection.guideStep2Label')}</strong>{t('admin.aiApiKeySection.guideStep2Text')}</p>
        <code className="block bg-white p-2 rounded text-xs font-mono border">
          X-AI-API-Key: ipig_ai_xxxxxxxxxxxxxxxx
        </code>
        <p><strong>{t('admin.aiApiKeySection.guideStep3Label')}</strong></p>
        <ul className="list-disc list-inside space-y-1 ml-2">
          <li><code className="text-xs">GET /api/ai/overview</code> — {t('admin.aiApiKeySection.endpointOverview')}</li>
          <li><code className="text-xs">GET /api/ai/schema</code> — {t('admin.aiApiKeySection.endpointSchema')}</li>
          <li><code className="text-xs">POST /api/ai/query</code> — {t('admin.aiApiKeySection.endpointQuery')}</li>
        </ul>
        <p className="text-xs">
          {t('admin.aiApiKeySection.queryDomains')}
        </p>
      </div>
    </div>
  )
}
