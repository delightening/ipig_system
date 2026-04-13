/**
 * MCP API Key 管理區塊
 * 嵌入個人設定頁（/profile/settings）
 * 讓執行秘書、主委、VET 產生/撤銷用於 claude.ai Remote MCP 的個人金鑰
 */
import { useState } from 'react'

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { Check, Copy, KeyRound, Loader2, Plus, Trash2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
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
import { getApiErrorMessage } from '@/lib/validation'

export function McpKeysSection() {
    const queryClient = useQueryClient()
    const [showCreate, setShowCreate] = useState(false)
    const [newKeyName, setNewKeyName] = useState('')
    const [createdKey, setCreatedKey] = useState<CreateMcpKeyResponse | null>(null)
    const [copied, setCopied] = useState(false)
    const [revokingId, setRevokingId] = useState<string | null>(null)

    const { data: keys = [], isLoading } = useQuery({
        queryKey: ['mcp-keys'],
        queryFn: mcpKeysApi.list,
    })

    const createMutation = useMutation({
        mutationFn: (name: string) => mcpKeysApi.create(name),
        onSuccess: (data) => {
            setCreatedKey(data)
            setShowCreate(false)
            setNewKeyName('')
            queryClient.invalidateQueries({ queryKey: ['mcp-keys'] })
        },
        onError: (error: unknown) => {
            toast({
                title: '建立失敗',
                description: getApiErrorMessage(error, '請稍後再試'),
                variant: 'destructive',
            })
        },
    })

    const revokeMutation = useMutation({
        mutationFn: (id: string) => mcpKeysApi.revoke(id),
        onSuccess: () => {
            toast({ title: '金鑰已撤銷' })
            setRevokingId(null)
            queryClient.invalidateQueries({ queryKey: ['mcp-keys'] })
        },
        onError: (error: unknown) => {
            toast({
                title: '撤銷失敗',
                description: getApiErrorMessage(error, '請稍後再試'),
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
                            <CardTitle className="text-base">MCP 連線金鑰</CardTitle>
                        </div>
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={() => setShowCreate(true)}
                            disabled={keys.length >= 5}
                        >
                            <Plus className="mr-1.5 h-3.5 w-3.5" />
                            產生新金鑰
                        </Button>
                    </div>
                    <CardDescription>
                        用於 claude.ai Remote MCP 連線，讓 Claude 直接讀取計畫書並協助審查。
                        每人最多 5 個有效金鑰。
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    {isLoading && (
                        <div className="flex items-center gap-2 py-4 text-sm text-muted-foreground">
                            <Loader2 className="h-4 w-4 animate-spin" />
                            載入中...
                        </div>
                    )}

                    {!isLoading && keys.length === 0 && (
                        <p className="py-4 text-sm text-muted-foreground text-center">
                            尚無金鑰。點擊「產生新金鑰」開始使用。
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
                                        <p className="text-sm font-medium text-foreground truncate">
                                            {key.name}
                                        </p>
                                        <p className="text-xs text-muted-foreground mt-0.5 font-mono">
                                            {key.key_prefix}...
                                        </p>
                                        <p className="text-xs text-muted-foreground mt-0.5">
                                            建立：{formatDate(key.created_at)}
                                            {key.last_used_at && (
                                                <> · 最後使用：{formatDate(key.last_used_at)}</>
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
                        <p className="font-medium text-foreground">連接 claude.ai 設定方式：</p>
                        <p>1. 產生金鑰後複製完整 key（僅顯示一次）</p>
                        <p>2. claude.ai → Settings → Integrations → Add MCP Server</p>
                        <p className="font-mono">URL：https://ipigsystem.asia/api/v1/mcp</p>
                        <p className="font-mono">Authorization：Bearer &lt;你的金鑰&gt;</p>
                    </div>
                </CardContent>
            </Card>

            {/* 建立新金鑰 Dialog */}
            <Dialog open={showCreate} onOpenChange={setShowCreate}>
                <DialogContent className="sm:max-w-sm">
                    <DialogHeader>
                        <DialogTitle>產生新 MCP 金鑰</DialogTitle>
                        <DialogDescription>
                            為此金鑰取一個名稱，例如「我的 claude.ai」。金鑰只顯示一次，請立即複製。
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-2">
                        <Label htmlFor="key-name">金鑰名稱</Label>
                        <Input
                            id="key-name"
                            placeholder="我的 claude.ai"
                            value={newKeyName}
                            onChange={(e) => setNewKeyName(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter' && newKeyName.trim()) {
                                    createMutation.mutate(newKeyName.trim())
                                }
                            }}
                        />
                    </div>
                    <DialogFooter>
                        <Button
                            variant="outline"
                            onClick={() => setShowCreate(false)}
                            disabled={createMutation.isPending}
                        >
                            取消
                        </Button>
                        <Button
                            onClick={() => createMutation.mutate(newKeyName.trim())}
                            disabled={!newKeyName.trim() || createMutation.isPending}
                        >
                            {createMutation.isPending && (
                                <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                            )}
                            產生
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
                <DialogContent className="sm:max-w-md">
                    <DialogHeader>
                        <DialogTitle>金鑰已產生</DialogTitle>
                        <DialogDescription>
                            請立即複製此金鑰，關閉後將無法再次查看完整內容。
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
                                        已複製
                                    </>
                                ) : (
                                    <>
                                        <Copy className="mr-1.5 h-4 w-4" />
                                        複製金鑰
                                    </>
                                )}
                            </Button>
                        </div>
                    )}
                    <DialogFooter>
                        <Button onClick={() => setCreatedKey(null)}>我已複製，關閉</Button>
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
                <DialogContent className="sm:max-w-sm">
                    <DialogHeader>
                        <DialogTitle>確認撤銷金鑰</DialogTitle>
                        <DialogDescription>
                            撤銷後此金鑰將立即失效，所有使用此金鑰的 MCP 連線將無法繼續存取。
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <Button
                            variant="outline"
                            onClick={() => setRevokingId(null)}
                            disabled={revokeMutation.isPending}
                        >
                            取消
                        </Button>
                        <Button
                            variant="destructive"
                            onClick={() => revokingId && revokeMutation.mutate(revokingId)}
                            disabled={revokeMutation.isPending}
                        >
                            {revokeMutation.isPending && (
                                <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                            )}
                            確認撤銷
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </>
    )
}
