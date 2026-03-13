import { ChevronLeft, ChevronRight } from 'lucide-react'

import { Button } from '@/components/ui/button'

interface AuditPaginationProps {
    total: number
    totalPages: number
    currentPage: number
    onPageChange: (page: number) => void
}

export function AuditPagination({ total, totalPages, currentPage, onPageChange }: AuditPaginationProps) {
    if (totalPages <= 0) return null

    return (
        <div className="flex flex-col md:flex-row items-center justify-between px-4 py-3 border-t gap-2">
            <p className="text-sm text-muted-foreground">
                共 {total} 筆，第 {currentPage} / {totalPages} 頁
            </p>
            <div className="flex items-center gap-2">
                <Button
                    variant="outline"
                    size="sm"
                    disabled={currentPage <= 1}
                    onClick={() => onPageChange(Math.max(1, currentPage - 1))}
                >
                    <ChevronLeft className="h-4 w-4 mr-1" />上一頁
                </Button>
                <Button
                    variant="outline"
                    size="sm"
                    disabled={currentPage >= totalPages}
                    onClick={() => onPageChange(Math.min(totalPages, currentPage + 1))}
                >
                    下一頁<ChevronRight className="h-4 w-4 ml-1" />
                </Button>
            </div>
        </div>
    )
}
