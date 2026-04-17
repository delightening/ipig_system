import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import { useAuthStore } from '@/stores/auth'
import {
  listMonitoringPoints,
  createMonitoringPoint,
  listReadings,
  createReading,
} from '@/lib/api/glpCompliance'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter,
} from '@/components/ui/dialog'
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Plus, Thermometer } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

const LOCATION_TYPES = [
  { value: 'animal_room', label: '動物房' },
  { value: 'lab', label: '實驗室' },
  { value: 'storage', label: '儲存區' },
  { value: 'clean_room', label: '無塵室' },
]

const INTERVALS = [
  { value: 'continuous', label: '連續' },
  { value: 'hourly', label: '每小時' },
  { value: 'daily', label: '每日' },
]

const INITIAL_POINT_FORM = { name: '', location_type: 'animal_room', monitoring_interval: 'daily' }
const INITIAL_READING_FORM = { monitoring_point_id: '', reading_time: '', notes: '', readings: [{ key: '', value: '' }] }

export function EnvironmentMonitoringPage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('env.monitoring.manage')

  const [selectedPointId, setSelectedPointId] = useState<string>('')
  const [showCreatePoint, setShowCreatePoint] = useState(false)
  const [showCreateReading, setShowCreateReading] = useState(false)
  const [pointForm, setPointForm] = useState(INITIAL_POINT_FORM)
  const [readingForm, setReadingForm] = useState(INITIAL_READING_FORM)

  const { data: points = [], isLoading: loadingPoints } = useQuery({
    queryKey: ['monitoring-points'],
    queryFn: () => listMonitoringPoints(),
  })

  const { data: readings = [], isLoading: loadingReadings } = useQuery({
    queryKey: ['env-readings', selectedPointId],
    queryFn: () => listReadings({ monitoring_point_id: selectedPointId || undefined }),
    enabled: !!selectedPointId,
  })

  const createPointMutation = useMutation({
    mutationFn: () =>
      createMonitoringPoint({
        name: pointForm.name,
        location_type: pointForm.location_type,
        monitoring_interval: pointForm.monitoring_interval,
        parameters: [],
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['monitoring-points'] })
      setShowCreatePoint(false)
      setPointForm(INITIAL_POINT_FORM)
      toast({ title: '監控點已建立' })
    },
    onError: (err: unknown) => toast({ title: '建立失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const createReadingMutation = useMutation({
    mutationFn: () => {
      const readingsMap: Record<string, number> = {}
      for (const r of readingForm.readings) {
        if (r.key && r.value) readingsMap[r.key] = Number(r.value)
      }
      return createReading({
        monitoring_point_id: readingForm.monitoring_point_id,
        reading_time: readingForm.reading_time,
        readings: readingsMap,
        notes: readingForm.notes || undefined,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['env-readings'] })
      setShowCreateReading(false)
      setReadingForm(INITIAL_READING_FORM)
      toast({ title: '讀數已記錄' })
    },
    onError: (err: unknown) => toast({ title: '記錄失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  function addReadingRow() {
    setReadingForm((f) => ({ ...f, readings: [...f.readings, { key: '', value: '' }] }))
  }

  function updateReadingRow(idx: number, field: 'key' | 'value', val: string) {
    setReadingForm((f) => ({
      ...f,
      readings: f.readings.map((r, i) => (i === idx ? { ...r, [field]: val } : r)),
    }))
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">環境監控</h1>
          <p className="text-muted-foreground">GLP / ISO 17025 環境條件管理</p>
        </div>
        {canManage && (
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => setShowCreateReading(true)}>
              <Thermometer className="mr-2 h-4 w-4" />
              記錄讀數
            </Button>
            <Button onClick={() => setShowCreatePoint(true)}>
              <Plus className="mr-2 h-4 w-4" />
              新增監控點
            </Button>
          </div>
        )}
      </div>

      {/* Monitoring Points */}
      <Card>
        <CardHeader><CardTitle>監控點</CardTitle></CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>名稱</TableHead>
                <TableHead>位置類型</TableHead>
                <TableHead>監控頻率</TableHead>
                <TableHead>啟用</TableHead>
                <TableHead>操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {loadingPoints ? (
                <TableRow><TableCell colSpan={5} className="text-center py-8">載入中...</TableCell></TableRow>
              ) : points.length === 0 ? (
                <TableRow><TableCell colSpan={5} className="text-center py-8 text-muted-foreground">尚無監控點</TableCell></TableRow>
              ) : (
                points.map((p) => (
                  <TableRow key={p.id}>
                    <TableCell className="font-medium">{p.name}</TableCell>
                    <TableCell>{LOCATION_TYPES.find((lt) => lt.value === p.location_type)?.label ?? p.location_type}</TableCell>
                    <TableCell>{INTERVALS.find((i) => i.value === p.monitoring_interval)?.label ?? p.monitoring_interval ?? '-'}</TableCell>
                    <TableCell>
                      <Badge className={p.is_active
                        ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                        : 'bg-muted text-muted-foreground'}>
                        {p.is_active ? '啟用' : '停用'}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <Button variant="ghost" size="sm" onClick={() => setSelectedPointId(p.id)}>
                        查看讀數
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {/* Readings */}
      {selectedPointId && (
        <Card>
          <CardHeader>
            <CardTitle>
              讀數紀錄 — {points.find((p) => p.id === selectedPointId)?.name ?? ''}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>讀數時間</TableHead>
                  <TableHead>讀數</TableHead>
                  <TableHead>超標</TableHead>
                  <TableHead>備註</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {loadingReadings ? (
                  <TableRow><TableCell colSpan={4} className="text-center py-8">載入中...</TableCell></TableRow>
                ) : readings.length === 0 ? (
                  <TableRow><TableCell colSpan={4} className="text-center py-8 text-muted-foreground">尚無讀數</TableCell></TableRow>
                ) : (
                  readings.map((rd) => (
                    <TableRow key={rd.id}>
                      <TableCell>{new Date(rd.reading_time).toLocaleString('zh-TW')}</TableCell>
                      <TableCell className="font-mono text-sm">
                        {Object.entries(rd.readings).map(([k, v]) => `${k}: ${v}`).join(', ')}
                      </TableCell>
                      <TableCell>
                        {rd.is_out_of_range ? (
                          <Badge className="bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200">超標</Badge>
                        ) : (
                          <Badge className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">正常</Badge>
                        )}
                      </TableCell>
                      <TableCell>{rd.notes ?? '-'}</TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}

      {/* Create Point Dialog */}
      <Dialog open={showCreatePoint} onOpenChange={setShowCreatePoint}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增監控點</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">名稱 *</label>
              <Input value={pointForm.name} onChange={(e) => setPointForm((f) => ({ ...f, name: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">位置類型 *</label>
              <Select value={pointForm.location_type} onValueChange={(v) => setPointForm((f) => ({ ...f, location_type: v }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {LOCATION_TYPES.map((lt) => (
                    <SelectItem key={lt.value} value={lt.value}>{lt.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">監控頻率 *</label>
              <Select value={pointForm.monitoring_interval} onValueChange={(v) => setPointForm((f) => ({ ...f, monitoring_interval: v }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {INTERVALS.map((i) => (
                    <SelectItem key={i.value} value={i.value}>{i.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreatePoint(false)}>取消</Button>
            <Button onClick={() => createPointMutation.mutate()} disabled={!pointForm.name || createPointMutation.isPending}>
              建立
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Create Reading Dialog */}
      <Dialog open={showCreateReading} onOpenChange={setShowCreateReading}>
        <DialogContent>
          <DialogHeader><DialogTitle>記錄讀數</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">監控點 *</label>
              <Select value={readingForm.monitoring_point_id} onValueChange={(v) => setReadingForm((f) => ({ ...f, monitoring_point_id: v }))}>
                <SelectTrigger><SelectValue placeholder="選擇監控點" /></SelectTrigger>
                <SelectContent>
                  {points.filter((p) => p.is_active).map((p) => (
                    <SelectItem key={p.id} value={p.id}>{p.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">讀數時間 *</label>
              <Input type="datetime-local" value={readingForm.reading_time} onChange={(e) => setReadingForm((f) => ({ ...f, reading_time: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">讀數 (參數 / 數值)</label>
              {readingForm.readings.map((r, idx) => (
                <div key={idx} className="flex gap-2">
                  <Input placeholder="參數名稱" value={r.key} onChange={(e) => updateReadingRow(idx, 'key', e.target.value)} />
                  <Input placeholder="數值" type="number" value={r.value} onChange={(e) => updateReadingRow(idx, 'value', e.target.value)} />
                </div>
              ))}
              <Button variant="ghost" size="sm" onClick={addReadingRow}>+ 新增參數</Button>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">備註</label>
              <Input value={readingForm.notes} onChange={(e) => setReadingForm((f) => ({ ...f, notes: e.target.value }))} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreateReading(false)}>取消</Button>
            <Button
              onClick={() => createReadingMutation.mutate()}
              disabled={!readingForm.monitoring_point_id || !readingForm.reading_time || createReadingMutation.isPending}
            >
              記錄
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
