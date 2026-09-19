// R40-A 站內信主頁
//
// 左側 thread 列表 + 右側 thread view（訊息流 + 輸入框 + 附件）
// 30s polling unread count；建新對話 dialog；圖片附件壓縮（複用 imageCompress.ts）

import { useState, useMemo, useRef, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { useAuthUser, useAuthHasPermission } from '@/stores/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { compressImage } from '@/lib/imageCompress'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { SearchableSelect } from '@/components/ui/searchable-select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import {
    Dialog, DialogContent, DialogHeader, DialogTitle,
} from '@/components/ui/dialog'
import {
    MessageSquare, Plus, Send, ImagePlus, Trash2, Loader2, Users, X,
} from 'lucide-react'
import { format } from 'date-fns'

// ── 型別 ──────────────────────────────

interface ParticipantSummary {
    user_id: string
    display_name: string
    email: string
    last_read_at: string | null
}

interface MessageAttachment {
    id: string
    message_id: string | null
    file_name: string
    file_size: number
    mime_type: string
    sort_order: number
    created_at: string
}

interface MessageWithSender {
    id: string
    thread_id: string
    sender_id: string
    body: string
    created_at: string
    edited_at: string | null
    deleted_at: string | null
    sender_display_name: string
    attachments: MessageAttachment[]
}

interface ThreadSummary {
    id: string
    type: 'direct' | 'group'
    subject: string | null
    created_by: string
    created_at: string
    last_message_at: string
    participants: ParticipantSummary[]
    unread_count: number
    last_message_preview: string | null
}

interface ThreadWithMessages {
    id: string
    type: 'direct' | 'group'
    subject: string | null
    created_by: string
    created_at: string
    last_message_at: string
    participants: ParticipantSummary[]
    messages: MessageWithSender[]
}

interface StaffOption { id: string; display_name: string; email: string }

// ── 主頁 ──────────────────────────────

export default function MessagingPage() {
    const { t } = useTranslation()
    const qc = useQueryClient()
    const [selectedThreadId, setSelectedThreadId] = useState<string | null>(null)
    const [showNewDialog, setShowNewDialog] = useState(false)

    const { data: threads = [], refetch: refetchThreads } = useQuery({
        queryKey: ['messaging-threads'],
        queryFn: async () => {
            const res = await api.get<ThreadSummary[]>('/messages/threads')
            return res.data
        },
        refetchInterval: 30_000,
    })

    const { data: thread, refetch: refetchThread } = useQuery({
        queryKey: ['messaging-thread', selectedThreadId],
        queryFn: async () => {
            const res = await api.get<ThreadWithMessages>(`/messages/threads/${selectedThreadId}`)
            return res.data
        },
        enabled: !!selectedThreadId,
        refetchInterval: 30_000,
    })

    // 進入 thread 時 mark read
    useEffect(() => {
        if (selectedThreadId) {
            // mark-read POST 完成後才失效 threads 列表：併發下列表可能比 last_read_at 先刷新，
            // 短暫還原舊 unread_count。未讀數為每 thread 的 unread_count（來自 messaging-threads），
            // 刷新即更新徽章；原 ['messaging-unread'] 為死 key（無 query 消費）已移除。
            api.post(`/messages/threads/${selectedThreadId}/read`)
                .then(() => qc.invalidateQueries({ queryKey: ['messaging-threads'] }))
                .catch(() => {})
        }
    }, [selectedThreadId, qc])

    return (
        // 站內信高度限制在 main 內容區內：
        // - main sticky top header (~4rem) + 父層 p-3/p-4 padding (~2rem) ≈ 6rem chrome
        // - 之前 9rem 多扣了 3rem 造成下方多空白
        <div className="flex h-[calc(100dvh-6rem)] bg-background overflow-hidden border rounded-lg">
            {/* 左側：thread 列表 */}
            <div className="w-80 border-r flex flex-col min-h-0">
                <div className="flex items-center justify-between px-4 py-3 border-b shrink-0">
                    <h2 className="text-lg font-semibold flex items-center gap-2">
                        <MessageSquare className="h-5 w-5" /> {t('nav.messaging')}
                    </h2>
                    <Button size="sm" onClick={() => setShowNewDialog(true)}>
                        <Plus className="h-4 w-4 mr-1" /> {t('messagingPage.newMessage')}
                    </Button>
                </div>
                <div className="flex-1 overflow-y-auto min-h-0">
                    {threads.length === 0 ? (
                        <p className="text-center text-muted-foreground text-sm p-6">{t('messagingPage.noThreads')}</p>
                    ) : threads.map(item => (
                        <ThreadListItem
                            key={item.id}
                            thread={item}
                            selected={item.id === selectedThreadId}
                            onClick={() => setSelectedThreadId(item.id)}
                        />
                    ))}
                </div>
            </div>

            {/* 右側：thread view — relative 定義 absolute 定位上下文，composer 才能釘底 */}
            <div className="flex-1 flex flex-col min-h-0 relative">
                {thread ? (
                    <ThreadView thread={thread} onMessageSent={() => refetchThread()} />
                ) : (
                    <div className="flex-1 flex items-center justify-center text-muted-foreground">
                        {t('messagingPage.selectPrompt')}
                    </div>
                )}
            </div>

            <NewThreadDialog
                open={showNewDialog}
                onOpenChange={setShowNewDialog}
                onCreated={(id) => {
                    setSelectedThreadId(id)
                    refetchThreads()
                }}
            />
        </div>
    )
}

// ── 左側列表項 ──────────────────────────────

function ThreadListItem({ thread, selected, onClick }: {
    thread: ThreadSummary
    selected: boolean
    onClick: () => void
}) {
    const title = thread.subject ?? thread.participants.map(p => p.display_name).join(', ')
    return (
        <button
            type="button"
            onClick={onClick}
            className={`w-full text-left px-4 py-3 border-b hover:bg-muted transition-colors ${selected ? 'bg-muted' : ''}`}
        >
            <div className="flex items-start justify-between gap-2">
                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1">
                        {thread.type === 'group' && <Users className="h-3 w-3 text-muted-foreground shrink-0" />}
                        <span className="font-medium text-sm truncate">{title}</span>
                    </div>
                    {thread.last_message_preview && (
                        <p className="text-xs text-muted-foreground truncate mt-1">{thread.last_message_preview}</p>
                    )}
                    <p className="text-[10px] text-muted-foreground mt-1">
                        {format(new Date(thread.last_message_at), 'MM-dd HH:mm')}
                    </p>
                </div>
                {thread.unread_count > 0 && (
                    <span className="bg-primary text-primary-foreground text-[10px] px-1.5 py-0.5 rounded-full font-semibold">
                        {thread.unread_count}
                    </span>
                )}
            </div>
        </button>
    )
}

// ── 右側對話檢視 ──────────────────────────────

function ThreadView({ thread, onMessageSent }: { thread: ThreadWithMessages; onMessageSent: () => void }) {
    const { t } = useTranslation()
    const qc = useQueryClient()
    const [body, setBody] = useState('')
    const [pendingAttachments, setPendingAttachments] = useState<MessageAttachment[]>([])
    const messagesEndRef = useRef<HTMLDivElement>(null)

    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
    }, [thread.messages.length])

    const uploadAttachmentMutation = useMutation({
        mutationFn: async (file: File): Promise<MessageAttachment> => {
            const compressed = await compressImage(file)
            const fd = new FormData()
            fd.append('file', compressed)
            const res = await api.post<MessageAttachment>('/messages/attachments', fd, {
                headers: { 'Content-Type': 'multipart/form-data' },
            })
            return res.data
        },
        onSuccess: (att) => setPendingAttachments(prev => [...prev, att]),
        onError: (err) => toast({ title: t('common.error'), description: getApiErrorMessage(err, t('messagingPage.uploadFailed')), variant: 'destructive' }),
    })

    const sendMutation = useMutation({
        mutationFn: async () => {
            const res = await api.post<MessageWithSender>(`/messages/threads/${thread.id}`, {
                body: body.trim(),
                attachment_ids: pendingAttachments.map(a => a.id),
            })
            return res.data
        },
        onSuccess: () => {
            setBody('')
            setPendingAttachments([])
            onMessageSent()
            qc.invalidateQueries({ queryKey: ['messaging-threads'] })
        },
        onError: (err) => toast({ title: t('common.error'), description: getApiErrorMessage(err, t('messagingPage.sendFailed')), variant: 'destructive' }),
    })

    const deleteMessageMutation = useMutation({
        mutationFn: async (id: string) => api.delete(`/messages/${id}`),
        onSuccess: () => onMessageSent(),
    })

    const handleSelectFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const files = e.target.files
        if (!files) return
        for (const file of Array.from(files)) {
            uploadAttachmentMutation.mutate(file)
        }
        e.target.value = ''
    }

    // 附件上傳中不能送，否則孤兒附件 + 訊息缺圖
    const canSend =
        (body.trim().length > 0 || pendingAttachments.length > 0)
        && !sendMutation.isPending
        && !uploadAttachmentMutation.isPending

    const title = thread.subject ?? thread.participants.map(p => p.display_name).join(', ')

    return (
        <>
            {/* shrink-0：header 不被擠壓 */}
            <div className="px-4 py-3 border-b shrink-0">
                <h3 className="font-semibold flex items-center gap-2">
                    {thread.type === 'group' && <Users className="h-4 w-4 text-muted-foreground" />}
                    {title}
                </h3>
                <p className="text-xs text-muted-foreground">{t('messagingPage.participantsCount', { count: thread.participants.length })}</p>
            </div>

            {/* 訊息列：flex-1 overflow-y-auto 內部捲動；底部 padding 預留 composer 高度 */}
            <div className="flex-1 overflow-y-auto p-4 pb-32 space-y-3 bg-muted/30 min-h-0">
                {thread.messages.map(m => (
                    <MessageBubble key={m.id} message={m} onDelete={() => deleteMessageMutation.mutate(m.id)} />
                ))}
                <div ref={messagesEndRef} />
            </div>

            {/* composer 用 absolute 釘在 right col 底部 — 父層 relative 提供定位上下文 */}
            <div className="absolute bottom-0 left-0 right-0 z-10 border-t p-3 space-y-2 bg-background">
                {pendingAttachments.length > 0 && (
                    <div className="flex flex-wrap gap-2">
                        {pendingAttachments.map(a => (
                            <div key={a.id} className="flex items-center gap-1 bg-muted rounded px-2 py-1 text-xs">
                                <ImagePlus className="h-3 w-3" />
                                <span className="max-w-[150px] truncate">{a.file_name}</span>
                                <button
                                    type="button"
                                    onClick={() => setPendingAttachments(prev => prev.filter(x => x.id !== a.id))}
                                    className="hover:text-destructive"
                                >
                                    <X className="h-3 w-3" />
                                </button>
                            </div>
                        ))}
                    </div>
                )}
                <div className="flex gap-2">
                    <label className="flex items-center justify-center w-10 h-10 border rounded cursor-pointer hover:bg-muted shrink-0">
                        {uploadAttachmentMutation.isPending
                            ? <Loader2 className="h-4 w-4 animate-spin" />
                            : <ImagePlus className="h-4 w-4" />
                        }
                        <input
                            type="file"
                            accept="image/*"
                            capture="environment"
                            multiple
                            className="hidden"
                            onChange={handleSelectFile}
                            disabled={uploadAttachmentMutation.isPending}
                        />
                    </label>
                    <Textarea
                        value={body}
                        onChange={(e) => setBody(e.target.value)}
                        placeholder={t('messagingPage.messagePlaceholder')}
                        rows={2}
                        className="flex-1 resize-none"
                        onKeyDown={(e) => {
                            if (e.key === 'Enter' && (e.ctrlKey || e.metaKey) && canSend) {
                                e.preventDefault()
                                sendMutation.mutate()
                            }
                        }}
                    />
                    <Button onClick={() => sendMutation.mutate()} disabled={!canSend} className="shrink-0">
                        {sendMutation.isPending
                            ? <Loader2 className="h-4 w-4 animate-spin" />
                            : <Send className="h-4 w-4" />
                        }
                    </Button>
                </div>
                <p className="text-[10px] text-muted-foreground">{t('messagingPage.sendHint')}</p>
            </div>
        </>
    )
}

