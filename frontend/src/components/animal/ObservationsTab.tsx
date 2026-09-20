import React, { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { GuestHide } from '@/components/ui/guest-hide'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  AnimalObservation,
  deleteResource,
  RecordType,
} from '@/lib/api'
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
import {
  Plus,
  Eye,
  Edit2,
  Trash2,
  History,
  CheckCircle2,
  ClipboardList,
  ChevronDown,
  Copy,
} from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { cn } from '@/lib/utils'
import { ObservationFormDialog } from './ObservationFormDialog'
import { VersionHistoryDialog } from './VersionHistoryDialog'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'
import { treatmentCategoryDisplayLabel, treatmentRouteDisplayLabel } from './treatmentConstants'

function getRecordTypeBadgeClass(type: RecordType): string {
  if (type === 'observation') return 'bg-status-info-bg text-status-info-text'
  if (type === 'experiment') return 'bg-status-success-bg text-status-success-text'
  return 'bg-status-warning-bg text-status-warning-text'
}

interface ObservationsTabProps {
  animalId: string
  earTag: string
  afterParam: string
  observations: AnimalObservation[] | undefined
}

export const ObservationsTab = React.memo(function ObservationsTab({ animalId, earTag, afterParam: _afterParam, observations }: ObservationsTabProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { sortedData, sort, toggleSort } = useTableSort(observations)

  const [showAddDialog, setShowAddDialog] = useState(false)
  const [editingObservation, setEditingObservation] = useState<AnimalObservation | null>(null)
  const [expandedId, setExpandedId] = useState<number | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<number | null>(null)
  const [versionHistoryRecordId, setVersionHistoryRecordId] = useState<number | null>(null)
  const [showVersionHistory, setShowVersionHistory] = useState(false)

  const deleteMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: number; reason: string }) => {
      return deleteResource(`/observations/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-observations', animalId] })
      toast({ title: t('common.success'), description: t('animalRecords.observations.deleted') })
      setDeleteTarget(null)
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(error, t('animalRecords.shared.deleteFailed')), variant: 'destructive' })
    },
  })

  const copyMutation = useMutation({
    mutationFn: async (sourceId: number) => {
      return api.post(`/animals/${animalId}/observations/copy`, { source_id: sourceId })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-observations', animalId] })
      toast({ title: t('common.success'), description: t('animalRecords.observations.copied') })
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(error, t('animalRecords.shared.copyFailed')), variant: 'destructive' })
    },
  })

  const ActionButtons = ({ obs }: { obs: AnimalObservation }) => (
    <div className="grid grid-cols-3 gap-0.5 justify-items-center">
      <Button variant="ghost" size="icon" onClick={() => setExpandedId(obs.id)} title={t('animalRecords.shared.viewDetails')}>
        <Eye className="h-4 w-4" />
      </Button>
      <GuestHide>
        <Can permission={PERMISSIONS.ANIMAL_RECORD_EDIT}>
          <Button variant="ghost" size="icon" onClick={() => { setEditingObservation(obs); setShowAddDialog(true) }} title={t('common.edit')}>
            <Edit2 className="h-4 w-4" />
          </Button>
        </Can>
      </GuestHide>
      <GuestHide>
        <Can permission={PERMISSIONS.ANIMAL_RECORD_COPY}>
          <Button variant="ghost" size="icon" onClick={() => { if (confirm(t('animalRecords.shared.copyConfirm'))) copyMutation.mutate(obs.id) }} disabled={copyMutation.isPending} title={t('animalRecords.shared.copy')}>
            <Copy className="h-4 w-4" />
          </Button>
        </Can>
      </GuestHide>
      {/* 版本歷史是唯讀，後端 get_observation_versions 也沒有 require_permission!，故不加閘 */}
      <GuestHide>
        <Button variant="ghost" size="icon" onClick={() => { setVersionHistoryRecordId(obs.id); setShowVersionHistory(true) }} title={t('animalRecords.shared.versionHistory')}>
          <History className="h-4 w-4" />
        </Button>
      </GuestHide>
      <GuestHide>
        <Can permission={PERMISSIONS.ANIMAL_RECORD_DELETE}>
          <Button variant="ghost" size="icon" onClick={() => setDeleteTarget(obs.id)} title={t('common.delete')}>
            <Trash2 className="h-4 w-4 text-status-error-solid" />
          </Button>
        </Can>
      </GuestHide>
    </div>
  )

  const formatDate = (dateStr: string) => {
    const d = new Date(dateStr)
    return `${d.getFullYear()}/${d.getMonth() + 1}/${d.getDate()}`
  }

  return (
    <>
      <Card className="overflow-hidden">
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>{t('animalDetail.tabs.observations')}</CardTitle>
            <CardDescription>{t('animalRecords.observations.description')}</CardDescription>
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
          {/* @container wraps both table and card views */}
          <div className="@container">

            {/* ── Table view: container ≥ 600px ── */}
            <div className="hidden @[600px]:block overflow-x-auto">
              <Table className="w-full" style={{ minWidth: '570px' }}>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <TableHead style={{ width: 40 }} />
                    <SortableTableHead
                      sortKey="event_date"
                      currentSort={sort.column}
                      currentDirection={sort.direction}
                      onSort={toggleSort}
                      style={{ width: 100 }}
                      className="text-center"
                    >
                      {t('animalRecords.observations.eventDate')}
                    </SortableTableHead>
                    <SortableTableHead
                      sortKey="record_type"
                      currentSort={sort.column}
                      currentDirection={sort.direction}
                      onSort={toggleSort}
                      style={{ width: 100 }}
                    >
                      {t('animalRecords.observations.recordNature')}
                    </SortableTableHead>
                    <TableHead style={{ minWidth: 150 }}>{t('animalRecords.observations.content')}</TableHead>
                    {/* Tertiary: hidden below 720px */}
                    <TableHead style={{ width: 60 }} className="text-center hidden @[720px]:table-cell">{t('animalRecords.shared.stopMedication')}</TableHead>
                    <TableHead style={{ width: 100 }} className="text-center">{t('animalRecords.shared.vetReadHeader')}</TableHead>
                    {/* Tertiary: hidden below 720px */}
                    <TableHead style={{ width: 90 }} className="hidden @[720px]:table-cell">{t('animalRecords.shared.recorder')}</TableHead>
                    <TableHead style={{ width: 80, minWidth: 80 }} className="sticky right-0 bg-card border-l text-center px-1 py-2">{t('common.actions')}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {!observations || observations.length === 0 ? (
                    <TableEmptyRow colSpan={8} icon={ClipboardList} title={t('animalRecords.observations.emptyTitle')} />
                  ) : (
                    sortedData?.map((obs: AnimalObservation) => (
                      <React.Fragment key={obs.id}>
                        <TableRow className="group cursor-pointer hover:bg-muted">
                          <TableCell style={{ width: 40 }} className="px-3 py-3">
                            <button
                              onClick={() => setExpandedId(expandedId === obs.id ? null : obs.id)}
                              className="p-1 hover:bg-muted rounded"
                              title={t('animalRecords.shared.expandDetails')}
                            >
                              <ChevronDown className={cn('h-4 w-4 transition-transform', expandedId === obs.id && 'rotate-180')} />
                            </button>
                          </TableCell>
                          <TableCell style={{ width: 100 }} className="px-3 py-3 text-sm text-center whitespace-nowrap">
                            {formatDate(obs.event_date)}
                          </TableCell>
                          <TableCell style={{ width: 100 }} className="px-3 py-3">
                            <Badge className={getRecordTypeBadgeClass(obs.record_type as RecordType)}>
                              {t(`animalRecords.observations.recordType.${obs.record_type as RecordType}`)}
                            </Badge>
                          </TableCell>
                          <TableCell style={{ minWidth: 150 }} className="px-3 py-3 whitespace-normal break-words leading-snug">
                            {obs.content}
                          </TableCell>
                          {/* Tertiary */}
                          <TableCell style={{ width: 60 }} className="px-3 py-3 text-center hidden @[720px]:table-cell">
                            {obs.no_medication_needed
                              ? <CheckCircle2 className="h-4 w-4 text-status-success-solid inline-block" />
                              : <span className="text-muted-foreground">-</span>}
                          </TableCell>
                          <TableCell style={{ width: 100 }} className="px-3 py-3 text-center">
                            {obs.vet_read
                              ? <Badge className="bg-status-success-bg text-status-success-text">{t('animalRecords.shared.read')}</Badge>
                              : <Badge variant="outline" className="text-muted-foreground">{t('animalRecords.shared.unread')}</Badge>}
                          </TableCell>
                          {/* Tertiary */}
                          <TableCell style={{ width: 90 }} className="px-3 py-3 whitespace-normal break-words hidden @[720px]:table-cell">
                            {obs.created_by_name || '-'}
                          </TableCell>
                          <TableCell style={{ width: 80, minWidth: 80 }} className="px-1 py-1 sticky right-0 bg-card group-hover:bg-muted border-l">
                            <ActionButtons obs={obs} />
                          </TableCell>
                        </TableRow>
                        {expandedId === obs.id && (
                          <TableRow>
                            <TableCell colSpan={8} className="bg-muted p-4">
                              <div className="grid grid-cols-2 gap-4">
                                <div>
                                  <Label className="text-muted-foreground">{t('animalRecords.observations.equipmentUsed')}</Label>
                                  <p>{obs.equipment_used?.join(', ') || '-'}</p>
                                </div>
                                <div>
                                  <Label className="text-muted-foreground">{t('animalRecords.observations.anesthesiaTime')}</Label>
                                  <p>
                                    {obs.anesthesia_start && obs.anesthesia_end
                                      ? `${obs.anesthesia_start} - ${obs.anesthesia_end}`
                                      : '-'}
                                  </p>
                                </div>
                                <div className="col-span-2">
                                  <Label className="text-muted-foreground">{t('animalRecords.observations.detailedContent')}</Label>
                                  <p className="whitespace-pre-wrap">{obs.content}</p>
                                </div>
                                {obs.treatments && obs.treatments.length > 0 && (
                                  <div className="col-span-2">
                                    <Label className="text-muted-foreground">{t('animalRecords.observations.treatments')}</Label>
                                    <div className="space-y-1 mt-1">
                                      {obs.treatments.map((item: { drug: string; dosage: string; dosage_unit?: string; end_date?: string; category?: string; route?: string }, i: number) => (
                                        <p key={i}>
                                          {item.category && `[${treatmentCategoryDisplayLabel(item.category, t)}] `}
                                          {item.drug} - {item.dosage}{item.dosage_unit ? ` ${item.dosage_unit}` : ''}
                                          {item.route && ` · ${treatmentRouteDisplayLabel(item.route, t)}`}
                                          {item.end_date && ` ${t('animalRecords.observations.treatmentUntil', { date: item.end_date })}`}
                                        </p>
                                      ))}
                                    </div>
                                  </div>
                                )}
                                {obs.remark && (
                                  <div className="col-span-2">
                                    <Label className="text-muted-foreground">{t('animalRecords.shared.remark')}</Label>
                                    <p>{obs.remark}</p>
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
              {!observations || observations.length === 0 ? (
                <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                  <ClipboardList className="h-8 w-8" />
                  <p className="text-sm">{t('animalRecords.observations.emptyTitle')}</p>
                </div>
              ) : (
                sortedData?.map((obs: AnimalObservation) => (
                  <div key={obs.id} className="rounded-lg border bg-card p-3 space-y-2">
                    {/* Card header */}
                    <div className="flex items-center justify-between gap-2">
                      <span className="text-sm font-medium text-foreground">{formatDate(obs.event_date)}</span>
                      <Badge className={getRecordTypeBadgeClass(obs.record_type as RecordType)}>
                        {t(`animalRecords.observations.recordType.${obs.record_type as RecordType}`)}
                      </Badge>
                    </div>
                    {/* Card body */}
                    <p className="text-sm text-muted-foreground leading-snug break-words">{obs.content}</p>
                    {/* Card footer */}
                    <div className="flex items-center justify-between gap-2 pt-1 border-t">
                      <div className="flex items-center gap-2">
                        {obs.vet_read
                          ? <Badge className="bg-status-success-bg text-status-success-text text-xs">{t('animalRecords.shared.vetRead')}</Badge>
                          : <Badge variant="outline" className="text-muted-foreground text-xs">{t('animalRecords.shared.vetUnread')}</Badge>}
                        {obs.no_medication_needed && (
                          <span title={t('animalRecords.shared.stopMedication')}>
                            <CheckCircle2 className="h-3.5 w-3.5 text-status-success-solid" />
                          </span>
                        )}
                      </div>
                      <ActionButtons obs={obs} />
                    </div>
                  </div>
                ))
              )}
            </div>

          </div>
        </CardContent>
      </Card>

      <ObservationFormDialog
        open={showAddDialog}
        onOpenChange={(open) => {
          setShowAddDialog(open)
          if (!open) setEditingObservation(null)
        }}
        animalId={animalId}
        earTag={earTag}
        observation={editingObservation || undefined}
      />

      {versionHistoryRecordId && (
        <VersionHistoryDialog
          open={showVersionHistory}
          onOpenChange={setShowVersionHistory}
          recordType="observation"
          recordId={versionHistoryRecordId}
        />
      )}

      <DeleteReasonDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        copy={{ title: t('animalRecords.observations.deleteTitle'), description: t('animalRecords.shared.deleteRecordDescription') }}
        onConfirm={(reason) => deleteMutation.mutate({ id: deleteTarget!, reason })}
        isPending={deleteMutation.isPending}
      />
    </>
  )
})
