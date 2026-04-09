import { useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { AnimalStatus, animalStatusNames, animalFieldCorrectionApi } from '@/lib/api'
import type { AnimalEditFormData } from '@/lib/validation'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { ArrowLeft, Loader2, Save, AlertCircle, FileEdit } from 'lucide-react'
import { RequestCorrectionDialog } from '@/components/animal/RequestCorrectionDialog'
import { SearchableSelect } from '@/components/ui/searchable-select'
import { AnimalEditReadOnlyFields } from './components/AnimalEditReadOnlyFields'
import { useAnimalEdit } from './hooks/useAnimalEdit'

export function AnimalEditPage() {
  const { id } = useParams<{ id: string }>()
  const animalId = id!
  const [correctionDialogOpen, setCorrectionDialogOpen] = useState(false)

  const { form, animal, animalLoading, sources, pens, approvedProtocols, updateMutation, navigate, queryClient } = useAnimalEdit(animalId)
  const { register, handleSubmit, watch, setValue, formState: { errors, isDirty } } = form

  const watchedStatus = watch('status')
  const watchedIacucNo = watch('iacuc_no')
  const watchedPenLocation = watch('pen_location')

  const penOptions = (pens ?? []).map((p) => ({
    value: p.code,
    label: p.code,
    description: p.name ?? undefined,
  }))

  const onValid = (data: AnimalEditFormData) => {
    if (data.pen_location && pens && !pens.some((p) => p.code === data.pen_location)) {
      form.setError('pen_location', { message: '此欄號不存在，請重新選擇' })
      return
    }
    updateMutation.mutate(data)
  }

  if (animalLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!animal) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px]">
        <AlertCircle className="h-12 w-12 text-muted-foreground mb-4" />
        <p className="text-muted-foreground">找不到此動物</p>
        <Button variant="outline" className="mt-4" onClick={() => navigate('/animals')}>返回列表</Button>
      </div>
    )
  }

  return (
    <div className="space-y-6 max-w-4xl mx-auto">
      <div className="flex items-center justify-between">
        <Link to={`/animals/${animalId}`} className="inline-flex items-center text-muted-foreground hover:text-foreground">
          <ArrowLeft className="h-4 w-4 mr-2" />
          回到動物詳情
        </Link>
      </div>

      <PageHeader
        title="編輯動物資料"
        description={`耳號：${animal.ear_tag}`}
        actions={
          <Button size="sm" type="button" variant="outline" onClick={() => setCorrectionDialogOpen(true)}
            className="text-status-warning-text border-status-warning-border hover:bg-status-warning-bg">
            <FileEdit className="h-4 w-4 mr-2" />
            申請修正（耳號/出生日期/性別/品種）
          </Button>
        }
      />

      <RequestCorrectionDialog
        open={correctionDialogOpen}
        onOpenChange={setCorrectionDialogOpen}
        animal={animal}
        onSubmit={async (data) => {
          await animalFieldCorrectionApi.create(animalId, data)
          queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
          toast({ title: '成功', description: '修正申請已提交，待管理員審核' })
        }}
      />

      <form onSubmit={handleSubmit(onValid)}>
        <Card>
          <CardHeader>
            <CardTitle>基本資料</CardTitle>
            <CardDescription>編輯動物的基本資訊</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-6">
              <AnimalEditReadOnlyFields animal={animal} sources={sources} />

              {/* 狀態 */}
              <div className="space-y-2">
                <Label>狀態 *</Label>
                <Select value={watchedStatus ?? ''} onValueChange={(v) => setValue('status', v as AnimalStatus, { shouldValidate: true, shouldDirty: true })}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {Object.entries(animalStatusNames)
                      .filter(([value]) => value !== 'transferred' && value !== 'sudden_death')
                      .map(([value, label]) => <SelectItem key={value} value={value}>{label}</SelectItem>)}
                  </SelectContent>
                </Select>
                {errors.status && <p className="text-sm text-destructive">{errors.status.message}</p>}
              </div>

              {/* 欄位 */}
              <div className="space-y-2">
                <Label>欄位</Label>
                <SearchableSelect
                  options={penOptions}
                  value={watchedPenLocation ?? ''}
                  onValueChange={(v) => setValue('pen_location', v, { shouldValidate: true, shouldDirty: true })}
                  placeholder="選擇欄號"
                  searchPlaceholder="輸入欄號搜尋..."
                  emptyMessage="找不到此欄號，請確認欄號正確"
                />
                {errors.pen_location && (
                  <p className="text-sm text-destructive">{errors.pen_location.message}</p>
                )}
              </div>

              {/* IACUC No. */}
              <div className="space-y-2">
                <Label>
                  IACUC No.
                  {watchedStatus === 'in_experiment' && <span className="text-destructive ml-1">*</span>}
                </Label>
                {animal.status === 'in_experiment' ? (
                  <>
                    <Input value={animal.iacuc_no || ''} disabled className="bg-muted" />
                    <p className="text-xs text-status-warning-text">實驗中的動物無法更改 IACUC No.</p>
                  </>
                ) : (
                  <Select value={watchedIacucNo || ''} onValueChange={(v) => setValue('iacuc_no', v === '' ? '' : v, { shouldValidate: true, shouldDirty: true })}>
                    <SelectTrigger><SelectValue placeholder="選擇 IACUC No." /></SelectTrigger>
                    <SelectContent>
                      {approvedProtocols?.map((protocol) => (
                        <SelectItem key={protocol.id} value={protocol.iacuc_no!}>{protocol.iacuc_no}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
                {errors.iacuc_no && <p className="text-sm text-destructive">{errors.iacuc_no.message}</p>}
              </div>

              <div className="space-y-2">
                <Label>實驗日期</Label>
                <Input type="date" {...register('experiment_date')} />
              </div>

              {/* 備註 */}
              <div className="space-y-2 col-span-2">
                <Label htmlFor="remark">備註</Label>
                <Textarea id="remark" {...register('remark')} placeholder="其他備註..." className="min-h-[100px]" />
              </div>
            </div>
          </CardContent>
        </Card>

        <div className="flex items-center justify-end gap-3 mt-6">
          <Button type="button" variant="outline" onClick={() => navigate(`/animals/${animalId}`)}>取消</Button>
          <Button type="submit" disabled={updateMutation.isPending || !isDirty} className="bg-primary hover:bg-primary/90">
            {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            <Save className="h-4 w-4 mr-2" />
            儲存變更
          </Button>
        </div>
      </form>
    </div>
  )
}
