import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { GuestHide } from '@/components/ui/guest-hide'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { AnimalPathologyReport } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { FileUpload, FileInfo } from '@/components/ui/file-upload'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Upload, Download, FileText, Loader2 } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { formatFileSize, uiLocale } from '@/lib/utils'

interface PathologyTabProps {
  animalId: string
  earTag: string
}

export function PathologyTab({ animalId, earTag }: PathologyTabProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const [showUploadDialog, setShowUploadDialog] = useState(false)
  const [files, setFiles] = useState<FileInfo[]>([])

  const { data: pathology } = useQuery({
    queryKey: ['animal-pathology', animalId],
    queryFn: async () => {
      const res = await api.get<AnimalPathologyReport>(`/animals/${animalId}/pathology`)
      return res.data
    },
    staleTime: 30_000,
  })

  const { sortedData: sortedAttachments, sort, toggleSort } = useTableSort(pathology?.attachments)

  const uploadMutation = useMutation({
    mutationFn: async (uploadFiles: FileInfo[]) => {
      return api.post(`/animals/${animalId}/pathology/upload`, {
        files: uploadFiles.map((f) => ({
          file_name: f.file_name,
          file_size: f.file_size,
        })),
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-pathology', animalId] })
      toast({ title: t('common.success'), description: t('animalRecords.pathology.uploaded') })
      setShowUploadDialog(false)
      setFiles([])
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalRecords.pathology.uploadFailed')),
        variant: 'destructive',
      })
    },
  })

  return (
    <>
      <Card className="overflow-hidden">
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>{t('animalDetail.tabs.pathology')}</CardTitle>
            <CardDescription>{t('animalRecords.pathology.description')}</CardDescription>
          </div>
          <GuestHide>
            <Can permission={PERMISSIONS.ANIMAL_PATHOLOGY_UPLOAD}>
              <Button className="bg-status-purple-solid hover:bg-status-purple-solid/90" onClick={() => setShowUploadDialog(true)}>
                <Upload className="h-4 w-4 mr-2" />
                {t('animalRecords.pathology.uploadFile')}
              </Button>
            </Can>
          </GuestHide>
        </CardHeader>
        <CardContent>
          <div className="@container">

            {/* ── Table view: container ≥ 600px ── */}
            <div className="hidden @[600px]:block overflow-x-auto">
              <Table className="w-full" style={{ minWidth: 448 }}>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <SortableTableHead sortKey="file_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} style={{ minWidth: 150 }}>{t('animalRecords.pathology.fileName')}</SortableTableHead>
                    <SortableTableHead sortKey="file_size" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} style={{ width: 90 }}>{t('animalRecords.pathology.fileSize')}</SortableTableHead>
                    <SortableTableHead sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} style={{ width: 160 }}>{t('animalRecords.pathology.uploadedAt')}</SortableTableHead>
                    <TableHead style={{ width: 48 }} className="text-right">{t('common.actions')}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {!pathology || !pathology.attachments || pathology.attachments.length === 0 ? (
                    <TableEmptyRow colSpan={4} icon={FileText} title={t('animalRecords.pathology.emptyTitle')} />
                  ) : (
                    (sortedAttachments ?? pathology.attachments).map((file) => (
                      <TableRow key={file.id}>
                        <TableCell style={{ minWidth: 150 }} className="font-medium whitespace-normal break-words">{file.file_name}</TableCell>
                        <TableCell style={{ width: 90 }}>{formatFileSize(file.file_size)}</TableCell>
                        <TableCell style={{ width: 160 }} className="text-xs text-muted-foreground">{new Date(file.created_at).toLocaleString(uiLocale(), { timeZone: 'Asia/Taipei' })}</TableCell>
                        <TableCell style={{ width: 48 }} className="text-right">
                          <Button variant="outline" size="icon" title={t('animalRecords.pathology.download')}>
                            <Download className="h-4 w-4" />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>

            {/* ── Card view: container < 600px ── */}
            <div className="@[600px]:hidden space-y-3 py-1">
              {!pathology || !pathology.attachments || pathology.attachments.length === 0 ? (
                <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                  <FileText className="h-8 w-8" />
                  <p className="text-sm">{t('animalRecords.pathology.emptyTitle')}</p>
                </div>
              ) : (
                (sortedAttachments ?? pathology.attachments).map((file) => (
                  <div key={file.id} className="rounded-lg border bg-card p-3 space-y-2">
                    <div className="flex items-start gap-2 text-sm font-medium text-foreground break-all">
                      <FileText className="h-4 w-4 shrink-0 mt-0.5 text-muted-foreground" />
                      <span>{file.file_name}</span>
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {new Date(file.created_at).toLocaleString(uiLocale(), { timeZone: 'Asia/Taipei' })}
                      <span className="mx-1.5">·</span>
                      {formatFileSize(file.file_size)}
                    </div>
                    <div className="flex justify-end pt-1 border-t">
                      <Button variant="outline" size="icon" title={t('animalRecords.pathology.download')}>
                        <Download className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                ))
              )}
            </div>

          </div>
        </CardContent>
      </Card>

      <Dialog open={showUploadDialog} onOpenChange={setShowUploadDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('animalRecords.pathology.uploadTitle')}</DialogTitle>
            <DialogDescription>{t('animalRecords.shared.earTagLine', { earTag })}</DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <FileUpload
              value={files}
              onChange={setFiles}
              accept=".pdf,.doc,.docx,.jpg,.jpeg,.png,.tiff"
              placeholder={t('animalRecords.pathology.dropPlaceholder')}
              maxSize={50}
              maxFiles={20}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowUploadDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={() => uploadMutation.mutate(files)}
              disabled={uploadMutation.isPending || files.length === 0}
              className="bg-status-success-solid hover:bg-status-success-solid/90"
            >
              {uploadMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('animalRecords.pathology.upload')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
