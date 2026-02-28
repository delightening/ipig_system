import React, { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  AnimalObservation,
  recordTypeNames,
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
  Stethoscope,
} from 'lucide-react'
import { ObservationFormDialog } from './ObservationFormDialog'
import { VersionHistoryDialog } from './VersionHistoryDialog'
import { VetRecommendationDialog } from './VetRecommendationDialog'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'

interface ObservationsTabProps {
  animalId: string
  earTag: string
  afterParam: string
  observations: AnimalObservation[] | undefined
}

export const ObservationsTab = React.memo(function ObservationsTab({ animalId, earTag, afterParam, observations }: ObservationsTabProps) {
  const queryClient = useQueryClient()

  const [showAddDialog, setShowAddDialog] = useState(false)
  const [editingObservation, setEditingObservation] = useState<AnimalObservation | null>(null)
  const [expandedId, setExpandedId] = useState<number | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<number | null>(null)

  const [versionHistoryRecordId, setVersionHistoryRecordId] = useState<number | null>(null)
  const [showVersionHistory, setShowVersionHistory] = useState(false)

  const [vetRecRecordId, setVetRecRecordId] = useState<number | null>(null)
  const [showVetRec, setShowVetRec] = useState(false)

  const deleteMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: number; reason: string }) => {
      return api.delete(`/observations/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-observations', animalId] })
      toast({ title: '成功', description: '觀察紀錄已刪除' })
      setDeleteTarget(null)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '刪除失敗',
        variant: 'destructive',
      })
    },
  })

  const copyMutation = useMutation({
    mutationFn: async (sourceId: number) => {
      return api.post(`/animals/${animalId}/observations/copy`, { source_id: sourceId })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-observations', animalId] })
      toast({ title: '成功', description: '觀察紀錄已複製，請編輯新紀錄' })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '複製失敗',
        variant: 'destructive',
      })
    },
  })

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>觀察試驗紀錄</CardTitle>
            <CardDescription>記錄日常觀察、異常狀況與試驗操作</CardDescription>
          </div>
          <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowAddDialog(true)}>
            <Plus className="h-4 w-4 mr-2" />
            新增紀錄
          </Button>
        </CardHeader>
        <CardContent>
          {!observations || observations.length === 0 ? (
            <div className="text-center py-12 text-slate-500">
              <ClipboardList className="h-12 w-12 mx-auto mb-4 text-slate-300" />
              <p>尚無觀察試驗紀錄</p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-10"></TableHead>
                  <TableHead>事件日期</TableHead>
                  <TableHead>紀錄性質</TableHead>
                  <TableHead>內容</TableHead>
                  <TableHead>停止用藥</TableHead>
                  <TableHead>獸醫師讀取</TableHead>
                  <TableHead>記錄者</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {observations.map((obs: AnimalObservation) => (
                  <>
                    <TableRow key={obs.id} className="cursor-pointer hover:bg-slate-50">
                      <TableCell>
                        <button
                          onClick={() => setExpandedId(expandedId === obs.id ? null : obs.id)}
                          className="p-1 hover:bg-slate-200 rounded"
                          title="展開詳細資料"
                          aria-label="展開詳細資料"
                        >
                          <ChevronDown
                            className={`h-4 w-4 transition-transform ${expandedId === obs.id ? 'rotate-180' : ''}`}
                          />
                        </button>
                      </TableCell>
                      <TableCell>{new Date(obs.event_date).toLocaleDateString('zh-TW')}</TableCell>
                      <TableCell>
                        <Badge variant="outline">{recordTypeNames[obs.record_type as RecordType]}</Badge>
                      </TableCell>
                      <TableCell className="max-w-xs truncate">{obs.content}</TableCell>
                      <TableCell>
                        {obs.no_medication_needed ? (
                          <CheckCircle2 className="h-4 w-4 text-green-500" />
                        ) : (
                          <span className="text-slate-400">-</span>
                        )}
                      </TableCell>
                      <TableCell>
                        {obs.vet_read ? (
                          <Badge className="bg-green-100 text-green-800">已讀</Badge>
                        ) : (
                          <Badge variant="outline" className="text-slate-500">未讀</Badge>
                        )}
                      </TableCell>
                      <TableCell>{obs.created_by_name || '-'}</TableCell>
                      <TableCell className="text-right">
                        <div className="flex items-center justify-end gap-1">
                          <Button variant="ghost" size="icon" onClick={() => setExpandedId(obs.id)} title="檢視詳情" aria-label="檢視詳情">
                            <Eye className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => {
                              setEditingObservation(obs)
                              setShowAddDialog(true)
                            }}
                            title="編輯"
                          >
                            <Edit2 className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => {
                              if (confirm('確定要複製此紀錄？將建立一份新紀錄供編輯。')) {
                                copyMutation.mutate(obs.id)
                              }
                            }}
                            disabled={copyMutation.isPending}
                            title="複製"
                          >
                            <Copy className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => {
                              setVersionHistoryRecordId(obs.id)
                              setShowVersionHistory(true)
                            }}
                            title="版本歷史"
                          >
                            <History className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => {
                              setVetRecRecordId(obs.id)
                              setShowVetRec(true)
                            }}
                            title="獸醫師建議"
                            className="text-green-600 hover:text-green-700"
                          >
                            <Stethoscope className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => setDeleteTarget(obs.id)}
                            title="刪除"
                            aria-label="刪除"
                          >
                            <Trash2 className="h-4 w-4 text-red-500" />
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                    {expandedId === obs.id && (
                      <TableRow>
                        <TableCell colSpan={8} className="bg-slate-50 p-4">
                          <div className="grid grid-cols-2 gap-4">
                            <div>
                              <Label className="text-slate-500">使用儀器</Label>
                              <p>{obs.equipment_used?.join(', ') || '-'}</p>
                            </div>
                            <div>
                              <Label className="text-slate-500">麻醉時間</Label>
                              <p>
                                {obs.anesthesia_start && obs.anesthesia_end
                                  ? `${obs.anesthesia_start} - ${obs.anesthesia_end}`
                                  : '-'}
                              </p>
                            </div>
                            <div className="col-span-2">
                              <Label className="text-slate-500">詳細內容</Label>
                              <p className="whitespace-pre-wrap">{obs.content}</p>
                            </div>
                            {obs.treatments && obs.treatments.length > 0 && (
                              <div className="col-span-2">
                                <Label className="text-slate-500">治療方式</Label>
                                <div className="space-y-1 mt-1">
                                  {obs.treatments.map((t: { drug: string; dosage: string; end_date?: string }, i: number) => (
                                    <p key={i}>
                                      {t.drug} - {t.dosage}
                                      {t.end_date && ` (至 ${t.end_date})`}
                                    </p>
                                  ))}
                                </div>
                              </div>
                            )}
                            {obs.remark && (
                              <div className="col-span-2">
                                <Label className="text-slate-500">備註</Label>
                                <p>{obs.remark}</p>
                              </div>
                            )}
                          </div>
                        </TableCell>
                      </TableRow>
                    )}
                  </>
                ))}
              </TableBody>
            </Table>
          )}
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

      {vetRecRecordId && (
        <VetRecommendationDialog
          open={showVetRec}
          onOpenChange={setShowVetRec}
          recordType="observation"
          recordId={vetRecRecordId}
          animalEarTag={earTag}
        />
      )}

      <DeleteReasonDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        title="刪除觀察紀錄"
        description="此操作將標記紀錄為已刪除，資料將保留於系統中以符合 GLP 規範。"
        onConfirm={(reason) => deleteMutation.mutate({ id: deleteTarget!, reason })}
        isPending={deleteMutation.isPending}
      />
    </>
  )
})