// ── 訊息氣泡 ──────────────────────────────

function MessageBubble({ message, onDelete }: { message: MessageWithSender; onDelete: () => void }) {
    const { t } = useTranslation()
    // R89-7：後端 soft_delete 只允許 sender 本人或 messaging.admin_view 刪，
    // 這裡的按鈕過去對所有訊息都顯示，其他人的訊息點下去必 403。
    const currentUser = useAuthUser()
    const hasPermission = useAuthHasPermission()
    const canDelete = message.sender_id === currentUser?.id || hasPermission(PERMISSIONS.MESSAGING_ADMIN_VIEW)

    return (
        <div className="bg-card border rounded p-3 max-w-[80%]">
            <div className="flex items-center justify-between gap-2">
                <span className="font-medium text-xs">{message.sender_display_name}</span>
                <div className="flex items-center gap-2">
                    <span className="text-[10px] text-muted-foreground">
                        {format(new Date(message.created_at), 'MM-dd HH:mm')}
                    </span>
                    {canDelete && (
                        <button
                            type="button"
                            onClick={onDelete}
                            className="text-muted-foreground hover:text-destructive"
                            title={t('messagingPage.deleteTitle')}
                        >
                            <Trash2 className="h-3 w-3" />
                        </button>
                    )}
                </div>
            </div>
            {message.body && (
                <p className="text-sm whitespace-pre-wrap mt-1">{message.body}</p>
            )}
            {message.attachments.length > 0 && (
                <div className="grid grid-cols-2 gap-2 mt-2">
                    {message.attachments.map(a => (
                        <a
                            key={a.id}
                            href={`/api/v1/messages/attachments/${a.id}/download?inline=1`}
                            target="_blank"
                            rel="noreferrer"
                            className="block"
                        >
                            <img
                                src={`/api/v1/messages/attachments/${a.id}/download?inline=1`}
                                alt={a.file_name}
                                className="w-full h-32 object-cover rounded bg-muted"
                                loading="lazy"
                            />
                        </a>
                    ))}
                </div>
            )}
        </div>
    )
}

