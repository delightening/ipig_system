import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { useAuthHasPermission } from '@/stores/auth'
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
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Plus, Thermometer } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { uiLocale } from '@/lib/utils'

// labelKey 是 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const LOCATION_TYPES = [
  { value: 'animal_room', labelKey: 'adminGlp.environmentMonitoring.locationType.animalRoom' },
  { value: 'lab', labelKey: 'adminGlp.environmentMonitoring.locationType.lab' },
  { value: 'storage', labelKey: 'adminGlp.environmentMonitoring.locationType.storage' },
  { value: 'clean_room', labelKey: 'adminGlp.environmentMonitoring.locationType.cleanRoom' },
]

const INTERVALS = [
  { value: 'continuous', labelKey: 'adminGlp.environmentMonitoring.interval.continuous' },
  { value: 'hourly', labelKey: 'adminGlp.environmentMonitoring.interval.hourly' },
  { value: 'daily', labelKey: 'adminGlp.environmentMonitoring.interval.daily' },
]

const INITIAL_POINT_FORM = { name: '', location_type: 'animal_room', monitoring_interval: 'daily' }
const INITIAL_READING_FORM = { monitoring_point_id: '', reading_time: '', notes: '', readings: [{ key: '', value: '' }] }

export function EnvironmentMonitoringPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
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
      toast({ title: t('adminGlp.environmentMonitoring.toast.pointCreated') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.shared.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
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
      toast({ title: t('adminGlp.environmentMonitoring.toast.readingRecorded') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.environmentMonitoring.toast.recordFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
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
          <h1 className="text-2xl font-bold tracking-tight">{t('adminGlp.environmentMonitoring.title')}</h1>
          <p className="text-muted-foreground">{t('adminGlp.environmentMonitoring.subtitle')}</p>
        </div>
        {canManage && (
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => setShowCreateReading(true)}>
              <Thermometer className="mr-2 h-4 w-4" />
              {t('adminGlp.environmentMonitoring.recordReading')}
            </Button>
            <Button onClick={() => setShowCreatePoint(true)}>
              <Plus className="mr-2 h-4 w-4" />
              {t('adminGlp.environmentMonitoring.addPoint')}
            </Button>
          </div>
        )}
      </div>

      {/* Monitoring Points */}
      <Card>
        <CardHeader><CardTitle>{t('adminGlp.environmentMonitoring.points')}</CardTitle></CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('adminGlp.environmentMonitoring.col.name')}</TableHead>
                <TableHead>{t('adminGlp.environmentMonitoring.col.locationType')}</TableHead>
                <TableHead>{t('adminGlp.environmentMonitoring.col.interval')}</TableHead>
                <TableHead>{t('adminGlp.environmentMonitoring.active')}</TableHead>
                <TableHead>{t('common.actions')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {loadingPoints ? (
                <TableRow><TableCell colSpan={5} className="p-0"><TableSkeleton rows={8} cols={5} /></TableCell></TableRow>
              ) : points.length === 0 ? (
                <TableEmptyRow colSpan={5} icon={Thermometer} title={t('adminGlp.environmentMonitoring.empty')} />
              ) : (
                points.map((p) => {
                  const locationType = LOCATION_TYPES.find((lt) => lt.value === p.location_type)
                  const interval = INTERVALS.find((i) => i.value === p.monitoring_interval)
                  return (
                    <TableRow key={p.id}>
                      <TableCell className="font-medium">{p.name}</TableCell>
                      <TableCell>{locationType ? t(locationType.labelKey) : p.location_type}</TableCell>
                      <TableCell>{interval ? t(interval.labelKey) : p.monitoring_interval ?? '-'}</TableCell>
                      <TableCell>
                        <Badge variant={p.is_active ? 'success' : 'secondary'}>
                          {p.is_active ? t('adminGlp.environmentMonitoring.active') : t('adminGlp.environmentMonitoring.inactive')}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <Button variant="ghost" size="sm" onClick={() => setSelectedPointId(p.id)}>
                          {t('adminGlp.environmentMonitoring.viewReadings')}
                        </Button>
                      </TableCell>
                    </TableRow>
                  )
                })
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
              {t('adminGlp.environmentMonitoring.readingsTitle', { name: points.find((p) => p.id === selectedPointId)?.name ?? '' })}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t('adminGlp.environmentMonitoring.col.readingTime')}</TableHead>
                  <TableHead>{t('adminGlp.environmentMonitoring.col.readings')}</TableHead>
                  <TableHead>{t('adminGlp.environmentMonitoring.outOfRange')}</TableHead>
                  <TableHead>{t('adminGlp.shared.remarks')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {loadingReadings ? (
                  <TableRow><TableCell colSpan={4} className="p-0"><TableSkeleton rows={8} cols={4} /></TableCell></TableRow>
                ) : readings.length === 0 ? (
                  <TableEmptyRow colSpan={4} icon={Thermometer} title={t('adminGlp.environmentMonitoring.noReadings')} />
                ) : (
                  readings.map((rd) => (
                    <TableRow key={rd.id}>
                      <TableCell>{new Date(rd.reading_time).toLocaleString(uiLocale())}</TableCell>
                      <TableCell className="font-mono text-sm">
                        {Object.entries(rd.readings).map(([k, v]) => `${k}: ${v}`).join(', ')}
                      </TableCell>
                      <TableCell>
                        {rd.is_out_of_range ? (
                          <Badge variant="destructive">{t('adminGlp.environmentMonitoring.outOfRange')}</Badge>
                        ) : (
                          <Badge variant="success">{t('adminGlp.environmentMonitoring.normal')}</Badge>
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
          <DialogHeader><DialogTitle>{t('adminGlp.environmentMonitoring.addPoint')}</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.environmentMonitoring.form.nameRequired')}</label>
              <Input value={pointForm.name} onChange={(e) => setPointForm((f) => ({ ...f, name: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.environmentMonitoring.form.locationTypeRequired')}</label>
              <Select value={pointForm.location_type} onValueChange={(v) => setPointForm((f) => ({ ...f, location_type: v }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {LOCATION_TYPES.map((lt) => (
                    <SelectItem key={lt.value} value={lt.value}>{t(lt.labelKey)}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.environmentMonitoring.form.intervalRequired')}</label>
              <Select value={pointForm.monitoring_interval} onValueChange={(v) => setPointForm((f) => ({ ...f, monitoring_interval: v }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {INTERVALS.map((i) => (
                    <SelectItem key={i.value} value={i.value}>{t(i.labelKey)}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreatePoint(false)}>{t('common.cancel')}</Button>
            <Button onClick={() => createPointMutation.mutate()} disabled={!pointForm.name || createPointMutation.isPending}>
              {t('adminGlp.shared.create')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Create Reading Dialog */}
      <Dialog open={showCreateReading} onOpenChange={setShowCreateReading}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('adminGlp.environmentMonitoring.recordReading')}</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.environmentMonitoring.form.pointRequired')}</label>
              <Select value={readingForm.monitoring_point_id} onValueChange={(v) => setReadingForm((f) => ({ ...f, monitoring_point_id: v }))}>
                <SelectTrigger><SelectValue placeholder={t('adminGlp.environmentMonitoring.form.selectPoint')} /></SelectTrigger>
                <SelectContent>
                  {points.filter((p) => p.is_active).map((p) => (
                    <SelectItem key={p.id} value={p.id}>{p.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.environmentMonitoring.form.readingTimeRequired')}</label>
              <Input type="datetime-local" value={readingForm.reading_time} onChange={(e) => setReadingForm((f) => ({ ...f, reading_time: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.environmentMonitoring.form.readingsLabel')}</label>
              {readingForm.readings.map((r, idx) => (
                <div key={idx} className="flex gap-2">
                  <Input placeholder={t('adminGlp.environmentMonitoring.form.parameterName')} value={r.key} onChange={(e) => updateReadingRow(idx, 'key', e.target.value)} />
                  <Input placeholder={t('adminGlp.environmentMonitoring.form.value')} type="number" value={r.value} onChange={(e) => updateReadingRow(idx, 'value', e.target.value)} />
                </div>
              ))}
              <Button variant="ghost" size="sm" onClick={addReadingRow}>{t('adminGlp.environmentMonitoring.form.addParameter')}</Button>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.shared.remarks')}</label>
              <Input value={readingForm.notes} onChange={(e) => setReadingForm((f) => ({ ...f, notes: e.target.value }))} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreateReading(false)}>{t('common.cancel')}</Button>
            <Button
              onClick={() => createReadingMutation.mutate()}
              disabled={!readingForm.monitoring_point_id || !readingForm.reading_time || createReadingMutation.isPending}
            >
              {t('adminGlp.environmentMonitoring.form.record')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
