import React, { useState, useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { GuestHide } from '@/components/ui/guest-hide'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource, treatmentDrugApi, AnimalVaccination } from '@/lib/api'
import { uiLocale } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Plus, Edit2, Trash2, Syringe, Loader2 } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'
import { getOtherOption, vaccineLabel, type VaccinationOption } from './vaccinationConstants'

interface VaccinationsTabProps {
  animalId: string
  earTag: string
  afterParam: string
  vaccinations: AnimalVaccination[] | undefined
}

interface VaccinationFormState {
  administered_date: string
  vaccineCode: string
  vaccineOther: string
  dewormerDrug: string
  dewormerOther: string
  dewormerDose: string
  /** 編輯既有紀錄時，若既有 deworming_dose 解析不出「藥名＋劑量」，原樣保留在這裡；
   * 使用者實際修改驅蟲相關欄位前都直接送出原字串，避免因無法補上劑量而卡住儲存其他欄位（如日期） */
  dewormerLegacyRaw: string | null
}

function emptyVaccinationForm(): VaccinationFormState {
  return {
    administered_date: new Date().toISOString().split('T')[0],
    vaccineCode: '',
    vaccineOther: '',
    dewormerDrug: '',
    dewormerOther: '',
    dewormerDose: '',
    dewormerLegacyRaw: null,
  }
}

/** 既有 vaccine 值 → 表單欄位（代碼命中清單則選中，否則歸入「其他」保留原文字）。
 *  options 為即時清單（含附註備援已合併，見呼叫端 useMemo）。 */
function parseVaccineValue(raw: string | null | undefined, options: readonly VaccinationOption[]): { code: string; other: string } {
  if (!raw) return { code: '', other: '' }
  // OTHER_OPTION 本身也在 options 裡（UI 端附加），須排除，否則舊資料剛好等於 'OTHER'
  // 字面值時會被誤判為「已知代碼」而非自由文字，讓 other 留空、表單因此卡住無法儲存
  const known = options.some((o) => o.value !== 'OTHER' && o.value === raw)
  return known ? { code: raw, other: '' } : { code: 'OTHER', other: raw }
}

/** 既有 deworming_dose 合成字串 → 表單欄位（best-effort 解析「藥名 數字ml」；解析不出來就整串歸入「其他」藥名，劑量留空待補） */
function parseDewormingValue(raw: string | null | undefined, options: readonly VaccinationOption[]): { drug: string; other: string; dose: string } {
  if (!raw) return { drug: '', other: '', dose: '' }
  const match = raw.trim().match(/^(.*?)\s+([\d.]+)\s*m?l$/i)
  if (match) {
    const name = match[1].trim()
    const dose = match[2]
    // 排除 OTHER_OPTION（label「其他」）：既有值若剛好是「其他 1ml」這種自訂藥名，
    // 不能被誤判為命中選單而回傳 other=''——會讓 isVaccinationFormValid 判定表單
    // 不合法（OTHER 分支要求 other 非空），連日期都改不了（2026-08 Qodo 審查發現）
    const known = options.find((o) => o.value !== 'OTHER' && o.label.toLowerCase() === name.toLowerCase())
    if (known) return { drug: known.value, other: '', dose }
    return { drug: 'OTHER', other: name, dose }
  }
  return { drug: 'OTHER', other: raw, dose: '' }
}

function composeVaccineValue(code: string, other: string): string {
  if (!code) return ''
  return code === 'OTHER' ? other.trim() : code
}

/** options 用來把選中的藥名代碼（如 ivermectin）轉成顯示全稱（如 Ivermectin）組進合成字串；
 *  查無時退回代碼原文，不再硬編碼單一藥物特例（未來藥物選單新增品項無需改這裡） */
function composeDewormingValue(drug: string, other: string, dose: string, options: readonly VaccinationOption[]): string {
  if (!drug || !dose) return ''
  const name = drug === 'OTHER' ? other.trim() : (options.find((o) => o.value === drug)?.label ?? drug)
  return name ? `${name} ${dose}ml` : ''
}

/** 劑量須為有限、非負的數字（type="number" 只擋得住瀏覽器 UI，程式化送出或貼上文字仍要擋） */
function isValidDose(dose: string): boolean {
  if (!dose) return false
  const n = Number(dose)
  return Number.isFinite(n) && n >= 0
}