// ── 新增對話 dialog ──────────────────────────────

function NewThreadDialog({ open, onOpenChange, onCreated }: {
    open: boolean
    onOpenChange: (v: boolean) => void
    onCreated: (id: string) => void
}) {
    const { t } = useTranslation()
    const [recipientIds, setRecipientIds] = useState<string[]>([])
    const [subject, setSubject] = useState('')
    const [body, setBody] = useState('')
    const [pendingAttachments, setPendingAttachments] = useState<MessageAttachment[]>([])

    // 收件人來源：messaging 專用 endpoint，依 access matrix 回傳可寄送對象
    //（含 admin / PI / VET 等全部 messaging-eligible 角色，非僅 EXPERIMENT_STAFF）
    const { data: staffList = [] } = useQuery({
        queryKey: ['messaging-recipients'],
        queryFn: async () => {
            const res = await api.get<StaffOption[]>('/messages/recipients')
            return res.data
        },
        staleTime: 5 * 60_000,
        enabled: open,
    })

    const staffOptions = useMemo(() => staffList.map(s => ({
        value: s.id,
        label: s.display_name,
        description: s.email,
    })), [staffList])

    const isGroup = recipientIds.length > 1

    const uploadAttachmentMutation = useMutation({
        mutationFn: async (file: File): Promise<MessageAttachment> => {
            const compressed = await compressImage(file)
            const fd = new FormData()
            fd.append('file', compressed)
            const res = await api.post<MessageAttachment>('/messages/attachments', fd, {
                headers: { 'Content-Type': 'multipart/form-data' },
            })
            return res.data
        },
        onSuccess: (att) => setPendingAttachments(prev => [...prev, att]),
        onError: (err) => toast({ title: t('common.error'), description: getApiErrorMessage(err, t('messagingPage.uploadFailed')), variant: 'destructive' }),
    })

    const canCreate = recipientIds.length > 0
        && body.trim().length > 0
        && !uploadAttachmentMutation.isPending

    const createMutation = useMutation({
        mutationFn: async () => {
            const res = await api.post<ThreadWithMessages>('/messages/threads', {
                type: isGroup ? 'group' : 'direct',
                recipient_ids: recipientIds,
                subject: isGroup ? (subject.trim() || null) : null,
                body: body.trim(),
                attachment_ids: pendingAttachments.map(a => a.id),
            })
            return res.data
        },
        onSuccess: (created) => {
            toast({ title: t('messagingPage.created') })
            onCreated(created.id)
            setRecipientIds([])
            setSubject('')
            setBody('')
            setPendingAttachments([])
            onOpenChange(false)
        },
        onError: (err) => toast({ title: t('common.error'), description: getApiErrorMessage(err, t('messagingPage.createFailed')), variant: 'destructive' }),
    })

    const handleSelectFile = (e: React.ChangeEvent<HTMLInputElement>) => {
        const files = e.target.files
        if (!files) return
        for (const file of Array.from(files)) {
            uploadAttachmentMutation.mutate(file)
        }
        e.target.value = ''
    }

    useEffect(() => {
        if (!open) {
            setRecipientIds([])
            setSubject('')
            setBody('')
            setPendingAttachments([])
        }
    }, [open])

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent size="lg" className="max-h-[90vh] overflow-y-auto">
                <DialogHeader>
                    <DialogTitle>{t('messagingPage.newDialog.title')}</DialogTitle>
                </DialogHeader>

                <div className="space-y-4">
                    <div>
                        <label className="text-sm font-medium block mb-1">{t('messagingPage.newDialog.recipients')}</label>
                        <SearchableSelect
                            options={staffOptions.filter(o => !recipientIds.includes(o.value))}
                            value=""
                            onValueChange={(v) => v && setRecipientIds(prev => [...prev, v])}
                            placeholder={t('messagingPage.newDialog.recipientsPlaceholder')}
                            searchPlaceholder={t('messagingPage.newDialog.recipientsSearchPlaceholder')}
                            emptyMessage={t('messagingPage.newDialog.recipientsEmpty')}
                        />
                        {recipientIds.length > 0 && (
                            <div className="flex flex-wrap gap-2 mt-2">
                                {recipientIds.map(id => {
                                    const s = staffList.find(x => x.id === id)
                                    return (
                                        <div key={id} className="flex items-center gap-1 bg-muted rounded px-2 py-1 text-xs">
                                            {s?.display_name ?? id}
                                            <button
                                                type="button"
                                                onClick={() => setRecipientIds(prev => prev.filter(x => x !== id))}
                                                className="hover:text-destructive"
                                            >
                                                <X className="h-3 w-3" />
                                            </button>
                                        </div>
                                    )
                                })}
                            </div>
                        )}
                        <p className="text-[10px] text-muted-foreground mt-1">
                            {t('messagingPage.newDialog.recipientsHint')}
                        </p>
                    </div>

                    {isGroup && (
                        <div>
                            <label className="text-sm font-medium block mb-1">{t('messagingPage.newDialog.groupSubject')}</label>
                            <Input value={subject} onChange={(e) => setSubject(e.target.value)} placeholder={t('messagingPage.newDialog.groupSubjectPlaceholder')} />
                        </div>
                    )}

                    <div>
                        <label className="text-sm font-medium block mb-1">{t('messagingPage.newDialog.firstMessage')}</label>
                        <Textarea value={body} onChange={(e) => setBody(e.target.value)} rows={6} placeholder={t('messagingPage.newDialog.firstMessagePlaceholder')} />
                    </div>

                    <div>
                        <div className="flex items-center justify-between mb-2">
                            <label className="text-sm font-medium">{t('messagingPage.newDialog.attachments')}</label>
                            <label className="flex items-center gap-1 text-xs text-status-success-solid hover:text-green-700 cursor-pointer">
                                {uploadAttachmentMutation.isPending
                                    ? <Loader2 className="h-3.5 w-3.5 animate-spin" />
                                    : <ImagePlus className="h-3.5 w-3.5" />
                                }
                                {t('messagingPage.newDialog.attachImages')}
                                <input
                                    type="file"
                                    accept="image/*"
                                    capture="environment"
                                    multiple
                                    className="hidden"
                                    onChange={handleSelectFile}
                                    disabled={uploadAttachmentMutation.isPending}
                                />
                            </label>
                        </div>
                        {pendingAttachments.length > 0 && (
                            <div className="grid grid-cols-3 gap-3">
                                {pendingAttachments.map(a => (
                                    <div key={a.id} className="relative border rounded p-1">
                                        <img
                                            src={`/api/v1/messages/attachments/${a.id}/download?inline=1`}
                                            alt={a.file_name}
                                            className="w-full h-32 object-cover rounded bg-muted"
                                        />
                                        <button
                                            type="button"
                                            onClick={() => setPendingAttachments(prev => prev.filter(x => x.id !== a.id))}
                                            className="absolute top-1 right-1 bg-background/80 hover:bg-destructive hover:text-destructive-foreground rounded-full p-1"
                                            title={t('messagingPage.newDialog.remove')}
                                        >
                                            <X className="h-3 w-3" />
                                        </button>
                                        <p className="text-[10px] text-muted-foreground truncate mt-1">{a.file_name}</p>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>

                <div className="flex justify-end gap-2 pt-4 border-t mt-4">
                    <Button variant="outline" onClick={() => onOpenChange(false)} disabled={createMutation.isPending}>{t('common.cancel')}</Button>
                    <Button onClick={() => createMutation.mutate()} disabled={!canCreate || createMutation.isPending}>
                        {createMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin mr-1" /> : <Send className="h-4 w-4 mr-1" />}
                        {t('messagingPage.newDialog.submit')}
                    </Button>
                </div>
            </DialogContent>
        </Dialog>
    )
}
