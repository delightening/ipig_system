import { useTranslation } from 'react-i18next'

import { cn } from '@/lib/utils'
import { Badge } from '@/components/ui/badge'
import type { ProtocolStatus } from '@/types/aup'

interface ProtocolTimelineProps {
    status: ProtocolStatus
}

interface TimelineNode {
    // i18n 鍵；渲染時才 t(labelKey)（模組頂層不可存翻譯後字串）
    labelKey: string
    statuses: ProtocolStatus[]
}

const TIMELINE_NODES: TimelineNode[] = [
    { labelKey: 'protocols.status.DRAFT', statuses: ['DRAFT'] },
    { labelKey: 'protocolComponents.timeline.submitted', statuses: ['SUBMITTED'] },
    { labelKey: 'protocolComponents.timeline.preReview', statuses: ['PRE_REVIEW', 'PRE_REVIEW_REVISION_REQUIRED'] },
    { labelKey: 'protocols.status.VET_REVIEW', statuses: ['VET_REVIEW', 'VET_REVISION_REQUIRED'] },
    { labelKey: 'protocolComponents.timeline.committee', statuses: ['UNDER_REVIEW', 'REVISION_REQUIRED', 'RESUBMITTED'] },
    { labelKey: 'protocolComponents.timeline.approved', statuses: ['APPROVED', 'APPROVED_WITH_CONDITIONS'] },
]

const SPECIAL_STATUSES: ProtocolStatus[] = ['REJECTED', 'SUSPENDED', 'CLOSED', 'DELETED', 'DEFERRED']
const REVISION_STATUSES: ProtocolStatus[] = [
    'PRE_REVIEW_REVISION_REQUIRED', 'VET_REVISION_REQUIRED', 'REVISION_REQUIRED',
]

function getNodeIndex(status: ProtocolStatus): number {
    return TIMELINE_NODES.findIndex(node => node.statuses.includes(status))
}

// 值為 i18n 鍵；渲染時才 t(key)
const specialStatusLabelKeys: Partial<Record<ProtocolStatus, string>> = {
    REJECTED: 'protocolComponents.timeline.rejected',
    SUSPENDED: 'protocols.status.SUSPENDED',
    CLOSED: 'protocols.status.CLOSED',
    DELETED: 'protocols.status.DELETED',
    DEFERRED: 'protocolComponents.timeline.deferred',
}

export function ProtocolTimeline({ status }: ProtocolTimelineProps) {
    const { t } = useTranslation()
    const isSpecial = SPECIAL_STATUSES.includes(status)
    const isRevision = REVISION_STATUSES.includes(status)
    const currentIndex = getNodeIndex(status)

    if (isSpecial) {
        return (
            <div className="flex items-center gap-3">
                <span className="text-sm text-muted-foreground">{t('protocolComponents.timeline.statusLabel')}</span>
                <Badge variant="destructive">{specialStatusLabelKeys[status] ? t(specialStatusLabelKeys[status]) : status}</Badge>
            </div>
        )
    }

    return (
        <div className="w-full overflow-x-auto">
            <div className="flex items-center min-w-[500px] px-2 py-4">
                {TIMELINE_NODES.map((node, i) => {
                    const isCompleted = i < currentIndex
                    const isCurrent = i === currentIndex
                    const isRevisionNode = isCurrent && isRevision

                    return (
                        <div key={node.labelKey} className="flex items-center flex-1 last:flex-none">
                            {/* Node */}
                            <div className="flex flex-col items-center gap-1.5">
                                <div
                                    className={cn(
                                        'h-4 w-4 rounded-full border-2 transition-all',
                                        isCompleted && 'bg-status-success-text border-status-success-text',
                                        isCurrent && !isRevisionNode && 'bg-primary border-primary animate-pulse',
                                        isRevisionNode && 'bg-status-warning-text border-status-warning-text',
                                        !isCompleted && !isCurrent && 'bg-muted border-muted-foreground/30',
                                    )}
                                />
                                <span
                                    className={cn(
                                        'text-xs whitespace-nowrap',
                                        isCompleted && 'text-status-success-text font-medium',
                                        isCurrent && !isRevisionNode && 'text-primary font-semibold',
                                        isRevisionNode && 'text-status-warning-text font-semibold',
                                        !isCompleted && !isCurrent && 'text-muted-foreground',
                                    )}
                                >
                                    {t(node.labelKey)}
                                </span>
                                {isRevisionNode && (
                                    <span className="text-[10px] text-status-warning-text">{t('protocolComponents.timeline.returnedForRevision')}</span>
                                )}
                            </div>

                            {/* Connector line */}
                            {i < TIMELINE_NODES.length - 1 && (
                                <div
                                    className={cn(
                                        'flex-1 h-0.5 mx-1',
                                        i < currentIndex ? 'bg-status-success-text' : 'bg-muted-foreground/20',
                                    )}
                                />
                            )}
                        </div>
                    )
                })}
            </div>
        </div>
    )
}
