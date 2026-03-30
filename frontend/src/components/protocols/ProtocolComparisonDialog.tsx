import { useState, useEffect } from 'react'
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
} from '@/components/ui/dialog'
import { ProtocolWorkingContent } from '@/types/protocol'
import { Check, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

type DiffEntry = { path: string; oldValue: unknown; newValue: unknown }

function findDifferences(obj1: unknown, obj2: unknown, path = ''): DiffEntry[] {
    const differences: DiffEntry[] = []

    if (obj1 === obj2) return differences

    // If one is null/undefined/different type, treat as difference
    if (
        obj1 === null || obj1 === undefined ||
        obj2 === null || obj2 === undefined ||
        typeof obj1 !== typeof obj2 ||
        (Array.isArray(obj1) !== Array.isArray(obj2))
    ) {
        differences.push({ path, oldValue: obj1, newValue: obj2 })
        return differences
    }

    if (typeof obj1 === 'object' && obj1 !== null && typeof obj2 === 'object' && obj2 !== null) {
        if (Array.isArray(obj1) && Array.isArray(obj2)) {
            if (obj1.length !== obj2.length) {
                differences.push({ path, oldValue: obj1, newValue: obj2 })
            } else {
                // Same length arrays, try to compare items if they are objects
                const allPrimitives = obj1.every(x => typeof x !== 'object')
                if (allPrimitives) {
                    if (JSON.stringify(obj1) !== JSON.stringify(obj2)) {
                        differences.push({ path, oldValue: obj1, newValue: obj2 })
                    }
                } else {
                    obj1.forEach((item, idx) => {
                        differences.push(...findDifferences(item, obj2[idx], `${path}.${idx}`))
                    })
                }
            }
            return differences
        }

        const o1 = obj1 as Record<string, unknown>
        const o2 = obj2 as Record<string, unknown>
        const allKeys = new Set([...Object.keys(o1), ...Object.keys(o2)])
        allKeys.forEach((key) => {
            const currentPath = path ? `${path}.${key}` : key
            differences.push(...findDifferences(o1[key], o2[key], currentPath))
        })
        return differences
    }

    if (obj1 !== obj2) {
        differences.push({ path, oldValue: obj1, newValue: obj2 })
    }

    return differences
}

interface Props {
    open: boolean
    onOpenChange: (open: boolean) => void
    versionA: { version_no: number; content: ProtocolWorkingContent }
    versionB: { version_no: number; content: ProtocolWorkingContent }
    protocolTitle: string
}

export function ProtocolComparisonDialog({
    open,
    onOpenChange,
    versionA,
    versionB,
    protocolTitle: _protocolTitle,
}: Props) {
    const { t } = useTranslation()
    const [diffs, setDiffs] = useState<DiffEntry[]>([])

    useEffect(() => {
        if (versionA && versionB) {
            setDiffs(findDifferences(versionA.content, versionB.content))
        }
    }, [versionA, versionB])

    const formatValue = (val: unknown) => {
        if (val === null || val === undefined) return <span className="text-muted-foreground italic">{t('protocols.detail.dialogs.version.noContent')}</span>
        if (typeof val === 'boolean') return val ? <Check className="h-4 w-4 text-status-success-solid" /> : <X className="h-4 w-4 text-status-error-solid" />

        if (Array.isArray(val)) {
            if (val.length === 0) return <span className="text-muted-foreground italic">{t('common.noData')}</span>
            // If it's an array of objects
            if (typeof val[0] === 'object' && val[0] !== null) {
                // If it's FileInfo
                const fileItems = val.filter((item: unknown) => typeof item === 'object' && item !== null && 'file_name' in item)
                if (fileItems.length > 0) {
                    return fileItems.map((f: unknown) => (f as { file_name: string }).file_name).join(', ')
                }

                return (
                    <div className="space-y-1">
                        {val.map((item: unknown, idx: number) => {
                            const o = item as Record<string, unknown>
                            return (
                            <div key={idx} className="text-xs border-l-2 border-border pl-2 py-0.5">
                                {String(o.name ?? o.species ?? o.drug_name ?? o.agent_name ?? `項目 ${idx + 1}`)}
                            </div>
                            )
                        })}
                        <div className="text-[10px] text-muted-foreground font-normal">共 {val.length} 筆項目</div>
                    </div>
                )
            }
            // If it's an array of strings/numbers
            return val.join(', ')
        }

        if (typeof val === 'object' && val !== null) {
            // Handle FileInfo objects
            if ('file_name' in val && typeof (val as { file_name: unknown }).file_name === 'string') return (val as { file_name: string }).file_name
            // Handle date objects or simple key-value shells
            return <pre className="text-[10px] font-mono leading-tight">{JSON.stringify(val, null, 1)}</pre>
        }

        return String(val)
    }

    const getLabel = (path: string) => {
        // Broad mapping for human-readable labels based on ProtocolWorkingContent
        // Mapping paths to i18n keys or direct translations
        const mapping: Record<string, string> = {
            'basic.study_title': t('protocols.content.sections.projectName'),
            'basic.apply_study_number': '申請編號',
            'basic.is_glp': t('protocols.content.sections.glpAttribute'),
            'basic.project_type': t('protocols.content.sections.projectType'),
            'basic.project_category': t('protocols.content.sections.projectCategory'),
            'basic.start_date': t('aup.basic.startDate'),
            'basic.end_date': t('aup.basic.endDate'),
            'basic.pi.name': t('protocols.content.sections.piName'),
            'basic.pi.phone': t('protocols.content.sections.piPhone'),
            'basic.pi.email': t('protocols.content.sections.piEmail'),
            'basic.pi.address': t('protocols.content.sections.piAddress'),
            'basic.sponsor.name': t('protocols.content.sections.sponsorName'),
            'basic.sponsor.contact_person': t('protocols.content.sections.contactPerson'),
            'basic.housing_location': t('protocols.content.sections.location'),
            'purpose.significance': t('protocols.content.sections.significance'),
            'purpose.replacement.rationale': t('protocols.content.sections.replacementRationale'),
            'purpose.reduction.design': t('protocols.content.sections.reductionDesign'),
            'purpose.duplicate.experiment': t('protocols.content.sections.duplicate'),
            'items.use_test_item': '投予試驗物質',
            'items.test_items': t('protocols.content.sections.testItems'),
            'items.control_items': t('protocols.content.sections.controlItems'),
            'design.pain.category': t('protocols.content.sections.painCategory'),
            'design.endpoints.experimental_endpoint': t('protocols.content.sections.experimentalEndpoint'),
            'design.endpoints.humane_endpoint': t('protocols.content.sections.humaneEndpoint'),
            'design.procedures': t('protocols.content.sections.procedures'),
            'design.anesthesia.is_under_anesthesia': t('protocols.content.sections.anesthesia'),
            'design.anesthesia.anesthesia_type': t('protocols.content.sections.anesthesiaType'),
            'design.final_handling.method': '最終處理方式',
            'design.carcass_disposal.method': '屍體處理方法',
            'design.hazards.used': '使用危害性物質',
            'design.controlled_substances.used': '使用管制藥品',
            'surgery.surgery_type': t('protocols.content.sections.surgeryType'),
            'surgery.preop_preparation': t('protocols.content.sections.preop_Preparation'),
            'surgery.surgery_description': t('protocols.content.sections.surgeryDescription'),
            'surgery.monitoring': t('protocols.content.sections.monitoring'),
            'surgery.postop_care': t('protocols.content.sections.postopCare'),
            'surgery.drugs': t('protocols.content.sections.drugPlan'),
            'animals.animals': t('protocols.content.sections.animals'),
            'animals.total_animals': t('protocols.content.sections.totalAnimals'),
            'personnel': t('protocols.content.sections.personnel'),
            'attachments': t('protocols.content.sections.attachments'),
            'signature': t('protocols.content.sections.signatures'),
        }

        // Handle array indices in path (e.g., personnel.0.name -> personnel (項目 1) > 姓名)
        const parts = path.split('.')
        let currentLabel = ''
        let currentPath = ''

        for (let i = 0; i < parts.length; i++) {
            const part = parts[i]
            const isIndex = !isNaN(Number(part))

            if (isIndex) {
                currentLabel += ` (${t('common.noDocs')?.includes('無') ? '項目' : 'Item'} ${Number(part) + 1})`
            } else {
                currentPath = currentPath ? `${currentPath}.${part}` : part
                const label = mapping[currentPath] || part
                currentLabel = currentLabel ? `${currentLabel} > ${label}` : label
            }
        }

        return currentLabel
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-5xl max-h-[90vh] flex flex-col">
                <DialogHeader>
                    <DialogTitle>{t('protocols.detail.sections.versionsTitle')}</DialogTitle>
                    <DialogDescription>
                        {t('protocols.detail.dialogs.version.title', { version: versionA?.version_no })}
                        {' vs '}
                        {t('protocols.detail.dialogs.version.title', { version: versionB?.version_no })}
                    </DialogDescription>
                </DialogHeader>

                <div className="flex-1 mt-4 border rounded-md p-4 overflow-y-auto max-h-[70vh]">
                    {diffs.length === 0 ? (
                        <div className="text-center py-12 text-muted-foreground">
                            <Check className="h-12 w-12 mx-auto mb-4 text-status-success-solid" />
                            <p>{t('common.noData')}</p>
                        </div>
                    ) : (
                        <div className="space-y-6">
                            {diffs.map((diff) => (
                                <div key={diff.path} className="grid grid-cols-12 gap-4 pb-4 border-b last:border-0">
                                    <div className="col-span-12 md:col-span-3 lg:col-span-2">
                                        <span className="text-sm font-semibold text-foreground">{getLabel(diff.path)}</span>
                                    </div>
                                    <div className="col-span-12 md:col-span-9 lg:col-span-5 p-3 rounded bg-status-error-bg border border-red-100">
                                        <div className="text-xs text-status-error-text mb-1 font-medium">
                                            {t('protocols.detail.dialogs.version.title', { version: versionA?.version_no })}
                                        </div>
                                        <div className="text-sm overflow-hidden text-ellipsis">{formatValue(diff.oldValue)}</div>
                                    </div>
                                    <div className="col-span-12 md:col-span-9 lg:col-span-5 p-3 rounded bg-status-success-bg border border-green-100">
                                        <div className="text-xs text-status-success-text mb-1 font-medium">
                                            {t('protocols.detail.dialogs.version.title', { version: versionB?.version_no })}
                                        </div>
                                        <div className="text-sm overflow-hidden text-ellipsis">{formatValue(diff.newValue)}</div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </DialogContent>
        </Dialog>
    )
}