function toVaccinationPayload(f: VaccinationFormState, dewormerOptions: readonly VaccinationOption[]) {
  const deworming_dose = f.dewormerLegacyRaw ?? composeDewormingValue(f.dewormerDrug, f.dewormerOther, f.dewormerDose, dewormerOptions)
  return {
    administered_date: f.administered_date,
    vaccine: composeVaccineValue(f.vaccineCode, f.vaccineOther) || null,
    deworming_dose: deworming_dose || null,
  }
}

function isVaccinationFormValid(f: VaccinationFormState): boolean {
  if (!f.administered_date) return false
  if (f.vaccineCode === 'OTHER' && !f.vaccineOther.trim()) return false
  // 既有無法解析的驅蟲舊資料維持原樣送出，不用結構化規則卡住其他欄位的編輯
  if (f.dewormerLegacyRaw === null) {
    if (f.dewormerDrug === 'OTHER' && !f.dewormerOther.trim()) return false
    if (f.dewormerDrug && !isValidDose(f.dewormerDose)) return false
    if (!f.dewormerDrug && f.dewormerDose) return false
  }
  return true
}

export const VaccinationsTab = React.memo(function VaccinationsTab({ animalId, earTag, afterParam: _afterParam, vaccinations }: VaccinationsTabProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { sortedData, sort, toggleSort } = useTableSort(vaccinations)

  // 藥物選單主檔（與 DrugCombobox 共用 queryKey，同頁若已載入可直接吃快取）。
  // retry: false——委託人/計畫主持人/Study Director 沒有這支 API 要求的
  // animal.animal.view_all，403 重試 3 次沒有意義，直接落回附註備援即可。
  const { data: drugOptionsRaw } = useQuery({
    queryKey: ['treatment-drugs'],
    queryFn: async () => (await treatmentDrugApi.list()).data,
    retry: false,
    staleTime: 5 * 60 * 1000,
  })

  // name !== 'OTHER'：防呆——若管理員之後在藥物選單頁建立一個真的叫「OTHER」的
  // 品項，避免跟下面手動附加的 OTHER_OPTION 撞出重複 value/key（2026-08 PR-Agent 審查發現）
  const vaccineOptions = useMemo((): VaccinationOption[] => [
    ...(drugOptionsRaw ?? [])
      .filter((o) => o.category === '疫苗' && o.name !== 'OTHER')
      .map((o) => ({ value: o.name, label: o.display_name || o.name })),
    getOtherOption(t),
  ], [drugOptionsRaw, t])

  const dewormerOptions = useMemo((): VaccinationOption[] => [
    ...(drugOptionsRaw ?? [])
      .filter((o) => o.category === '驅蟲' && o.name !== 'OTHER')
      .map((o) => ({ value: o.name, label: o.display_name || o.name })),
    getOtherOption(t),
  ], [drugOptionsRaw, t])

  const [showAddDialog, setShowAddDialog] = useState(false)
  const [newVaccination, setNewVaccination] = useState<VaccinationFormState>(emptyVaccinationForm())
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null)
  const [editTarget, setEditTarget] = useState<AnimalVaccination | null>(null)
  const [editForm, setEditForm] = useState<VaccinationFormState>(emptyVaccinationForm())

  const openEdit = useCallback((vac: AnimalVaccination) => {
    setEditTarget(vac)
    const v = parseVaccineValue(vac.vaccine, vaccineOptions)
    const d = parseDewormingValue(vac.deworming_dose, dewormerOptions)
    setEditForm({
      administered_date: vac.administered_date.split('T')[0],
      vaccineCode: v.code,
      vaccineOther: v.other,
      dewormerDrug: d.drug,
      dewormerOther: d.other,
      dewormerDose: d.dose,
      dewormerLegacyRaw: !d.dose && vac.deworming_dose ? vac.deworming_dose : null,
    })
  }, [vaccineOptions, dewormerOptions])

  const addMutation = useMutation({
    mutationFn: async (data: VaccinationFormState) => {
      return api.post(`/animals/${animalId}/vaccinations`, toVaccinationPayload(data, dewormerOptions))
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-vaccinations', animalId] })
      toast({ title: t('common.success'), description: t('animalRecords.vaccinations.added') })
      setShowAddDialog(false)
      setNewVaccination(emptyVaccinationForm())
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalRecords.shared.createFailed')),
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: VaccinationFormState }) => {
      return api.put(`/vaccinations/${id}`, toVaccinationPayload(data, dewormerOptions))
    },
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['animal-vaccinations', animalId] })
      toast({ title: t('common.success'), description: t('animalRecords.vaccinations.updated') })
      // 使用者可能在請求進行中切去編輯另一筆；只有目前開啟的仍是同一筆時才清空 dialog
      if (editTarget?.id === variables.id) {
        setEditTarget(null)
        setEditForm(emptyVaccinationForm())
      }
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalRecords.shared.updateFailed')),
        variant: 'destructive',
      })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: string; reason: string }) => {
      return deleteResource(`/vaccinations/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-vaccinations', animalId] })
      toast({ title: t('common.success'), description: t('animalRecords.vaccinations.deleted') })
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

  return (
    <>
      <Card className="overflow-hidden">
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>{t('animalDetail.tabs.vaccinations')}</CardTitle>
            <CardDescription>{t('animalRecords.vaccinations.description')}</CardDescription>
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
              <Table className="w-full" style={{ minWidth: 530 }}>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <SortableTableHead style={{ width: 100 }} sortKey="administered_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('animalRecords.vaccinations.administeredDate')}</SortableTableHead>
                    <SortableTableHead style={{ minWidth: 120 }} sortKey="vaccine" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('animalRecords.vaccinations.vaccine')}</SortableTableHead>
                    <TableHead style={{ minWidth: 120 }}>{t('animalRecords.vaccinations.dewormerDose')}</TableHead>
                    <TableHead style={{ width: 100 }}>{t('animalRecords.shared.recorder')}</TableHead>
                    <SortableTableHead style={{ width: 160 }} sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="hidden @[690px]:table-cell">{t('animalRecords.shared.createdAt')}</SortableTableHead>
                    <TableHead style={{ width: 90 }} className="text-right">{t('common.actions')}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {!vaccinations || vaccinations.length === 0 ? (
                    <TableEmptyRow colSpan={6} icon={Syringe} title={t('animalRecords.vaccinations.emptyTitle')} />
                  ) : (
                    sortedData?.map((vac) => (
                      <TableRow key={vac.id}>
                        <TableCell style={{ width: 100 }} className="whitespace-nowrap">{new Date(vac.administered_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })}</TableCell>
                        <TableCell style={{ minWidth: 120 }} className="whitespace-normal break-words">{vaccineLabel(vac.vaccine, vaccineOptions, t) || '-'}</TableCell>
                        <TableCell style={{ minWidth: 120 }} className="whitespace-normal break-words">{vac.deworming_dose || '-'}</TableCell>
                        <TableCell style={{ width: 100 }} className="whitespace-normal break-words">{vac.created_by_name || '-'}</TableCell>
                        <TableCell style={{ width: 160 }} className="text-xs text-muted-foreground hidden @[690px]:table-cell">{new Date(vac.created_at).toLocaleString(uiLocale(), { timeZone: 'Asia/Taipei' })}</TableCell>
                        <TableCell style={{ width: 90 }} className="text-right">
                          <div className="flex items-center justify-end gap-1">
                            <GuestHide>
                              <Can permission={PERMISSIONS.ANIMAL_RECORD_EDIT}>
                                <Button variant="ghost" size="icon" onClick={() => openEdit(vac)} aria-label={t('common.edit')}>
                                  <Edit2 className="h-4 w-4" />
                                </Button>
                              </Can>
                              <Can permission={PERMISSIONS.ANIMAL_RECORD_DELETE}>
                                <Button variant="ghost" size="icon" onClick={() => setDeleteTarget(vac.id)} aria-label={t('common.delete')}>
                                  <Trash2 className="h-4 w-4 text-status-error-solid" />
                                </Button>
                              </Can>
                            </GuestHide>
                          </div>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>

            {/* ── Card view: container < 600px ── */}
            <div className="@[600px]:hidden space-y-3 py-1">
              {!vaccinations || vaccinations.length === 0 ? (
                <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                  <Syringe className="h-8 w-8" />
                  <p className="text-sm">{t('animalRecords.vaccinations.emptyTitle')}</p>
                </div>
              ) : (
                sortedData?.map((vac) => (
                  <div key={vac.id} className="rounded-lg border bg-card p-3 space-y-2">
                    <div className="text-sm font-medium text-foreground">
                      {new Date(vac.administered_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })}
                    </div>
                    {vac.vaccine && (
                      <div className="text-sm text-muted-foreground">
                        💉 {t('animalRecords.vaccinations.cardVaccine', { value: vaccineLabel(vac.vaccine, vaccineOptions, t) })}
                      </div>
                    )}
                    {vac.deworming_dose && (
                      <div className="text-sm text-muted-foreground">
                        💊 {t('animalRecords.vaccinations.cardDeworming', { value: vac.deworming_dose })}
                      </div>
                    )}
                    <div className="flex items-center justify-between gap-2 pt-1 border-t">
                      <span className="text-xs text-muted-foreground">{vac.created_by_name || '-'}</span>
                      <div className="flex gap-0.5">
                        <GuestHide>
                          <Can permission={PERMISSIONS.ANIMAL_RECORD_EDIT}>
                            <Button variant="ghost" size="icon" onClick={() => openEdit(vac)} aria-label={t('common.edit')}>
                              <Edit2 className="h-4 w-4" />
                            </Button>
                          </Can>
                          <Can permission={PERMISSIONS.ANIMAL_RECORD_DELETE}>
                            <Button variant="ghost" size="icon" onClick={() => setDeleteTarget(vac.id)} aria-label={t('common.delete')}>
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

      <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('animalRecords.vaccinations.addTitle')}</DialogTitle>
            <DialogDescription>{t('animalRecords.shared.earTagLine', { earTag })}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="vac_date">{t('animalRecords.vaccinations.administeredDateRequired')}</Label>
              <Input
                id="vac_date"
                type="date"
                value={newVaccination.administered_date}
                onChange={(e) => setNewVaccination({ ...newVaccination, administered_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="vaccine">{t('animalRecords.vaccinations.vaccine')}</Label>
              <Select
                value={newVaccination.vaccineCode}
                onValueChange={(v) => setNewVaccination({ ...newVaccination, vaccineCode: v, vaccineOther: v === 'OTHER' ? newVaccination.vaccineOther : '' })}
              >
                <SelectTrigger id="vaccine">
                  <SelectValue placeholder={t('animalRecords.vaccinations.selectVaccine')} />
                </SelectTrigger>
                <SelectContent>
                  {vaccineOptions.map((o) => (
                    <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {newVaccination.vaccineCode === 'OTHER' && (
                <Input
                  id="vaccine_other"
                  aria-label={t('animalRecords.vaccinations.customVaccineName')}
                  value={newVaccination.vaccineOther}
                  onChange={(e) => setNewVaccination({ ...newVaccination, vaccineOther: e.target.value })}
                  placeholder={t('animalRecords.vaccinations.enterVaccineName')}
                />
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor="dewormer_drug">{t('animalRecords.vaccinations.deworming')}</Label>
              <Select
                value={newVaccination.dewormerDrug}
                onValueChange={(v) => setNewVaccination({ ...newVaccination, dewormerDrug: v, dewormerOther: v === 'OTHER' ? newVaccination.dewormerOther : '', dewormerLegacyRaw: null })}
              >
                <SelectTrigger id="dewormer_drug">
                  <SelectValue placeholder={t('animalRecords.vaccinations.selectDrug')} />
                </SelectTrigger>
                <SelectContent>
                  {dewormerOptions.map((o) => (
                    <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {newVaccination.dewormerDrug === 'OTHER' && (
                <Input
                  id="dewormer_other"
                  aria-label={t('animalRecords.vaccinations.customDewormerName')}
                  value={newVaccination.dewormerOther}
                  onChange={(e) => setNewVaccination({ ...newVaccination, dewormerOther: e.target.value, dewormerLegacyRaw: null })}
                  placeholder={t('animalRecords.vaccinations.enterDrugName')}
                />
              )}
              <div className="flex items-center gap-2">
                <Input
                  id="dewormer_dose"
                  aria-label={t('animalRecords.vaccinations.dewormerDoseAria')}
                  type="number"
                  step="0.1"
                  min="0"
                  value={newVaccination.dewormerDose}
                  onChange={(e) => setNewVaccination({ ...newVaccination, dewormerDose: e.target.value, dewormerLegacyRaw: null })}
                  placeholder={t('animalRecords.shared.dosePlaceholder')}
                />
                <span className="text-sm text-muted-foreground shrink-0">ml</span>
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAddDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={() => addMutation.mutate(newVaccination)}
              disabled={addMutation.isPending || !isVaccinationFormValid(newVaccination)}
              className="bg-status-success-solid hover:bg-status-success-solid/90"
            >
              {addMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={editTarget !== null} onOpenChange={(open) => {
        if (!open) {
          setEditTarget(null)
          updateMutation.reset()
        }
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('animalRecords.vaccinations.editTitle')}</DialogTitle>
            <DialogDescription>{t('animalRecords.shared.earTagLine', { earTag })}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="edit_vac_date">{t('animalRecords.vaccinations.administeredDateRequired')}</Label>
              <Input
                id="edit_vac_date"
                type="date"
                value={editForm.administered_date}
                onChange={(e) => setEditForm({ ...editForm, administered_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit_vaccine">{t('animalRecords.vaccinations.vaccine')}</Label>
              <Select
                value={editForm.vaccineCode}
                onValueChange={(v) => setEditForm({ ...editForm, vaccineCode: v, vaccineOther: v === 'OTHER' ? editForm.vaccineOther : '' })}
              >
                <SelectTrigger id="edit_vaccine">
                  <SelectValue placeholder={t('animalRecords.vaccinations.selectVaccine')} />
                </SelectTrigger>
                <SelectContent>
                  {vaccineOptions.map((o) => (
                    <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {editForm.vaccineCode === 'OTHER' && (
                <Input
                  id="edit_vaccine_other"
                  aria-label={t('animalRecords.vaccinations.customVaccineName')}
                  value={editForm.vaccineOther}
                  onChange={(e) => setEditForm({ ...editForm, vaccineOther: e.target.value })}
                  placeholder={t('animalRecords.vaccinations.enterVaccineName')}
                />
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit_dewormer_drug">{t('animalRecords.vaccinations.deworming')}</Label>
              <Select
                value={editForm.dewormerDrug}
                onValueChange={(v) => setEditForm({ ...editForm, dewormerDrug: v, dewormerOther: v === 'OTHER' ? editForm.dewormerOther : '', dewormerLegacyRaw: null })}
              >
                <SelectTrigger id="edit_dewormer_drug">
                  <SelectValue placeholder={t('animalRecords.vaccinations.selectDrug')} />
                </SelectTrigger>
                <SelectContent>
                  {dewormerOptions.map((o) => (
                    <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {editForm.dewormerDrug === 'OTHER' && (
                <Input
                  id="edit_dewormer_other"
                  aria-label={t('animalRecords.vaccinations.customDewormerName')}
                  value={editForm.dewormerOther}
                  onChange={(e) => setEditForm({ ...editForm, dewormerOther: e.target.value, dewormerLegacyRaw: null })}
                  placeholder={t('animalRecords.vaccinations.enterDrugName')}
                />
              )}
              <div className="flex items-center gap-2">
                <Input
                  id="edit_dewormer_dose"
                  aria-label={t('animalRecords.vaccinations.dewormerDoseAria')}
                  type="number"
                  step="0.1"
                  min="0"
                  value={editForm.dewormerDose}
                  onChange={(e) => setEditForm({ ...editForm, dewormerDose: e.target.value, dewormerLegacyRaw: null })}
                  placeholder={t('animalRecords.shared.dosePlaceholder')}
                />
                <span className="text-sm text-muted-foreground shrink-0">ml</span>
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditTarget(null)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={() => editTarget && updateMutation.mutate({ id: editTarget.id, data: editForm })}
              disabled={updateMutation.isPending || !isVaccinationFormValid(editForm)}
              className="bg-status-success-solid hover:bg-status-success-solid/90"
            >
              {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <DeleteReasonDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        copy={{ title: t('animalRecords.vaccinations.deleteTitle'), description: t('animalRecords.shared.deleteRecordDescription') }}
        onConfirm={(reason) => deleteMutation.mutate({ id: deleteTarget!, reason })}
        isPending={deleteMutation.isPending}
      />
    </>
  )
})
