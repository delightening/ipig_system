/**
 * 設備履歷頁面 — 設備基本資訊 + 統一 Timeline
 */
import { useParams, Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { ArrowLeft, Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import api from '@/lib/api'
import type { PaginatedResponse } from '@/types/common'

import type { Equipment, EquipmentTimelineEntry } from './types'
import { EquipmentInfoCard } from './components/EquipmentInfoCard'
import { EquipmentTimeline } from './components/EquipmentTimeline'

export function EquipmentHistoryPage() {
  const { id } = useParams<{ id: string }>()

  const { data: equipment, isLoading: equipLoading } = useQuery({
    queryKey: ['equipment', id],
    queryFn: async () => {
      const res = await api.get<Equipment>(`/equipment/${id}`)
      return res.data
    },
    enabled: !!id,
  })

  const { data: timeline, isLoading: timelineLoading } = useQuery({
    queryKey: ['equipment-timeline', id],
    queryFn: async () => {
      const res = await api.get<PaginatedResponse<EquipmentTimelineEntry>>(
        `/equipment/${id}/timeline`,
        { params: { per_page: 200 } },
      )
      return res.data
    },
    enabled: !!id,
  })

  const isLoading = equipLoading || timelineLoading

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-24">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!equipment) {
    return (
      <div className="text-center py-24 text-muted-foreground">
        <p>找不到該設備</p>
        <Link to="/equipment" className="text-primary hover:underline mt-2 inline-block">
          返回設備列表
        </Link>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" asChild>
          <Link to="/equipment">
            <ArrowLeft className="h-5 w-5" />
          </Link>
        </Button>
        <h1 className="text-2xl font-bold">設備履歷</h1>
      </div>

      <EquipmentInfoCard equipment={equipment} />

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">完整履歷</CardTitle>
        </CardHeader>
        <CardContent>
          <EquipmentTimeline entries={timeline?.data ?? []} />
        </CardContent>
      </Card>
    </div>
  )
}
