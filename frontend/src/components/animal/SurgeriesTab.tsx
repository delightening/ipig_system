import React, { useState } from 'react'
import { GuestHide } from '@/components/ui/guest-hide'
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
import { getApiErrorMessage } from '@/lib/validation'
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
  Stethoscope,
} from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { SurgeryFormDialog } from './SurgeryFormDialog'
import { VersionHistoryDialog } from './VersionHistoryDialog'
import { VetRecommendationDialog } from './VetRecommendationDialog'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'

interface SurgeriesTabProps {
  animalId: string
  earTag: string
  afterParam: string
  surgeries: AnimalSurgery[] | undefined
}

export const SurgeriesTab = React.memo(function SurgeriesTab({ animalId, earTag, afterParam: _afterParam, surgeries }: SurgeriesTabProps) {
  const queryClient = useQueryClient()
  const { sortedData, sort, toggleSort } = useTableSort(surgeries)

  const [showAddDialog, setShowAddDialog] = useState(false)
  const [editingSurgery, setEditingSurgery] = useState<AnimalSurgery | null>(null)
  const [expandedId, setExpandedId] = useState<number | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<number | null>(null)

  const [versionHistoryRecordId, setVersionHistoryRecordId] = useState<number | null>(null)
  const [showVersionHistory, setShowVersionHistory] = useState(false)

  const [vetRecRecordId, setVetRecRecordId] = useState<number | null>(null)
  const [showVetRec, setShowVetRec] = useState(false)

  const deleteMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: number; reason: string }) => {
      return deleteResource(`/surgeries/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-surgeries', animalId] })
      toast({ title: '成功', description: '手術紀錄已刪除' })
      setDeleteTarget(null)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '刪除失敗'),
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
      toast({ title: '成功', description: '手術紀錄已複製，請編輯新紀錄' })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '複製失敗'),
        variant: 'destructive',
      })
    },
  })

  return (
    <>
      <Card className="overflow-hidden">
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>手術紀錄</CardTitle>
            <CardDescription>記錄手術過程、麻醉資訊與術後照護</CardDescription>
          </div>
          <GuestHide>
            <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowAddDialog(true)}>
              <Plus className="h-4 w-4 mr-2" />
              新增紀錄
            </Button>
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
                    <TableHead style={{ width: 70 }} className="text-center">是否首次</TableHead>
                    <SortableTableHead style={{ width: 100 }} sortKey="surgery_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>手術日期</SortableTableHead>
                    <SortableTableHead style={{ minWidth: 150 }} sortKey="surgery_site" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>手術部位</SortableTableHead>
                    <TableHead style={{ width: 60 }} className="text-center hidden @[690px]:table-cell">停止用藥</TableHead>
                    <TableHead style={{ width: 100 }} className="text-center">獸醫師讀取</TableHead>
                    <TableHead style={{ width: 90 }} className="hidden @[690px]:table-cell">記錄者</TableHead>
                    <TableHead style={{ width: 80, minWidth: 80 }} className="sticky right-0 bg-card border-l text-center px-1 py-2">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {!surgeries || surgeries.length === 0 ? (
                    <TableEmptyRow colSpan={8} icon={Scissors} title="尚無手術紀錄" />
                  ) : (
                    sortedData?.map((surgery) => (
                      <React.Fragment key={surgery.id}>
                        <TableRow className="group cursor-pointer hover:bg-muted">
                          <TableCell style={{ width: 40 }}>
                            <button
                              type="button"
                              onClick={() => setExpandedId(expandedId === surgery.id ? null : surgery.id)}
                              className="p-1 hover:bg-muted rounded"
                              title="展開詳細資料"
                              aria-label="展開詳細資料"
                            >
                              <ChevronDown
                                className={`h-4 w-4 transition-transform ${expandedId === surgery.id ? 'rotate-180' : ''}`}
                              />
                            </button>
                          </TableCell>
                          <TableCell style={{ width: 70 }} className="text-center">
                            {surgery.is_first_experiment
                              ? <Badge className="bg-status-warning-bg text-status-warning-text">首次</Badge>
                              : <span className="text-muted-foreground text-sm">否</span>}
                          </TableCell>
                          <TableCell style={{ width: 100 }} className="whitespace-nowrap">{new Date(surgery.surgery_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })}</TableCell>
                          <TableCell style={{ minWidth: 150 }} className="whitespace-normal break-words">{surgery.surgery_site}</TableCell>
                          <TableCell style={{ width: 60 }} className="text-center hidden @[690px]:table-cell">
                            {surgery.no_medication_needed ? (
                              <CheckCircle2 className="h-4 w-4 text-status-success-solid inline-block" />
                            ) : (
                              <span className="text-muted-foreground">-</span>
                            )}
                          </TableCell>
                          <TableCell style={{ width: 100 }} className="text-center">
                            {surgery.vet_read ? (
                              <Badge className="bg-status-success-bg text-status-success-text">已讀</Badge>
                            ) : (
                              <Badge variant="outline" className="text-muted-foreground">未讀</Badge>
                            )}
                          </TableCell>
                          <TableCell style={{ width: 90 }} className="whitespace-normal break-words hidden @[690px]:table-cell">{surgery.created_by_name || '-'}</TableCell>
                          <TableCell style={{ width: 80, minWidth: 80 }} className="px-1 py-1 sticky right-0 bg-card group-hover:bg-muted border-l">
                            <div className="grid grid-cols-3 gap-0.5 justify-items-center">
                              <Button variant="ghost" size="icon" onClick={() => setExpandedId(surgery.id)} title="檢視詳情" aria-label="檢視詳情">
                                <Eye className="h-4 w-4" />
                              </Button>
                              <GuestHide>
                                <Button variant="ghost" size="icon" onClick={() => { setEditingSurgery(surgery); setShowAddDialog(true) }} title="編輯">
                                  <Edit2 className="h-4 w-4" />
                                </Button>
                                <Button variant="ghost" size="icon" onClick={() => { if (confirm('確定要複製此紀錄？將建立一份新紀錄供編輯。')) copyMutation.mutate(surgery.id) }} disabled={copyMutation.isPending} title="複製">
                                  <Copy className="h-4 w-4" />
                                </Button>
                                <Button variant="ghost" size="icon" onClick={() => { setVersionHistoryRecordId(surgery.id); setShowVersionHistory(true) }} title="版本歷史">
                                  <History className="h-4 w-4" />
                                </Button>
                                <Button variant="ghost" size="icon" onClick={() => { setVetRecRecordId(surgery.id); setShowVetRec(true) }} title="獸醫師建議" className="text-status-success-text hover:text-status-success-text">
                                  <Stethoscope className="h-4 w-4" />
                                </Button>
                                <Button variant="ghost" size="icon" onClick={() => setDeleteTarget(surgery.id)} title="刪除" aria-label="刪除">
                                  <Trash2 className="h-4 w-4 text-status-error-solid" />
                                </Button>
                              </GuestHide>
                            </div>
                          </TableCell>
                        </TableRow>
                        {expandedId === surgery.id && (
                          <TableRow>
                            <TableCell colSpan={8} className="bg-muted p-4">
                          <div className="grid grid-cols-3 gap-4">
                            <div>
                              <Label className="text-muted-foreground">誘導麻醉</Label>
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
                              <Label className="text-muted-foreground">麻醉維持</Label>
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
                              <Label className="text-muted-foreground">固定姿勢</Label>
                              <p>{surgery.positioning ? surgery.positioning.split(',').join('、') : '-'}</p>
                            </div>
                            {surgery.anesthesia_observation && (
                              <div className="col-span-3">
                                <Label className="text-muted-foreground">麻醉觀察過程</Label>
                                <p className="whitespace-pre-wrap">{surgery.anesthesia_observation}</p>
                              </div>
                            )}
                            {surgery.vital_signs && surgery.vital_signs.length > 0 && (
                              <div className="col-span-3">
                                <Label className="text-muted-foreground">生理數值</Label>
                                <div className="mt-2 overflow-x-auto">
                                  <table className="min-w-full text-sm">
                                    <thead>
                                      <tr className="border-b">
                                        <th className="px-2 py-1 text-left">時間</th>
                                        <th className="px-2 py-1 text-left">心跳</th>
                                        <th className="px-2 py-1 text-left">呼吸</th>
                                        <th className="px-2 py-1 text-left">體溫</th>
                                        <th className="px-2 py-1 text-left">SPO2</th>
                                      </tr>
                                    </thead>
                                    <tbody>
                                      {surgery.vital_signs.map((vs, i) => (
                                        <tr key={i} className="border-b">
                                          <td className="px-2 py-1">{vs.time}</td>
                                          <td className="px-2 py-1">{vs.heart_rate}/分</td>
                                          <td className="px-2 py-1">{vs.respiration_rate}/分</td>
                                          <td className="px-2 py-1">{vs.temperature}°C</td>
                                          <td className="px-2 py-1">{vs.spo2}%</td>
                                        </tr>
                                      ))}
                                    </tbody>
                                  </table>
                                </div>
                              </div>
                            )}
                            {surgery.reflex_recovery && (
                              <div className="col-span-3">
                                <Label className="text-muted-foreground">反射恢復觀察</Label>
                                <p>{surgery.reflex_recovery}</p>
                              </div>
                            )}
                            {surgery.remark && (
                              <div className="col-span-3">
                                <Label className="text-muted-foreground">備註</Label>
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
                  <p className="text-sm">尚無手術紀錄</p>
                </div>
              ) : (
                sortedData?.map((surgery) => (
                  <div key={surgery.id} className="rounded-lg border bg-card p-3 space-y-2">
                    <div className="flex items-center justify-between gap-2">
                      <span className="text-sm font-medium text-foreground">
                        {new Date(surgery.surgery_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })}
                      </span>
                      {surgery.is_first_experiment && (
                        <Badge className="bg-status-warning-bg text-status-warning-text">首次</Badge>
                      )}
                    </div>
                    <p className="text-sm text-muted-foreground leading-snug break-words">{surgery.surgery_site}</p>
                    <div className="flex items-center justify-between gap-2 pt-1 border-t">
                      <div className="flex items-center gap-2">
                        {surgery.vet_read
                          ? <Badge className="bg-status-success-bg text-status-success-text text-xs">獸醫已讀</Badge>
                          : <Badge variant="outline" className="text-muted-foreground text-xs">獸醫未讀</Badge>}
                        {surgery.no_medication_needed && (
                          <span title="停止用藥">
                            <CheckCircle2 className="h-3.5 w-3.5 text-status-success-solid" />
                          </span>
                        )}
                      </div>
                      <div className="grid grid-cols-3 gap-0.5">
                        <Button variant="ghost" size="icon" onClick={() => setExpandedId(surgery.id)} title="檢視詳情">
                          <Eye className="h-4 w-4" />
                        </Button>
                        <GuestHide>
                          <Button variant="ghost" size="icon" onClick={() => { setEditingSurgery(surgery); setShowAddDialog(true) }} title="編輯">
                            <Edit2 className="h-4 w-4" />
                          </Button>
                          <Button variant="ghost" size="icon" onClick={() => { if (confirm('確定要複製此紀錄？將建立一份新紀錄供編輯。')) copyMutation.mutate(surgery.id) }} disabled={copyMutation.isPending} title="複製">
                            <Copy className="h-4 w-4" />
                          </Button>
                          <Button variant="ghost" size="icon" onClick={() => { setVersionHistoryRecordId(surgery.id); setShowVersionHistory(true) }} title="版本歷史">
                            <History className="h-4 w-4" />
                          </Button>
                          <Button variant="ghost" size="icon" onClick={() => { setVetRecRecordId(surgery.id); setShowVetRec(true) }} title="獸醫師建議" className="text-status-success-text hover:text-status-success-text">
                            <Stethoscope className="h-4 w-4" />
                          </Button>
                          <Button variant="ghost" size="icon" onClick={() => setDeleteTarget(surgery.id)} title="刪除">
                            <Trash2 className="h-4 w-4 text-status-error-solid" />
                          </Button>
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

      {vetRecRecordId && (
        <VetRecommendationDialog
          open={showVetRec}
          onOpenChange={setShowVetRec}
          recordType="surgery"
          recordId={vetRecRecordId}
          animalEarTag={earTag}
        />
      )}

      <DeleteReasonDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        title="刪除手術紀錄"
        description="此操作將標記紀錄為已刪除，資料將保留於系統中以符合 GLP 規範。"
        onConfirm={(reason) => deleteMutation.mutate({ id: deleteTarget!, reason })}
        isPending={deleteMutation.isPending}
      />
    </>
  )
})
