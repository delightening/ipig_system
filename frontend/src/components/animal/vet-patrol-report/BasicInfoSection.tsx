// 報告基本資訊（巡場日期 + 陪同人員）（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import { useTranslation } from 'react-i18next'

import { DatePicker } from '@/components/ui/date-picker'
import { SearchableSelect } from '@/components/ui/searchable-select'
import type { VetPatrolReportVM } from './useVetPatrolReport'

export function BasicInfoSection({ vm }: { vm: VetPatrolReportVM }) {
    const { t } = useTranslation()
    return (
        <div className="grid grid-cols-2 gap-3 mb-4">
            <div>
                <label className="text-sm font-medium mb-1 block">{t('animalActions.vetPatrol.patrolDate')}</label>
                <DatePicker value={vm.patrolDate} onChange={(v) => { vm.markInteracted(); vm.setHasUnsavedTextChanges(true); vm.setPatrolDate(v) }} required disabled={vm.isReadOnly || vm.canEditFollowUpOnly} />
            </div>
            <div>
                <label className="text-sm font-medium mb-1 block">{t('animalActions.vetPatrol.accompanying')}</label>
                <SearchableSelect
                    options={vm.staffOptions}
                    value={vm.accompanyingPersonnel}
                    onValueChange={(v) => {
                        vm.markInteracted()
                        vm.setHasUnsavedTextChanges(true)
                        vm.setAccompanyingPersonnel(v)
                        // 同步指派為「追蹤者」：依 display_name 查 staff.id
                        const staff = (vm.staffList ?? []).find(s => s.display_name === v)
                        vm.setFollowUpUserId(staff?.id ?? '')
                    }}
                    placeholder={t('animalActions.vetPatrol.accompanyingPlaceholder')}
                    searchPlaceholder={t('animalActions.vetPatrol.staffSearch')}
                    emptyMessage={t('animalActions.vetPatrol.staffEmpty')}
                    disabled={vm.isReadOnly}
                />
            </div>
        </div>
    )
}
