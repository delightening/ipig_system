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
          {attachmentsLoading ? (
            <div className="flex justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : attachments && attachments.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t('protocols.detail.tables.fileName')}</TableHead>
                  <TableHead>{t('protocols.detail.tables.size')}</TableHead>
                  <TableHead>{t('protocols.detail.tables.uploadedBy')}</TableHead>
                  <TableHead>{t('protocols.detail.tables.uploadTime')}</TableHead>
                  <TableHead className="text-right">{t('protocols.detail.tables.actions')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {attachments.map((attachment) => (
                  <TableRow key={attachment.id}>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <Paperclip className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">{attachment.file_name}</span>
                      </div>
                    </TableCell>
                    <TableCell>{formatFileSize(attachment.file_size)}</TableCell>
                    <TableCell>{attachment.uploaded_by_name || '-'}</TableCell>
                    <TableCell>{formatDateTime(attachment.created_at)}</TableCell>
                    <TableCell>
                      <div className="flex items-center justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => downloadAttachmentMutation.mutate(attachment)}
                          disabled={downloadAttachmentMutation.isPending}
                        >
                          {downloadAttachmentMutation.isPending ? (
                            <Loader2 className="h-4 w-4 animate-spin" />
                          ) : (
                            <Download className="h-4 w-4" />
                          )}
                        </Button>
                        {canManageAttachments && (
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={async () => {
                              const ok = await confirm({
                                title: '刪除附件',
                                description: t('protocols.detail.actions.deleteConfirm'),
                                variant: 'destructive',
                                confirmLabel: '確認刪除',
                              })
                              if (ok) {
                                deleteAttachmentMutation.mutate(attachment.id)
                              }
                            }}
                            disabled={deleteAttachmentMutation.isPending}
                          >
                            <Trash2 className="h-4 w-4 text-destructive" />
                          </Button>
                        )}
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              <Paperclip className="h-12 w-12 mx-auto mb-2" />
              <p>{t('protocols.detail.tables.noAttachments')}</p>
              {canManageAttachments && (
                <Button variant="link" onClick={() => fileInputRef.current?.click()} className="mt-2">
                  {t('protocols.detail.tables.uploadFirst')}
                </Button>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      <ConfirmDialog state={dialogState} />
    </>
  )
}
