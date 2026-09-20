/**
 * MCP API Key 管理區塊
 * 嵌入個人設定頁（/profile/settings）
 * 讓執行秘書、主委、VET 產生/撤銷用於 claude.ai Remote MCP 的個人金鑰
 */
import { useState } from 'react'

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { useTranslation } from 'react-i18next'

import { Check, Copy, KeyRound, Loader2, Plus, Trash2 } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import { mcpKeysApi } from '@/lib/api'
import type { CreateMcpKeyResponse } from '@/lib/api'
import { formatDate } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/apiError'

export function McpKeysSection() {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const [showCreate, setShowCreate] = useState(false)
    const [newKeyName, setNewKeyName] = useState('')
    const [newKeyWrite, setNewKeyWrite] = useState(false)
    const [createdKey, setCreatedKey] = useState<CreateMcpKeyResponse | null>(null)
    const [copied, setCopied] = useState(false)
    const [revokingId, setRevokingId] = useState<string | null>(null)

    const { data: keys = [], isLoading } = useQuery({
        queryKey: ['mcp-keys'],
        queryFn: mcpKeysApi.list,
    })

    const createMutation = useMutation({
        mutationFn: ({ name, write }: { name: string; write: boolean }) =>
            mcpKeysApi.create(name, write),
        onSuccess: (data) => {
            setCreatedKey(data)
            setShowCreate(false)
            setNewKeyName('')
            setNewKeyWrite(false)
            queryClient.invalidateQueries({ queryKey: ['mcp-keys'] })
        },
        onError: (error: unknown) => {
            toast({
                title: t('mcpKeys.createFailed'),
                description: getApiErrorMessage(error, t('errors.tryAgainLater')),
                variant: 'destructive',
            })
        },
    })

    const revokeMutation = useMutation({
        mutationFn: (id: string) => mcpKeysApi.revoke(id),
        onSuccess: () => {
            toast({ title: t('mcpKeys.revoked') })
            setRevokingId(null)
            queryClient.invalidateQueries({ queryKey: ['mcp-keys'] })
        },
        onError: (error: unknown) => {
            toast({
                title: t('mcpKeys.revokeFailed'),
                description: getApiErrorMessage(error, t('errors.tryAgainLater')),
                variant: 'destructive',
            })
            setRevokingId(null)
        },
    })

    const handleCopy = async () => {
        if (!createdKey) return
        await navigator.clipboard.writeText(createdKey.full_key)
        setCopied(true)
        setTimeout(() => setCopied(false), 2000)
    }

    return (
        <>
            <Card>
                <CardHeader>
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <KeyRound className="h-5 w-5 text-primary" />
                            <CardTitle className="text-base">{t('mcpKeys.title')}</CardTitle>
                        </div>
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={() => setShowCreate(true)}
                            disabled={keys.length >= 5}
                        >
                            <Plus className="mr-1.5 h-3.5 w-3.5" />
                            {t('mcpKeys.generateButton')}
                        </Button>
                    </div>
                    <CardDescription>
                        {t('mcpKeys.description')}
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    {isLoading && (
                        <div className="flex items-center gap-2 py-4 text-sm text-muted-foreground">
                            <Loader2 className="h-4 w-4 animate-spin" />
                            {t('common.loading')}
                        </div>
                    )}

                    {!isLoading && keys.length === 0 && (
                        <p className="py-4 text-sm text-muted-foreground text-center">
                            {t('mcpKeys.empty')}
                        </p>
                    )}

                    {keys.length > 0 && (
                        <ul className="divide-y divide-border">
                            {keys.map((key) => (
                                <li
                                    key={key.id}
                                    className="flex items-center justify-between py-3"
                                >
                                    <div className="min-w-0">
                                        <div className="flex items-center gap-2">
                                            <p className="text-sm font-medium text-foreground truncate">
                                                {key.name}
                                            </p>
                                            <Badge
                                                variant={
                                                    key.scopes?.includes('write')
                                                        ? 'warning'
                                                        : 'secondary'
                                                }
                                            >
                                                {key.scopes?.includes('write') ? t('mcpKeys.scopeWrite') : t('mcpKeys.scopeReadOnly')}
                                            </Badge>
                                        </div>
                                        <p className="text-xs text-muted-foreground mt-0.5 font-mono">
                                            {key.key_prefix}...
                                        </p>
                                        <p className="text-xs text-muted-foreground mt-0.5">
                                            {t('mcpKeys.createdAt', { date: formatDate(key.created_at) })}
                                            {key.last_used_at && (
                                                <> · {t('mcpKeys.lastUsedAt', { date: formatDate(key.last_used_at) })}</>
                                            )}
                                            {key.expires_at && (
                                                <> · {t('mcpKeys.expiresAt', { date: formatDate(key.expires_at) })}</>
                                            )}
                                        </p>
                                    </div>
                                    <Button
                                        size="sm"
                                        variant="ghost"
                                        className="text-destructive hover:text-destructive shrink-0 ml-4"
                                        onClick={() => setRevokingId(key.id)}
                                        disabled={revokeMutation.isPending}
                                    >
                                        <Trash2 className="h-3.5 w-3.5" />
                                    </Button>
                                </li>
                            ))}
                        </ul>
                    )}

                    <div className="mt-4 rounded-md bg-muted/50 p-3 text-xs text-muted-foreground space-y-1">
                        <p className="font-medium text-foreground">{t('mcpKeys.setup.heading')}</p>
                        <p>{t('mcpKeys.setup.step1')}</p>
                        <p>2. claude.ai → Settings → Integrations → Add MCP Server</p>
                        <p className="font-mono">{t('mcpKeys.setup.urlLine', { url: 'https://ipigsystem.asia/api/v1/mcp' })}</p>
                        <p className="font-mono">{t('mcpKeys.setup.authLine')}</p>
                    </div>
                </CardContent>
            </Card>

            {/* 建立新金鑰 Dialog */}
            <Dialog
                open={showCreate}
                onOpenChange={(open) => {
                    setShowCreate(open)
                    if (!open) setNewKeyWrite(false)
                }}
            >
                <DialogContent size="sm">
                    <DialogHeader>
                        <DialogTitle>{t('mcpKeys.createDialog.title')}</DialogTitle>
                        <DialogDescription>
                            {t('mcpKeys.createDialog.description')}
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4">
                        <div className="space-y-2">
                            <Label htmlFor="key-name">{t('mcpKeys.createDialog.nameLabel')}</Label>
                            <Input
                                id="key-name"
                                placeholder={t('mcpKeys.createDialog.namePlaceholder')}
                                value={newKeyName}
                                onChange={(e) => setNewKeyName(e.target.value)}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter' && newKeyName.trim()) {
                                        createMutation.mutate({
                                            name: newKeyName.trim(),
                                            write: newKeyWrite,
                                        })
                                    }
                                }}
                            />
                        </div>
                        <div className="space-y-1.5">
                            <Checkbox
                                id="key-write"
                                label={t('mcpKeys.createDialog.writeLabel')}
                                checked={newKeyWrite}
                                onCheckedChange={setNewKeyWrite}
                            />
                            <p className="text-xs text-muted-foreground pl-6">
                                {t('mcpKeys.createDialog.writeHint')}
                            </p>
                        </div>
                    </div>
                    <DialogFooter>
                        <Button
                            variant="outline"
                            onClick={() => setShowCreate(false)}
                            disabled={createMutation.isPending}
                        >
                            {t('common.cancel')}
                        </Button>
                        <Button
                            onClick={() =>
                                createMutation.mutate({
                                    name: newKeyName.trim(),
                                    write: newKeyWrite,
                                })
                            }
                            disabled={!newKeyName.trim() || createMutation.isPending}
                        >
                            {createMutation.isPending && (
                                <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                            )}
                            {t('mcpKeys.createDialog.submit')}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* 金鑰顯示 Dialog（一次性） */}
            <Dialog
                open={!!createdKey}
                onOpenChange={(open) => {
                    if (!open) setCreatedKey(null)
                }}
            >
                <DialogContent size="sm">
                    <DialogHeader>
                        <DialogTitle>{t('mcpKeys.createdDialog.title')}</DialogTitle>
                        <DialogDescription>
                            {t('mcpKeys.createdDialog.description')}
                        </DialogDescription>
                    </DialogHeader>
                    {createdKey && (
                        <div className="space-y-3">
                            <div className="rounded-md bg-muted p-3 font-mono text-sm break-all select-all">
                                {createdKey.full_key}
                            </div>
                            <Button
                                variant="outline"
                                className="w-full"
                                onClick={handleCopy}
                            >
                                {copied ? (
                                    <>
                                        <Check className="mr-1.5 h-4 w-4 text-green-600" />
                                        {t('mcpKeys.createdDialog.copied')}
                                    </>
                                ) : (
                                    <>
                                        <Copy className="mr-1.5 h-4 w-4" />
                                        {t('mcpKeys.createdDialog.copyKey')}
                                    </>
                                )}
                            </Button>
                        </div>
                    )}
                    <DialogFooter>
                        <Button onClick={() => setCreatedKey(null)}>{t('mcpKeys.createdDialog.copiedAndClose')}</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* 撤銷確認 Dialog */}
            <Dialog
                open={!!revokingId}
                onOpenChange={(open) => {
                    if (!open) setRevokingId(null)
                }}
            >
                <DialogContent size="sm">
                    <DialogHeader>
                        <DialogTitle>{t('mcpKeys.revokeDialog.title')}</DialogTitle>
                        <DialogDescription>
                            {t('mcpKeys.revokeDialog.description')}
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <Button
                            variant="outline"
                            onClick={() => setRevokingId(null)}
                            disabled={revokeMutation.isPending}
                        >
                            {t('common.cancel')}
                        </Button>
                        <Button
                            variant="destructive"
                            onClick={() => revokingId && revokeMutation.mutate(revokingId)}
                            disabled={revokeMutation.isPending}
                        >
                            {revokeMutation.isPending && (
                                <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                            )}
                            {t('mcpKeys.revokeDialog.confirm')}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </>
    )
}
