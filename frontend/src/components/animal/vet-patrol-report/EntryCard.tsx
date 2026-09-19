// 單筆觀察卡片（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import { useTranslation } from 'react-i18next'

import { Textarea } from '@/components/ui/input'
import { SearchableMultiSelect } from '@/components/ui/searchable-multi-select'
import { Trash2, ImagePlus } from 'lucide-react'
import { CaptionInput } from './CaptionInput'
import type { CATEGORIES } from './constants'
import type { EntryRow } from './types'
import type { VetPatrolReportVM } from './useVetPatrolReport'

type Category = typeof CATEGORIES[number]

export function EntryCard({
    vm,
    cat,
    row,
    idx,
}: {
    vm: VetPatrolReportVM
    cat: Category
    row: EntryRow
    idx: number
}) {
    const { t } = useTranslation()
    // placeholderKeys 的值為 '' 代表該欄位沒有 hint
    const ph = (key: string) => (key ? t(key) : '')
    const rowEntryPhotos = row.id ? (vm.entryPhotosByEntry.get(row.id) ?? []) : []
    // 條目結構（刪條目）與照片（上傳/改說明/刪除）都不是追蹤者能碰的：
    // 後端照片三個 handler 為 require_permission!("animal.vet.recommend")，刪條目則由
    // update service 於 AWAITING_FOLLOW_UP 明確擋下（#378）。與同卡片文字欄位共用同一組
    // 階段旗標，差別是這裡隱藏而非 disable——按下去必定失敗的按鈕比不出現更誤導。
    const canEditStructure = !vm.isReadOnly && !vm.canEditFollowUpOnly
    return (
        <div data-temp-key={row.tempKey} className="border rounded-lg bg-card p-3 space-y-2">
            <div className="flex items-start justify-between gap-2">
                <span className="text-xs text-muted-foreground">{t('animalActions.vetPatrol.entryNumber', { index: idx + 1 })}</span>
                {canEditStructure && (
                    <button
                        type="button"
                        onClick={() => vm.removeRow(cat.key, idx)}
                        className="p-1 rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
                        title={t('animalActions.vetPatrol.deleteEntry')}
                    >
                        <Trash2 className="h-3.5 w-3.5" />
                    </button>
                )}
            </div>

            {cat.hasAnimal && (
                <div>
                    <label className="text-xs font-medium text-muted-foreground mb-1 block">
                        {t('animalActions.vetPatrol.animal')}
                        <span className="ml-2 text-[10px] text-muted-foreground/70">{t('animalActions.vetPatrol.animalMultiHint')}</span>
                    </label>
                    <SearchableMultiSelect
                        options={vm.animalOptions}
                        value={row.animal_ids}
                        onValueChange={(v) => vm.setAnimalIds(cat.key, idx, v)}
                        placeholder={t('animalActions.vetPatrol.animalPlaceholder')}
                        searchPlaceholder={t('animalActions.vetPatrol.earTagSearch')}
                        disabled={vm.isReadOnly || vm.canEditFollowUpOnly}
                    />
                </div>
            )}

            <div className="grid grid-cols-1 md:grid-cols-3 gap-2">
                <div>
                    <label className="text-xs font-medium text-muted-foreground mb-1 block">{t('animalActions.vetPatrol.observationContent')}</label>
                    <Textarea
                        value={row.observation}
                        onChange={(e) => vm.updateRow(cat.key, idx, 'observation', e.target.value)}
                        placeholder={ph(cat.placeholderKeys.observation)}
                        className="min-h-[64px] text-sm resize-none"
                        rows={3}
                        disabled={vm.isReadOnly || vm.canEditFollowUpOnly}
                    />
                </div>
                <div>
                    <label className="text-xs font-medium text-muted-foreground mb-1 block">{t('animalActions.common.recommendation')}</label>
                    <Textarea
                        value={row.suggestion}
                        onChange={(e) => vm.updateRow(cat.key, idx, 'suggestion', e.target.value)}
                        placeholder={ph(cat.placeholderKeys.suggestion)}
                        className="min-h-[64px] text-sm resize-none"
                        rows={3}
                        disabled={vm.isReadOnly || vm.canEditFollowUpOnly}
                    />
                </div>
                <div>
                    <label className="text-xs font-medium text-muted-foreground mb-1 block">
                        {t('animalActions.common.followUp')}
                        <span className="ml-2 text-[10px] text-muted-foreground/70">{t('animalActions.vetPatrol.filledByAccompanying')}</span>
                    </label>
                    <Textarea
                        value={row.follow_up}
                        onChange={(e) => vm.updateRow(cat.key, idx, 'follow_up', e.target.value)}
                        placeholder={vm.canEditFollowUpOnly ? ph(cat.placeholderKeys.follow_up) : t('animalActions.vetPatrol.followUpPendingHint')}
                        className="min-h-[64px] text-sm resize-none"
                        rows={3}
                        disabled={!vm.canEditFollowUpOnly}
                    />
                </div>
            </div>

            {/* Entry-level 照片區塊 */}
            <div className="border-t pt-2">
                <div className="flex items-center justify-between mb-2">
                    <label className="text-xs font-medium text-muted-foreground">{t('animalActions.vetPatrol.photoAttachments')}</label>
                    {canEditStructure && (row.id ? (
                        <label className="flex items-center gap-1 text-xs text-status-success-solid hover:text-status-success-text cursor-pointer">
                            <ImagePlus className="h-3.5 w-3.5" />
                            {t('animalActions.vetPatrol.addPhoto')}
                            <input
                                type="file"
                                accept="image/*"
                                capture="environment"
                                multiple
                                className="hidden"
                                onChange={(e) => row.id && vm.handleSelectEntryPhoto(row.id, e)}
                                disabled={vm.uploadEntryPhotoMutation.isPending}
                            />
                        </label>
                    ) : (
                        <span className="text-xs text-muted-foreground italic">{t('animalActions.vetPatrol.draftCreating')}</span>
                    ))}
                </div>
                {rowEntryPhotos.length > 0 && (
                    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
                        {rowEntryPhotos.map((photo) => (
                            <div key={photo.id} className="border rounded p-1 space-y-1">
                                <div className="relative">
                                    <img
                                        src={`/api/v1/vet-patrol-entry-photos/${photo.id}/download`}
                                        alt={photo.caption || photo.file_name}
                                        className="w-full h-20 object-cover rounded"
                                    />
                                    {canEditStructure && (
                                        <button
                                            type="button"
                                            onClick={() => vm.deleteEntryPhotoMutation.mutate(photo.id)}
                                            className="absolute top-1 right-1 bg-background/80 hover:bg-destructive hover:text-destructive-foreground rounded-full p-0.5"
                                            title={t('common.delete')}
                                        >
                                            <Trash2 className="h-3 w-3" />
                                        </button>
                                    )}
                                </div>
                                {/* placeholder 寫成問句＋範例：實測 prod 28 張條目照片 caption 全空，
                                    「說明（選填）」沒有告訴填寫者要寫什麼，導致 PDF 圖說只剩耳號。 */}
                                <CaptionInput
                                    value={photo.caption}
                                    placeholder={t('animalActions.vetPatrol.entryCaptionHint')}
                                    className="text-xs h-7"
                                    disabled={!canEditStructure}
                                    onSave={(caption) => vm.updateEntryCaptionMutation.mutate({
                                        photoId: photo.id,
                                        caption,
                                    })}
                                />
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    )
}
