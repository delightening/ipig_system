import { ChevronLeft, ChevronRight } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'

interface AuditPaginationProps {
    total: number
    totalPages: number
    currentPage: number
    onPageChange: (page: number) => void
}

export function AuditPagination({ total, totalPages, currentPage, onPageChange }: AuditPaginationProps) {
    const { t } = useTranslation()

    if (totalPages <= 0) return null

    return (
        <div className="flex flex-col md:flex-row items-center justify-between px-4 py-3 border-t gap-2">
            <p className="text-sm text-muted-foreground">
                {t('admin.auditPagination.summary', { total, currentPage, totalPages })}
            </p>
            <div className="flex items-center gap-2">
                <Button
                    variant="outline"
                    size="sm"
                    disabled={currentPage <= 1}
                    onClick={() => onPageChange(Math.max(1, currentPage - 1))}
                >
                    <ChevronLeft className="h-4 w-4 mr-1" />{t('admin.auditPagination.previous')}
                </Button>
                <Button
                    variant="outline"
                    size="sm"
                    disabled={currentPage >= totalPages}
                    onClick={() => onPageChange(Math.min(totalPages, currentPage + 1))}
                >
                    {t('admin.auditPagination.next')}<ChevronRight className="h-4 w-4 ml-1" />
                </Button>
            </div>
        </div>
    )
}
