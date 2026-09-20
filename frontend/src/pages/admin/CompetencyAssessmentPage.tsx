import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { useAuthHasPermission } from '@/stores/auth'
import {
  listCompetencyAssessments,
  createCompetencyAssessment,
} from '@/lib/api/glpCompliance'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
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
import { Plus, ClipboardCheck } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'

// labelKey 是 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const ASSESSMENT_TYPES = [
  { value: 'initial', labelKey: 'adminGlp.competencyAssessment.assessmentType.initial' },
  { value: 'periodic', labelKey: 'adminGlp.competencyAssessment.assessmentType.periodic' },
  { value: 'requalification', labelKey: 'adminGlp.competencyAssessment.assessmentType.requalification' },
]

const RESULTS = [
  { value: 'competent', labelKey: 'adminGlp.competencyAssessment.result.competent' },
  { value: 'not_yet_competent', labelKey: 'adminGlp.competencyAssessment.result.notYetCompetent' },
  { value: 'requires_supervision', labelKey: 'adminGlp.competencyAssessment.result.requiresSupervision' },
]

const METHODS = [
  { value: 'observation', labelKey: 'adminGlp.competencyAssessment.method.observation' },
  { value: 'written_test', labelKey: 'adminGlp.competencyAssessment.method.writtenTest' },
  { value: 'practical_test', labelKey: 'adminGlp.competencyAssessment.method.practicalTest' },
  { value: 'peer_review', labelKey: 'adminGlp.competencyAssessment.method.peerReview' },
]

const RESULT_VARIANTS: Record<string, 'default' | 'secondary' | 'destructive' | 'success' | 'warning'> = {
  competent: 'success',
  not_yet_competent: 'destructive',
  requires_supervision: 'warning',
}

const INITIAL_FORM = {
  user_id: '',
  assessment_type: 'initial',
  skill_area: '',
  assessment_date: '',
  result: 'competent',
  method: 'observation',
  score: '',
  valid_until: '',
}

