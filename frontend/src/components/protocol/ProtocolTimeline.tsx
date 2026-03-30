import { cn } from '@/lib/utils'
import { Badge } from '@/components/ui/badge'
import type { ProtocolStatus } from '@/types/aup'

interface ProtocolTimelineProps {
    status: ProtocolStatus
}

interface TimelineNode {
    label: string
    statuses: ProtocolStatus[]
}

const TIMELINE_NODES: TimelineNode[] = [
    { label: '草稿', statuses: ['DRAFT'] },
    { label: '已提交', statuses: ['SUBMITTED'] },
    { label: '初審', statuses: ['PRE_REVIEW', 'PRE_REVIEW_REVISION_REQUIRED'] },
    { label: '獸醫審查', statuses: ['VET_REVIEW', 'VET_REVISION_REQUIRED'] },
    { label: '委員會', statuses: ['UNDER_REVIEW', 'REVISION_REQUIRED', 'RESUBMITTED'] },
    { label: '核准', statuses: ['APPROVED', 'APPROVED_WITH_CONDITIONS'] },
]

const SPECIAL_STATUSES: ProtocolStatus[] = ['REJECTED', 'SUSPENDED', 'CLOSED', 'DELETED', 'DEFERRED']
const REVISION_STATUSES: ProtocolStatus[] = [
    'PRE_REVIEW_REVISION_REQUIRED', 'VET_REVISION_REQUIRED', 'REVISION_REQUIRED',
]

function getNodeIndex(status: ProtocolStatus): number {
    return TIMELINE_NODES.findIndex(node => node.statuses.includes(status))
}

const specialStatusLabels: Partial<Record<ProtocolStatus, string>> = {
    REJECTED: '已否決',
    SUSPENDED: '已暫停',
    CLOSED: '已結案',
    DELETED: '已刪除',
    DEFERRED: '延後審議',
}

export function ProtocolTimeline({ status }: ProtocolTimelineProps) {
    const isSpecial = SPECIAL_STATUSES.includes(status)
    const isRevision = REVISION_STATUSES.includes(status)
    const currentIndex = getNodeIndex(status)

    if (isSpecial) {
        return (
            <div className="flex items-center gap-3">
                <span className="text-sm text-muted-foreground">計畫狀態：</span>
                <Badge variant="destructive">{specialStatusLabels[status] || status}</Badge>
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
                        <div key={node.label} className="flex items-center flex-1 last:flex-none">
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
                                    {node.label}
                                </span>
                                {isRevisionNode && (
                                    <span className="text-[10px] text-status-warning-text">退回修改</span>
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
