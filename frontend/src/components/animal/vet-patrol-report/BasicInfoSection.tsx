// 報告基本資訊（巡場日期 + 陪同人員）（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import { DatePicker } from '@/components/ui/date-picker'
import { SearchableSelect } from '@/components/ui/searchable-select'
import type { VetPatrolReportVM } from './useVetPatrolReport'

export function BasicInfoSection({ vm }: { vm: VetPatrolReportVM }) {
    return (
        <div className="grid grid-cols-2 gap-3 mb-4">
            <div>
                <label className="text-sm font-medium mb-1 block">巡場日期</label>
                <DatePicker value={vm.patrolDate} onChange={(v) => { vm.markInteracted(); vm.setHasUnsavedTextChanges(true); vm.setPatrolDate(v) }} required disabled={vm.isReadOnly || vm.canEditFollowUpOnly} />
            </div>
            <div>
                <label className="text-sm font-medium mb-1 block">陪同人員</label>
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
                    placeholder="選擇陪同人員（也是追蹤者）"
                    searchPlaceholder="搜尋姓名..."
                    emptyMessage="無符合人員（可在 HR 模組新增 staff）"
                    disabled={vm.isReadOnly}
                />
            </div>
        </div>
    )
}