export function CompetencyAssessmentPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
  const canManage = hasPermission('competency.assessment.manage')

  const [filterResult, setFilterResult] = useState<string>('')
  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState(INITIAL_FORM)

  const { data: assessments = [], isLoading } = useQuery({
    queryKey: ['competency-assessments', filterResult],
    queryFn: () => listCompetencyAssessments({ result: filterResult || undefined }),
  })

  const createMutation = useMutation({
    mutationFn: () =>
      createCompetencyAssessment({
        user_id: form.user_id,
        assessment_type: form.assessment_type,
        skill_area: form.skill_area,
        assessment_date: form.assessment_date,
        result: form.result,
        method: form.method || undefined,
        score: form.score ? Number(form.score) : undefined,
        valid_until: form.valid_until || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['competency-assessments'] })
      setShowCreate(false)
      setForm(INITIAL_FORM)
      toast({ title: t('adminGlp.competencyAssessment.toast.created') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.shared.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t('adminGlp.competencyAssessment.title')}</h1>
          <p className="text-muted-foreground">{t('adminGlp.competencyAssessment.subtitle')}</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            {t('adminGlp.competencyAssessment.create')}
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <div className="flex gap-4">
            <Select value={filterResult} onValueChange={setFilterResult}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder={t('adminGlp.competencyAssessment.allResults')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">{t('adminGlp.competencyAssessment.allResults')}</SelectItem>
                {RESULTS.map((r) => (
                  <SelectItem key={r.value} value={r.value}>{t(r.labelKey)}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('adminGlp.competencyAssessment.col.assessee')}</TableHead>
                <TableHead>{t('adminGlp.competencyAssessment.col.skillArea')}</TableHead>
                <TableHead>{t('adminGlp.competencyAssessment.col.assessmentType')}</TableHead>
                <TableHead>{t('adminGlp.competencyAssessment.col.assessmentDate')}</TableHead>
                <TableHead>{t('adminGlp.competencyAssessment.col.result')}</TableHead>
                <TableHead>{t('adminGlp.competencyAssessment.col.score')}</TableHead>
                <TableHead>{t('adminGlp.competencyAssessment.col.assessor')}</TableHead>
                <TableHead>{t('adminGlp.competencyAssessment.col.validUntil')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={8} className="p-0"><TableSkeleton rows={8} cols={8} /></TableCell></TableRow>
              ) : assessments.length === 0 ? (
                <TableEmptyRow colSpan={8} icon={ClipboardCheck} title={t('adminGlp.competencyAssessment.empty')} />
              ) : (
                assessments.map((a) => {
                  const assessmentType = ASSESSMENT_TYPES.find((at) => at.value === a.assessment_type)
                  const result = RESULTS.find((r) => r.value === a.result)
                  return (
                    <TableRow key={a.id}>
                      <TableCell className="font-medium">{a.user_name ?? a.user_id}</TableCell>
                      <TableCell>{a.skill_area}</TableCell>
                      <TableCell>{assessmentType ? t(assessmentType.labelKey) : a.assessment_type}</TableCell>
                      <TableCell>{a.assessment_date}</TableCell>
                      <TableCell>
                        <Badge variant={RESULT_VARIANTS[a.result] ?? 'secondary'}>
                          {result ? t(result.labelKey) : a.result}
                        </Badge>
                      </TableCell>
                      <TableCell>{a.score ?? '-'}</TableCell>
                      <TableCell>{a.assessor_name ?? '-'}</TableCell>
                      <TableCell>{a.valid_until ?? '-'}</TableCell>
                    </TableRow>
                  )
                })
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('adminGlp.competencyAssessment.dialog.title')}</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.competencyAssessment.dialog.assesseeIdRequired')}</label>
              <Input value={form.user_id} onChange={(e) => setForm((f) => ({ ...f, user_id: e.target.value }))} placeholder={t('adminGlp.competencyAssessment.dialog.assesseePlaceholder')} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.competencyAssessment.dialog.skillAreaRequired')}</label>
              <Input value={form.skill_area} onChange={(e) => setForm((f) => ({ ...f, skill_area: e.target.value }))} />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">{t('adminGlp.competencyAssessment.dialog.assessmentTypeRequired')}</label>
                <Select value={form.assessment_type} onValueChange={(v) => setForm((f) => ({ ...f, assessment_type: v }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {ASSESSMENT_TYPES.map((at) => (
                      <SelectItem key={at.value} value={at.value}>{t(at.labelKey)}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">{t('adminGlp.competencyAssessment.dialog.assessmentMethod')}</label>
                <Select value={form.method} onValueChange={(v) => setForm((f) => ({ ...f, method: v }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {METHODS.map((m) => (
                      <SelectItem key={m.value} value={m.value}>{t(m.labelKey)}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.competencyAssessment.dialog.assessmentDateRequired')}</label>
              <Input type="date" value={form.assessment_date} onChange={(e) => setForm((f) => ({ ...f, assessment_date: e.target.value }))} />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">{t('adminGlp.competencyAssessment.dialog.resultRequired')}</label>
                <Select value={form.result} onValueChange={(v) => setForm((f) => ({ ...f, result: v }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {RESULTS.map((r) => (
                      <SelectItem key={r.value} value={r.value}>{t(r.labelKey)}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">{t('adminGlp.competencyAssessment.col.score')}</label>
                <Input type="number" value={form.score} onChange={(e) => setForm((f) => ({ ...f, score: e.target.value }))} />
              </div>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.competencyAssessment.col.validUntil')}</label>
              <Input type="date" value={form.valid_until} onChange={(e) => setForm((f) => ({ ...f, valid_until: e.target.value }))} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>{t('common.cancel')}</Button>
            <Button
              onClick={() => createMutation.mutate()}
              disabled={!form.user_id || !form.skill_area || !form.assessment_date || createMutation.isPending}
            >
              {t('adminGlp.shared.create')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
