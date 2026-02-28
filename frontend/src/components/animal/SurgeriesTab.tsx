import React, { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { AnimalSurgery } from '@/lib/api'
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
      return api.delete(`/surgeries/${id}`, { data: { reason } })
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
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>手術紀錄</CardTitle>
            <CardDescription>記錄手術過程、麻醉資訊與術後照護</CardDescription>
          </div>
          <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowAddDialog(true)}>
            <Plus className="h-4 w-4 mr-2" />
            新增紀錄
          </Button>
        </CardHeader>
        <CardContent>
          {!surgeries || surgeries.length === 0 ? (
            <div className="text-center py-12 text-slate-500">
              <Scissors className="h-12 w-12 mx-auto mb-4 text-slate-300" />
              <p>尚無手術紀錄</p>
              <p className="text-sm mt-1">點擊上方按鈕新增</p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-10"></TableHead>
                  <TableHead>是否首次</TableHead>
                  <TableHead>手術日期</TableHead>
                  <TableHead>手術部位</TableHead>
                  <TableHead>停止用藥</TableHead>
                  <TableHead>獸醫師讀取</TableHead>
                  <TableHead>記錄者</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {surgeries.map((surgery) => (
                  <>
                    <TableRow key={surgery.id} className="cursor-pointer hover:bg-slate-50">
                      <TableCell>
                        <button
                          type="button"
                          onClick={() => setExpandedId(expandedId === surgery.id ? null : surgery.id)}
                          className="p-1 hover:bg-slate-200 rounded"
                          title="展開詳細資料"
                          aria-label="展開詳細資料"
                        >
                          <ChevronDown
                            className={`h-4 w-4 transition-transform ${expandedId === surgery.id ? 'rotate-180' : ''}`}
                          />
                        </button>
                      </TableCell>
                      <TableCell>{surgery.is_first_experiment ? '是' : '否'}</TableCell>
                      <TableCell>{new Date(surgery.surgery_date).toLocaleDateString('zh-TW')}</TableCell>
                      <TableCell className="max-w-xs truncate">{surgery.surgery_site}</TableCell>
                      <TableCell>
                        {surgery.no_medication_needed ? (
                          <CheckCircle2 className="h-4 w-4 text-green-500" />
                        ) : (
                          <span className="text-slate-400">-</span>
                        )}
                      </TableCell>
                      <TableCell>
                        {surgery.vet_read ? (
                          <Badge className="bg-green-100 text-green-800">已讀</Badge>
                        ) : (
                          <Badge variant="outline" className="text-slate-500">未讀</Badge>
                        )}
                      </TableCell>
                      <TableCell>{surgery.created_by_name || '-'}</TableCell>
                      <TableCell className="text-right">
                        <div className="flex items-center justify-end gap-1">
                          <Button variant="ghost" size="icon" onClick={() => setExpandedId(surgery.id)} title="檢視詳情" aria-label="檢視詳情">
                            <Eye className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => {
                              setEditingSurgery(surgery)
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
                                copyMutation.mutate(surgery.id)
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
                              setVersionHistoryRecordId(surgery.id)
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
                              setVetRecRecordId(surgery.id)
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
                            onClick={() => setDeleteTarget(surgery.id)}
                            title="刪除"
                            aria-label="刪除"
                          >
                            <Trash2 className="h-4 w-4 text-red-500" />
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                    {expandedId === surgery.id && (
                      <TableRow>
                        <TableCell colSpan={8} className="bg-slate-50 p-4">
                          <div className="grid grid-cols-3 gap-4">
                            <div>
                              <Label className="text-slate-500">誘導麻醉</Label>
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
                              <Label className="text-slate-500">麻醉維持</Label>
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
                              <Label className="text-slate-500">固定姿勢</Label>
                              <p>{surgery.positioning || '-'}</p>
                            </div>
                            {surgery.anesthesia_observation && (
                              <div className="col-span-3">
                                <Label className="text-slate-500">麻醉觀察過程</Label>
                                <p className="whitespace-pre-wrap">{surgery.anesthesia_observation}</p>
                              </div>
                            )}
                            {surgery.vital_signs && surgery.vital_signs.length > 0 && (
                              <div className="col-span-3">
                                <Label className="text-slate-500">生理數值</Label>
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
                                <Label className="text-slate-500">反射恢復觀察</Label>
                                <p>{surgery.reflex_recovery}</p>
                              </div>
                            )}
                            {surgery.remark && (
                              <div className="col-span-3">
                                <Label className="text-slate-500">備註</Label>
                                <p>{surgery.remark}</p>
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
