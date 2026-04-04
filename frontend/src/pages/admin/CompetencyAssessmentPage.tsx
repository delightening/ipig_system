import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import { useAuthStore } from '@/stores/auth'
import {
  listCompetencyAssessments,
  createCompetencyAssessment,
  type CompetencyAssessment,
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
import { useTranslation } from 'react-i18next'
import { Plus } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

const ASSESSMENT_TYPES = [
  { value: 'initial', label: '初次評鑑' },
  { value: 'periodic', label: '定期評鑑' },
  { value: 'requalification', label: '重新資格認定' },
]

const RESULTS = [
  { value: 'competent', label: '合格' },
  { value: 'not_yet_competent', label: '不合格' },
  { value: 'requires_supervision', label: '需監督' },
]

const METHODS = [
  { value: 'observation', label: '觀察' },
  { value: 'written_test', label: '筆試' },
  { value: 'practical_test', label: '實作測驗' },
  { value: 'peer_review', label: '同儕審查' },
]

const RESULT_COLORS: Record<string, string> = {
  competent: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  not_yet_competent: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
  requires_supervision: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
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
  const { hasPermission } = useAuthStore()
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
      toast({ title: '能力評鑑已建立' })
    },
    onError: (err: unknown) => toast({ title: '建立失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">能力評鑑</h1>
          <p className="text-muted-foreground">ISO 17025 / ISO 9001 人員能力管理</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            新增評鑑
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <div className="flex gap-4">
            <Select value={filterResult} onValueChange={setFilterResult}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="所有結果" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">所有結果</SelectItem>
                {RESULTS.map((r) => (
                  <SelectItem key={r.value} value={r.value}>{r.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>受評人</TableHead>
                <TableHead>技能領域</TableHead>
                <TableHead>評鑑類型</TableHead>
                <TableHead>評鑑日期</TableHead>
                <TableHead>結果</TableHead>
                <TableHead>分數</TableHead>
                <TableHead>評鑑者</TableHead>
                <TableHead>有效至</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={8} className="text-center py-8">載入中...</TableCell></TableRow>
              ) : assessments.length === 0 ? (
                <TableRow><TableCell colSpan={8} className="text-center py-8 text-muted-foreground">尚無評鑑紀錄</TableCell></TableRow>
              ) : (
                assessments.map((a) => (
                  <TableRow key={a.id}>
                    <TableCell className="font-medium">{a.user_name ?? a.user_id}</TableCell>
                    <TableCell>{a.skill_area}</TableCell>
                    <TableCell>{ASSESSMENT_TYPES.find((at) => at.value === a.assessment_type)?.label ?? a.assessment_type}</TableCell>
                    <TableCell>{a.assessment_date}</TableCell>
                    <TableCell>
                      <Badge className={RESULT_COLORS[a.result] ?? ''}>
                        {RESULTS.find((r) => r.value === a.result)?.label ?? a.result}
                      </Badge>
                    </TableCell>
                    <TableCell>{a.score ?? '-'}</TableCell>
                    <TableCell>{a.assessor_name ?? '-'}</TableCell>
                    <TableCell>{a.valid_until ?? '-'}</TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增能力評鑑</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">受評人 ID *</label>
              <Input value={form.user_id} onChange={(e) => setForm((f) => ({ ...f, user_id: e.target.value }))} placeholder="使用者 UUID" />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">技能領域 *</label>
              <Input value={form.skill_area} onChange={(e) => setForm((f) => ({ ...f, skill_area: e.target.value }))} />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">評鑑類型 *</label>
                <Select value={form.assessment_type} onValueChange={(v) => setForm((f) => ({ ...f, assessment_type: v }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {ASSESSMENT_TYPES.map((at) => (
                      <SelectItem key={at.value} value={at.value}>{at.label}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">評鑑方法</label>
                <Select value={form.method} onValueChange={(v) => setForm((f) => ({ ...f, method: v }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {METHODS.map((m) => (
                      <SelectItem key={m.value} value={m.value}>{m.label}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">評鑑日期 *</label>
              <Input type="date" value={form.assessment_date} onChange={(e) => setForm((f) => ({ ...f, assessment_date: e.target.value }))} />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">結果 *</label>
                <Select value={form.result} onValueChange={(v) => setForm((f) => ({ ...f, result: v }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {RESULTS.map((r) => (
                      <SelectItem key={r.value} value={r.value}>{r.label}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">分數</label>
                <Input type="number" value={form.score} onChange={(e) => setForm((f) => ({ ...f, score: e.target.value }))} />
              </div>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">有效至</label>
              <Input type="date" value={form.valid_until} onChange={(e) => setForm((f) => ({ ...f, valid_until: e.target.value }))} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>取消</Button>
            <Button
              onClick={() => createMutation.mutate()}
              disabled={!form.user_id || !form.skill_area || !form.assessment_date || createMutation.isPending}
            >
              建立
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
