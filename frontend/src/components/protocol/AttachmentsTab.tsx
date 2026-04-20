import React, { useRef } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource, ProtocolAttachment } from '@/lib/api'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { toast } from '@/components/ui/use-toast'
import { Download, Loader2, Paperclip, Trash2, Upload } from 'lucide-react'
import { formatDateTime, formatFileSize } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

interface AttachmentsTabProps {
  protocolId: string
  canManageAttachments: boolean
}

export function AttachmentsTab({ protocolId, canManageAttachments }: AttachmentsTabProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()
  const fileInputRef = useRef<HTMLInputElement>(null)

  const { data: attachments, isLoading: attachmentsLoading } = useQuery({
    queryKey: ['protocol-attachments', protocolId],
    queryFn: async () => {
      const response = await api.get<ProtocolAttachment[]>('/attachments', {
        params: { entity_type: 'protocol', entity_id: protocolId },
      })
      return response.data
    },
    enabled: !!protocolId,
  })

  const { sortedData: sortedAttachments, sort, toggleSort } = useTableSort(attachments)

  const uploadAttachmentMutation = useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData()
      formData.append('file', file)
      return api.post(`/protocols/${protocolId}/attachments`, formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
      })
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.tables.uploadSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol-attachments', protocolId] })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.tables.uploadFailed')),
        variant: 'destructive',
      })
    },
  })

  const deleteAttachmentMutation = useMutation({
    mutationFn: async (attachmentId: string) => {
      return deleteResource(`/attachments/${attachmentId}`)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.actions.deleteSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol-attachments', protocolId] })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.actions.deleteFailed')),
        variant: 'destructive',
      })
    },
  })

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      uploadAttachmentMutation.mutate(file)
    }
    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }

  const downloadAttachmentMutation = useMutation({
    mutationFn: async (attachment: ProtocolAttachment) => {
      const response = await api.get(`/attachments/${attachment.id}/download`, {
        responseType: 'blob',
      })
      const url = window.URL.createObjectURL(new Blob([response.data]))
      const link = document.createElement('a')
      link.href = url
      link.setAttribute('download', attachment.file_name)
      document.body.appendChild(link)
      link.click()
      link.remove()
    },
    onError: () => {
      toast({ title: t('common.error'), description: t('common.downloadFailed'), variant: 'destructive' })
    },
  })

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>{t('protocols.detail.sections.attachmentsTitle')}</CardTitle>
            <CardDescription>{t('protocols.detail.sections.attachmentsDesc')}</CardDescription>
          </div>
          {canManageAttachments && (
            <>
              <input
                type="file"
                ref={fileInputRef}
                onChange={handleFileUpload}
                className="hidden"
                aria-label={t('protocols.detail.sections.attachmentsTitle')}
              />
              <Button onClick={() => fileInputRef.current?.click()} disabled={uploadAttachmentMutation.isPending}>
                {uploadAttachmentMutation.isPending ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <Upload className="mr-2 h-4 w-4" />
                )}
                {t('protocols.detail.tables.upload')}
              </Button>
            </>
          )}
        </CardHeader>
        <CardContent>
          <div className="@container">
            <div className="hidden @[600px]:block overflow-x-auto">
              <Table className="w-full" style={{ minWidth: 570 }}>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <SortableTableHead style={{ minWidth: 150 }} sortKey="file_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.detail.tables.fileName')}</SortableTableHead>
                    <SortableTableHead style={{ width: 80 }} sortKey="file_size" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.detail.tables.size')}</SortableTableHead>
                    <TableHead style={{ width: 100 }}>{t('protocols.detail.tables.uploadedBy')}</TableHead>
                    <SortableTableHead style={{ width: 160 }} sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.detail.tables.uploadTime')}</SortableTableHead>
                    <TableHead style={{ width: 80 }} className="text-right">{t('protocols.detail.tables.actions')}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {attachmentsLoading ? (
                    <TableRow>
                      <TableCell colSpan={5} className="p-0">
                        <TableSkeleton rows={3} cols={5} />
                      </TableCell>
                    </TableRow>
                  ) : sortedAttachments && sortedAttachments.length > 0 ? (
                    sortedAttachments.map((attachment) => (
                      <TableRow key={attachment.id}>
                        <TableCell style={{ minWidth: 150 }}>
                          <div className="flex items-center gap-2">
                            <Paperclip className="h-4 w-4 text-muted-foreground shrink-0" />
                            <span className="font-medium break-all">{attachment.file_name}</span>
                          </div>
                        </TableCell>
                        <TableCell style={{ width: 80 }}>{formatFileSize(attachment.file_size)}</TableCell>
                        <TableCell style={{ width: 100 }} className="whitespace-normal break-words">{attachment.uploaded_by_name || '-'}</TableCell>
                        <TableCell style={{ width: 160 }} className="text-xs text-muted-foreground">{formatDateTime(attachment.created_at)}</TableCell>
                        <TableCell style={{ width: 80 }}>
                          <div className="flex items-center justify-end gap-1">
                            <Button variant="ghost" size="icon" onClick={() => downloadAttachmentMutation.mutate(attachment)} disabled={downloadAttachmentMutation.isPending} title="下載">
                              {downloadAttachmentMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Download className="h-4 w-4" />}
                            </Button>
                            {canManageAttachments && (
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={async () => {
                                  const ok = await confirm({
                                    title: '刪除附件',
                                    description: t('protocols.detail.actions.deleteConfirm'),
                                    variant: 'destructive',
                                    confirmLabel: '確認刪除',
                                  })
                                  if (ok) deleteAttachmentMutation.mutate(attachment.id)
                                }}
                                disabled={deleteAttachmentMutation.isPending}
                                title="刪除"
                              >
                                <Trash2 className="h-4 w-4 text-destructive" />
                              </Button>
                            )}
                          </div>
                        </TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <TableEmptyRow colSpan={5} icon={Paperclip} title={t('protocols.detail.tables.noAttachments')} />
                  )}
                </TableBody>
              </Table>
            </div>

            <div className="@[600px]:hidden space-y-3 py-1">
              {attachmentsLoading ? (
                <TableSkeleton rows={3} cols={1} />
              ) : sortedAttachments && sortedAttachments.length > 0 ? (
                sortedAttachments.map((attachment) => (
                  <div key={attachment.id} className="rounded-lg border bg-card p-3 space-y-2">
                    <div className="flex items-start gap-2 text-sm font-medium text-foreground break-all">
                      <Paperclip className="h-4 w-4 shrink-0 mt-0.5 text-muted-foreground" />
                      <span>{attachment.file_name}</span>
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {formatDateTime(attachment.created_at)}
                      <span className="mx-1.5">·</span>
                      {formatFileSize(attachment.file_size)}
                      {attachment.uploaded_by_name && <><span className="mx-1.5">·</span>{attachment.uploaded_by_name}</>}
                    </div>
                    <div className="flex justify-end gap-1 pt-1 border-t">
                      <Button variant="ghost" size="icon" onClick={() => downloadAttachmentMutation.mutate(attachment)} disabled={downloadAttachmentMutation.isPending} title="下載">
                        {downloadAttachmentMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Download className="h-4 w-4" />}
                      </Button>
                      {canManageAttachments && (
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={async () => {
                            const ok = await confirm({
                              title: '刪除附件',
                              description: t('protocols.detail.actions.deleteConfirm'),
                              variant: 'destructive',
                              confirmLabel: '確認刪除',
                            })
                            if (ok) deleteAttachmentMutation.mutate(attachment.id)
                          }}
                          disabled={deleteAttachmentMutation.isPending}
                          title="刪除"
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      )}
                    </div>
                  </div>
                ))
              ) : (
                <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                  <Paperclip className="h-8 w-8" />
                  <p className="text-sm">{t('protocols.detail.tables.noAttachments')}</p>
                </div>
              )}
            </div>

          </div>
        </CardContent>
      </Card>

      <ConfirmDialog state={dialogState} />
    </>
  )
}
