import React, { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { GuestHide } from '@/components/ui/guest-hide'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource, AnimalSurgery } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { uiLocale } from '@/lib/utils'
import {
  Plus,
  Eye,
  Edit2,
  Trash2,
  History,
  CheckCircle2,
  Scissors,
  ChevronDown,
  Copy,
  FileDown,
} from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { SurgeryFormDialog } from './SurgeryFormDialog'
import { POSTURE_OPTIONS } from './useSurgeryForm'
import { VersionHistoryDialog } from './VersionHistoryDialog'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'

interface SurgeriesTabProps {
  animalId: string
  earTag: string
  afterParam: string
  surgeries: AnimalSurgery[] | undefined
}

export const SurgeriesTab = React.memo(function SurgeriesTab({ animalId, earTag, afterParam: _afterParam, surgeries }: SurgeriesTabProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { sortedData, sort, toggleSort } = useTableSort(surgeries)

  const [showAddDialog, setShowAddDialog] = useState(false)
  const [editingSurgery, setEditingSurgery] = useState<AnimalSurgery | null>(null)
  const [expandedId, setExpandedId] = useState<number | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<number | null>(null)

  const [versionHistoryRecordId, setVersionHistoryRecordId] = useState<number | null>(null)
  const [showVersionHistory, setShowVersionHistory] = useState(false)

  const deleteMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: number; reason: string }) => {
      return deleteResource(`/surgeries/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-surgeries', animalId] })
      toast({ title: t('common.success'), description: t('animalRecords.surgeries.deleted') })
      setDeleteTarget(null)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalRecords.shared.deleteFailed')),
        variant: 'destructive',
      })
    },
  })

  const copyMutation = useMutation({
    mutationFn: async (sourceId: number) => {
      return api.post(`/animals/${animalId}/surgeries/copy`, { source_id: sourceId })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-surgeries', animalId] })
      toast({ title: t('common.success'), description: t('animalRecords.surgeries.copied') })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalRecords.shared.copyFailed')),
        variant: 'destructive',
      })
    },
  })

  // R32-A8b：下載手術紀錄 PDF（v3 docx → Word COM）
  const downloadSurgeryPdf = async (surgeryId: string | number, surgeryDate: string) => {
    try {
      const response = await api.get(`/surgeries/${surgeryId}/export-pdf-v3?format=pdf`, {
        responseType: 'blob',
        _silentError: true,
        timeout: 120_000,
      })
      const blob = new Blob([response.data], { type: 'application/pdf' })
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `surgery_${earTag}_${surgeryDate}.pdf`
      a.click()
      window.URL.revokeObjectURL(url)
    } catch (error) {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalRecords.surgeries.pdfExportFailed')),
        variant: 'destructive',
      })
    }
  }

  return (
    <>
      <Card className="overflow-hidden">
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>{t('animalDetail.tabs.surgeries')}</CardTitle>
            <CardDescription>{t('animalRecords.surgeries.description')}</CardDescription>
          </div>
          <GuestHide>
            <Can permission={PERMISSIONS.ANIMAL_RECORD_CREATE}>
              <Button className="bg-status-purple-solid hover:bg-status-purple-solid/90" onClick={() => setShowAddDialog(true)}>
                <Plus className="h-4 w-4 mr-2" />
                {t('animalRecords.shared.addRecord')}
              </Button>
            </Can>
          </GuestHide>
        </CardHeader>
        <CardContent>
          <div className="@container">

            {/* ── Table view: container ≥ 600px ── */}
            <div className="hidden @[600px]:block overflow-x-auto">
              <Table className="w-full" style={{ minWidth: 540 }}>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <TableHead style={{ width: 40 }}></TableHead>
                    <TableHead style={{ width: 70 }} className="text-center">{t('animalRecords.surgeries.isFirst')}</TableHead>
                    <SortableTableHead style={{ width: 100 }} sortKey="surgery_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('animalRecords.surgeries.surgeryDate')}</SortableTableHead>
                    <SortableTableHead style={{ minWidth: 150 }} sortKey="surgery_site" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('animalRecords.surgeries.surgerySite')}</SortableTableHead>
                    <TableHead style={{ width: 60 }} className="text-center hidden @[690px]:table-cell">{t('animalRecords.shared.stopMedication')}</TableHead>
                    <TableHead style={{ width: 110 }} className="text-center whitespace-nowrap">{t('animalRecords.shared.vetReadHeader')}</TableHead>
                    <TableHead style={{ width: 90 }} className="hidden @[690px]:table-cell">{t('animalRecords.shared.recorder')}</TableHead>
                    <TableHead style={{ width: 160, minWidth: 160 }} className="sticky right-0 bg-card border-l text-center px-1 py-2">{t('common.actions')}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {!surgeries || surgeries.length === 0 ? (
                    <TableEmptyRow colSpan={8} icon={Scissors} title={t('animalRecords.surgeries.emptyTitle')} />
                  ) : (
                    sortedData?.map((surgery) => (
                      <React.Fragment key={surgery.id}>
                        <TableRow className="group cursor-pointer hover:bg-muted">
                          <TableCell style={{ width: 40 }}>
                            <button
                              type="button"
                              onClick={() => setExpandedId(expandedId === surgery.id ? null : surgery.id)}
                              className="p-1 hover:bg-muted rounded"
                              title={t('animalRecords.shared.expandDetails')}
                              aria-label={t('animalRecords.shared.expandDetails')}
                            >
                              <ChevronDown
                                className={`h-4 w-4 transition-transform ${expandedId === surgery.id ? 'rotate-180' : ''}`}
                              />
                            </button>
                          </TableCell>
                          <TableCell style={{ width: 70 }} className="text-center">
                            {surgery.is_first_experiment
                              ? <Badge className="bg-status-warning-bg text-status-warning-text">{t('animalRecords.surgeries.firstBadge')}</Badge>
                              : <span className="text-muted-foreground text-sm">{t('common.no')}</span>}
                          </TableCell>
                          <TableCell style={{ width: 100 }} className="whitespace-nowrap">{new Date(surgery.surgery_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })}</TableCell>
                          <TableCell style={{ minWidth: 150 }} className="whitespace-normal break-words">{surgery.surgery_site}</TableCell>
                          <TableCell style={{ width: 60 }} className="text-center hidden @[690px]:table-cell">
                            {surgery.no_medication_needed ? (
                              <CheckCircle2 className="h-4 w-4 text-status-success-solid inline-block" />
                            ) : (
                              <span className="text-muted-foreground">-</span>
                            )}
                          </TableCell>
                          <TableCell style={{ width: 110 }} className="text-center">
                            {surgery.vet_read ? (
                              <Badge className="bg-status-success-bg text-status-success-text">{t('animalRecords.shared.read')}</Badge>
                            ) : (
                              <Badge variant="outline" className="text-muted-foreground">{t('animalRecords.shared.unread')}</Badge>
                            )}
                          </TableCell>
                          <TableCell style={{ width: 90 }} className="whitespace-normal break-words hidden @[690px]:table-cell">{surgery.created_by_name || '-'}</TableCell>
                          <TableCell style={{ width: 160, minWidth: 160 }} className="px-1 py-1 sticky right-0 bg-card group-hover:bg-muted border-l">
                            <div className="grid grid-cols-4 gap-0.5 justify-items-center">
                              <Button variant="ghost" size="icon" onClick={() => setExpandedId(surgery.id)} title={t('animalRecords.shared.viewDetails')} aria-label={t('animalRecords.shared.viewDetails')}>
                                <Eye className="h-4 w-4" />
                              </Button>
                              <GuestHide>
                                <Can permission={PERMISSIONS.ANIMAL_RECORD_EDIT}>
                                  <Button variant="ghost" size="icon" onClick={() => { setEditingSurgery(surgery); setShowAddDialog(true) }} title={t('common.edit')}>
                                    <Edit2 className="h-4 w-4" />
                                  </Button>
                                </Can>
                                <Can permission={PERMISSIONS.ANIMAL_RECORD_COPY}>
                                  <Button variant="ghost" size="icon" onClick={() => { if (confirm(t('animalRecords.shared.copyConfirm'))) copyMutation.mutate(surgery.id) }} disabled={copyMutation.isPending} title={t('animalRecords.shared.copy')}>
                                    <Copy className="h-4 w-4" />
                                  </Button>
                                </Can>
                                <Button variant="ghost" size="icon" onClick={() => { setVersionHistoryRecordId(surgery.id); setShowVersionHistory(true) }} title={t('animalRecords.shared.versionHistory')}>
                                  <History className="h-4 w-4" />
                                </Button>
                                <Can permission={PERMISSIONS.ANIMAL_EXPORT_SURGERY}>
                                  <Button variant="ghost" size="icon" onClick={() => downloadSurgeryPdf(surgery.id, surgery.surgery_date)} title={t('common.pdfExport.downloadPdf')} aria-label={t('common.pdfExport.downloadPdf')}>
                                    <FileDown className="h-4 w-4" />
                                  </Button>
                                </Can>
                                <Can permission={PERMISSIONS.ANIMAL_RECORD_DELETE}>
                                  <Button variant="ghost" size="icon" onClick={() => setDeleteTarget(surgery.id)} title={t('common.delete')} aria-label={t('common.delete')}>
                                    <Trash2 className="h-4 w-4 text-status-error-solid" />
                                  </Button>
                                </Can>
                              </GuestHide>
                            </div>
                          </TableCell>
                        </TableRow>
                        {expandedId === surgery.id && (
                          <TableRow>
                            <TableCell colSpan={8} className="bg-muted p-4">
                          <div className="grid grid-cols-3 gap-4">
                            <div>
                              <Label className="text-muted-foreground">{t('animalRecords.surgeries.inductionAnesthesia')}</Label>
                              <p>
                                {surgery.induction_anesthesia
                                  ? Object.entries(surgery.induction_anesthesia as Record<string, string>)
                                    .filter(([k]) => k !== 'others')
                                    .map(([k, v]) => `${k}: ${v}`)
                                    .join(', ') || '-'
                                  : '-'}
                              </p>
                            </div>
                            <div>
                              <Label className="text-muted-foreground">{t('animalRecords.surgeries.anesthesiaMaintenance')}</Label>
                              <p>
                                {surgery.anesthesia_maintenance
                                  ? Object.entries(surgery.anesthesia_maintenance as Record<string, string>)
                                    .filter(([k]) => k !== 'others')
                                    .map(([k, v]) => `${k}: ${v}`)
                                    .join(', ') || '-'
                                  : '-'}
                              </p>
                            </div>
                            <div>
                              <Label className="text-muted-foreground">{t('animalRecords.surgeries.positioning')}</Label>
                              <p>{surgery.positioning
                                ? surgery.positioning
                                  .split(',')
                                  .map((value) => {
                                    const posture = POSTURE_OPTIONS.find((p) => p.value === value)
                                    return posture ? t(posture.labelKey) : value
                                  })
                                  .join(t('animalRecords.shared.listSeparator'))
                                : '-'}</p>
                            </div>
                            {surgery.anesthesia_observation && (
                              <div className="col-span-3">
                                <Label className="text-muted-foreground">{t('animalRecords.surgeries.anesthesiaObservation')}</Label>
                                <p className="whitespace-pre-wrap">{surgery.anesthesia_observation}</p>
                              </div>
                            )}
                            {surgery.vital_signs && surgery.vital_signs.length > 0 && (
                              <div className="col-span-3">
                                <Label className="text-muted-foreground">{t('animalRecords.surgeries.vitalSigns')}</Label>
                                <div className="mt-2 overflow-x-auto">
                                  <Table className="min-w-full text-sm">
                                    <TableHeader>
                                      <TableRow className="border-b">
                                        <TableHead className="px-2 py-1 text-left">{t('animalRecords.surgeries.vitalTime')}</TableHead>
                                        <TableHead className="px-2 py-1 text-left">{t('animalRecords.surgeries.heartRate')}</TableHead>
                                        <TableHead className="px-2 py-1 text-left">{t('animalRecords.surgeries.respiration')}</TableHead>
                                        <TableHead className="px-2 py-1 text-left">{t('animalRecords.surgeries.temperature')}</TableHead>
                                        <TableHead className="px-2 py-1 text-left">SPO2</TableHead>
                                      </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                      {surgery.vital_signs.map((vs, i) => (
                                        <TableRow key={i} className="border-b">
                                          <TableCell className="px-2 py-1">{vs.time}</TableCell>
                                          <TableCell className="px-2 py-1">{t('animalRecords.surgeries.perMinute', { value: vs.heart_rate })}</TableCell>
                                          <TableCell className="px-2 py-1">{t('animalRecords.surgeries.perMinute', { value: vs.respiration_rate })}</TableCell>
                                          <TableCell className="px-2 py-1">{vs.temperature}°C</TableCell>
                                          <TableCell className="px-2 py-1">{vs.spo2}%</TableCell>
                                        </TableRow>
                                      ))}
                                    </TableBody>
                                  </Table>
                                </div>
                              </div>
                            )}
                            {surgery.reflex_recovery && (
                              <div className="col-span-3">
                                <Label className="text-muted-foreground">{t('animalRecords.surgeries.reflexRecovery')}</Label>
                                <p>{surgery.reflex_recovery}</p>
                              </div>
                            )}
                            {surgery.remark && (
                              <div className="col-span-3">
                                <Label className="text-muted-foreground">{t('animalRecords.shared.remark')}</Label>
                                <p>{surgery.remark}</p>
                              </div>
                            )}
                          </div>
                        </TableCell>
                      </TableRow>
                    )}
                  </React.Fragment>
                ))
              )}
            </TableBody>
          </Table>
            </div>

            {/* ── Card view: container < 600px ── */}
            <div className="@[600px]:hidden space-y-3 py-1">
              {!surgeries || surgeries.length === 0 ? (
                <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                  <Scissors className="h-8 w-8" />
                  <p className="text-sm">{t('animalRecords.surgeries.emptyTitle')}</p>
                </div>
              ) : (
                sortedData?.map((surgery) => (
                  <div key={surgery.id} className="rounded-lg border bg-card p-3 space-y-2">
                    <div className="flex items-center justify-between gap-2">
                      <span className="text-sm font-medium text-foreground">
                        {new Date(surgery.surgery_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })}
                      </span>
                      {surgery.is_first_experiment && (
                        <Badge className="bg-status-warning-bg text-status-warning-text">{t('animalRecords.surgeries.firstBadge')}</Badge>
                      )}
                    </div>
                    <p className="text-sm text-muted-foreground leading-snug break-words">{surgery.surgery_site}</p>
                    <div className="flex items-center justify-between gap-2 pt-1 border-t">
                      <div className="flex items-center gap-2">
                        {surgery.vet_read
                          ? <Badge className="bg-status-success-bg text-status-success-text text-xs">{t('animalRecords.shared.vetRead')}</Badge>
                          : <Badge variant="outline" className="text-muted-foreground text-xs">{t('animalRecords.shared.vetUnread')}</Badge>}
                        {surgery.no_medication_needed && (
                          <span title={t('animalRecords.shared.stopMedication')}>
                            <CheckCircle2 className="h-3.5 w-3.5 text-status-success-solid" />
                          </span>
                        )}
                      </div>
                      <div className="grid grid-cols-3 gap-0.5">
                        <Button variant="ghost" size="icon" onClick={() => setExpandedId(surgery.id)} title={t('animalRecords.shared.viewDetails')}>
                          <Eye className="h-4 w-4" />
                        </Button>
                        <GuestHide>
                          <Can permission={PERMISSIONS.ANIMAL_RECORD_EDIT}>
                            <Button variant="ghost" size="icon" onClick={() => { setEditingSurgery(surgery); setShowAddDialog(true) }} title={t('common.edit')}>
                              <Edit2 className="h-4 w-4" />
                            </Button>
                          </Can>
                          <Can permission={PERMISSIONS.ANIMAL_RECORD_COPY}>
                            <Button variant="ghost" size="icon" onClick={() => { if (confirm(t('animalRecords.shared.copyConfirm'))) copyMutation.mutate(surgery.id) }} disabled={copyMutation.isPending} title={t('animalRecords.shared.copy')}>
                              <Copy className="h-4 w-4" />
                            </Button>
                          </Can>
                          <Button variant="ghost" size="icon" onClick={() => { setVersionHistoryRecordId(surgery.id); setShowVersionHistory(true) }} title={t('animalRecords.shared.versionHistory')}>
                            <History className="h-4 w-4" />
                          </Button>
                          <Can permission={PERMISSIONS.ANIMAL_EXPORT_SURGERY}>
                            <Button variant="ghost" size="icon" onClick={() => downloadSurgeryPdf(surgery.id, surgery.surgery_date)} title={t('common.pdfExport.downloadPdf')} aria-label={t('common.pdfExport.downloadPdf')}>
                              <FileDown className="h-4 w-4" />
                            </Button>
                          </Can>
                          <Can permission={PERMISSIONS.ANIMAL_RECORD_DELETE}>
                            <Button variant="ghost" size="icon" onClick={() => setDeleteTarget(surgery.id)} title={t('common.delete')}>
                              <Trash2 className="h-4 w-4 text-status-error-solid" />
                            </Button>
                          </Can>
                        </GuestHide>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>

          </div>
        </CardContent>
      </Card>

      <SurgeryFormDialog
        open={showAddDialog}
        onOpenChange={(open) => {
          setShowAddDialog(open)
          if (!open) setEditingSurgery(null)
        }}
        animalId={animalId}
        earTag={earTag}
        surgery={editingSurgery || undefined}
      />

      {versionHistoryRecordId && (
        <VersionHistoryDialog
          open={showVersionHistory}
          onOpenChange={setShowVersionHistory}
          recordType="surgery"
          recordId={versionHistoryRecordId}
        />
      )}


      <DeleteReasonDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        copy={{ title: t('animalRecords.surgeries.deleteTitle'), description: t('animalRecords.shared.deleteRecordDescription') }}
        onConfirm={(reason) => deleteMutation.mutate({ id: deleteTarget!, reason })}
        isPending={deleteMutation.isPending}
      />
    </>
  )
})
