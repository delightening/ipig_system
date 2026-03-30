import React, { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  ProtocolActivity,
  ProtocolStatus,
  getProtocolActivities,
} from '@/lib/api'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Clock, Loader2, ChevronLeft, ChevronRight } from 'lucide-react'
import { formatDateTime } from '@/lib/utils'

const statusColors: Record<
  ProtocolStatus,
  'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline'
> = {
  DRAFT: 'secondary',
  SUBMITTED: 'default',
  PRE_REVIEW: 'default',
  PRE_REVIEW_REVISION_REQUIRED: 'destructive',
  VET_REVIEW: 'warning',
  VET_REVISION_REQUIRED: 'destructive',
  UNDER_REVIEW: 'warning',
  REVISION_REQUIRED: 'destructive',
  RESUBMITTED: 'default',
  APPROVED: 'success',
  APPROVED_WITH_CONDITIONS: 'success',
  DEFERRED: 'secondary',
  REJECTED: 'destructive',
  SUSPENDED: 'destructive',
  CLOSED: 'outline',
  DELETED: 'destructive',
}

interface HistoryTabProps {
  protocolId: string
}

export const HistoryTab = React.memo(function HistoryTab({ protocolId }: HistoryTabProps) {
  const { t } = useTranslation()
  const [activityPage, setActivityPage] = useState(1)
  const ACTIVITIES_PER_PAGE = 15

  const { data: activities, isLoading: activitiesLoading } = useQuery({
    queryKey: ['protocol-activities', protocolId],
    queryFn: () => getProtocolActivities(protocolId),
    enabled: !!protocolId,
  })

  const totalActivities = activities?.length || 0
  const totalActivityPages = Math.ceil(totalActivities / ACTIVITIES_PER_PAGE)
  const paginatedActivities =
    activities?.slice(
      (activityPage - 1) * ACTIVITIES_PER_PAGE,
      activityPage * ACTIVITIES_PER_PAGE
    ) || []

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('protocols.detail.sections.historyTitle')}</CardTitle>
        <CardDescription>
          {t('protocols.detail.sections.historyDesc')}
        </CardDescription>
      </CardHeader>
      <CardContent>
        {activitiesLoading ? (
          <div className="flex justify-center py-8">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : paginatedActivities.length > 0 ? (
          <>
            <div className="relative">
              <div className="absolute left-4 top-0 bottom-0 w-0.5 bg-muted" />
              <ul className="space-y-4">
                {paginatedActivities.map((activity: ProtocolActivity) => (
                  <li key={activity.id} className="relative pl-10">
                    <div className="absolute left-2 w-4 h-4 rounded-full bg-primary border-2 border-white mt-1.5" />
                    <div className="bg-muted p-4 rounded-lg border border-border shadow-sm">
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                          <Badge
                            variant="outline"
                            className="text-[10px] uppercase font-bold py-0 h-5"
                          >
                            {activity.activity_type_display || activity.activity_type}
                          </Badge>
                          <span className="font-semibold text-foreground">
                            {activity.actor_name}
                          </span>
                        </div>
                        <span className="text-xs text-muted-foreground flex items-center gap-1">
                          <Clock className="h-3 w-3" />
                          {formatDateTime(activity.created_at)}
                        </span>
                      </div>
                      {activity.from_value && activity.to_value && (
                        <div className="flex items-center gap-2 text-sm">
                          <Badge
                            variant={
                              statusColors[activity.from_value as ProtocolStatus] ||
                              'outline'
                            }
                          >
                            {t(`protocols.status.${activity.from_value}`)}
                          </Badge>
                          <span className="text-muted-foreground">→</span>
                          <Badge
                            variant={
                              statusColors[activity.to_value as ProtocolStatus] ||
                              'outline'
                            }
                          >
                            {t(`protocols.status.${activity.to_value}`)}
                          </Badge>
                        </div>
                      )}
                      {activity.target_entity_name && (
                        <p className="text-sm text-muted-foreground mt-1">
                          {activity.target_entity_name}
                        </p>
                      )}
                      {activity.remark && (
                        <div className="bg-white p-2 rounded border border-border text-sm text-muted-foreground mt-2">
                          <span className="font-medium text-xs text-muted-foreground block mb-1">
                            {t('protocols.detail.dialogs.status.remark')}:
                          </span>
                          {activity.remark}
                        </div>
                      )}
                    </div>
                  </li>
                ))}
              </ul>
            </div>

            {totalActivityPages > 1 && (
              <div className="flex items-center justify-between mt-4 pt-4 border-t">
                <p className="text-sm text-muted-foreground">
                  共 {totalActivities} 筆，第 {activityPage} / {totalActivityPages}{' '}
                  頁
                </p>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={activityPage <= 1}
                    onClick={() => setActivityPage((p) => Math.max(1, p - 1))}
                  >
                    <ChevronLeft className="h-4 w-4 mr-1" />
                    上一頁
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={activityPage >= totalActivityPages}
                    onClick={() =>
                      setActivityPage((p) => Math.min(totalActivityPages, p + 1))
                    }
                  >
                    下一頁
                    <ChevronRight className="h-4 w-4 ml-1" />
                  </Button>
                </div>
              </div>
            )}
          </>
        ) : (
          <div className="text-center py-8 text-muted-foreground">
            <Clock className="h-12 w-12 mx-auto mb-2" />
            <p>{t('protocols.detail.tables.noHistory')}</p>
          </div>
        )}
      </CardContent>
    </Card>
  )
})
