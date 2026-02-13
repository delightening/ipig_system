/**
 * 骨架屏元件（Skeleton）
 *
 * 提供表格、卡片、表單三種型態的骨架屏，
 * 用於資料載入中的佔位顯示。
 *
 * 使用方式：
 *   <Skeleton variant="table" rows={5} />
 *   <Skeleton variant="card" count={3} />
 *   <Skeleton variant="form" fields={4} />
 */

import { cn } from '@/lib/utils'

interface SkeletonBaseProps {
    className?: string
}

/** 基礎脈衝動畫區塊 */
function SkeletonPulse({ className }: SkeletonBaseProps) {
    return (
        <div className={cn('animate-pulse rounded-md bg-muted', className)} />
    )
}

// ============================================
// 表格骨架屏
// ============================================

interface TableSkeletonProps extends SkeletonBaseProps {
    variant: 'table'
    /** 列數，預設 5 */
    rows?: number
    /** 欄數，預設 4 */
    columns?: number
}

function TableSkeleton({ rows = 5, columns = 4, className }: Omit<TableSkeletonProps, 'variant'>) {
    return (
        <div className={cn('space-y-3', className)}>
            {/* 表頭 */}
            <div className="flex gap-4 px-4">
                {Array.from({ length: columns }).map((_, i) => (
                    <SkeletonPulse key={`h-${i}`} className="h-4 flex-1" />
                ))}
            </div>
            {/* 分隔線 */}
            <div className="border-b" />
            {/* 資料列 */}
            {Array.from({ length: rows }).map((_, ri) => (
                <div key={`r-${ri}`} className="flex gap-4 px-4 py-2">
                    {Array.from({ length: columns }).map((_, ci) => (
                        <SkeletonPulse key={`c-${ri}-${ci}`} className="h-4 flex-1" />
                    ))}
                </div>
            ))}
        </div>
    )
}

// ============================================
// 卡片骨架屏
// ============================================

interface CardSkeletonProps extends SkeletonBaseProps {
    variant: 'card'
    /** 卡片數量，預設 3 */
    count?: number
}

function CardSkeleton({ count = 3, className }: Omit<CardSkeletonProps, 'variant'>) {
    return (
        <div className={cn('grid gap-4 sm:grid-cols-2 lg:grid-cols-3', className)}>
            {Array.from({ length: count }).map((_, i) => (
                <div key={i} className="rounded-lg border p-4 space-y-3">
                    <SkeletonPulse className="h-5 w-2/3" />
                    <SkeletonPulse className="h-4 w-full" />
                    <SkeletonPulse className="h-4 w-4/5" />
                    <div className="flex gap-2 pt-2">
                        <SkeletonPulse className="h-8 w-16" />
                        <SkeletonPulse className="h-8 w-16" />
                    </div>
                </div>
            ))}
        </div>
    )
}

// ============================================
// 表單骨架屏
// ============================================

interface FormSkeletonProps extends SkeletonBaseProps {
    variant: 'form'
    /** 欄位數量，預設 4 */
    fields?: number
}

function FormSkeleton({ fields = 4, className }: Omit<FormSkeletonProps, 'variant'>) {
    return (
        <div className={cn('space-y-6', className)}>
            {Array.from({ length: fields }).map((_, i) => (
                <div key={i} className="space-y-2">
                    <SkeletonPulse className="h-4 w-24" />
                    <SkeletonPulse className="h-10 w-full" />
                </div>
            ))}
            <div className="flex gap-2 pt-2">
                <SkeletonPulse className="h-10 w-24" />
                <SkeletonPulse className="h-10 w-24" />
            </div>
        </div>
    )
}

// ============================================
// 整合匯出
// ============================================

type SkeletonProps = TableSkeletonProps | CardSkeletonProps | FormSkeletonProps

/**
 * 統一骨架屏元件
 *
 * @example
 * // 表格型態
 * <Skeleton variant="table" rows={5} columns={4} />
 *
 * // 卡片型態
 * <Skeleton variant="card" count={3} />
 *
 * // 表單型態
 * <Skeleton variant="form" fields={6} />
 */
export function Skeleton(props: SkeletonProps) {
    switch (props.variant) {
        case 'table':
            return <TableSkeleton {...props} />
        case 'card':
            return <CardSkeleton {...props} />
        case 'form':
            return <FormSkeleton {...props} />
    }
}

/** 基礎行內骨架（適合在文字行間使用） */
export function InlineSkeleton({ className }: SkeletonBaseProps) {
    return <SkeletonPulse className={cn('inline-block h-4 w-20', className)} />
}

export { SkeletonPulse }
