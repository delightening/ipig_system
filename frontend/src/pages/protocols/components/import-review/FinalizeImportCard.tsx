import { useEffect, useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Loader2 } from 'lucide-react'

import { finalizeImport } from '@/lib/api/protocol'
import { listTemplateVersions } from '@/lib/api/protocolTemplateVersions'
import { getApiErrorMessage } from '@/lib/apiError'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { formatDate } from '@/lib/utils'

/**
 * 補登：完成補登（建 v1 快照 + 記原始版本號 + 鎖定），含二次確認。
 * 原計劃書版本號為必填，並依計畫核准通過日（approvalDate）自動帶入院區「計畫書範本版本」
 * 登記中當時生效的版本（生效日 ≤ 核准日 的最新一筆），下拉預選供填寫人確認 / 修改。
 */
export function FinalizeImportCard({
  protocolId,
  approvalDate,
}: {
  protocolId: string
  approvalDate?: string
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [versionLabel, setVersionLabel] = useState('')
  const [confirming, setConfirming] = useState(false)

  const { data: versions = [], isLoading: versionsLoading } = useQuery({
    queryKey: ['protocol-template-versions'],
    queryFn: listTemplateVersions,
  })

  // 依生效日新→舊排序
  const sortedVersions = useMemo(
    () => [...versions].sort((a, b) => (b.effective_date ?? '').localeCompare(a.effective_date ?? '')),
    [versions]
  )

  // 自動帶入：取「生效日 ≤ 計畫核准通過日」中最新的一筆（即核准當時生效的範本版本）
  const autoMatched = useMemo(() => {
    if (!approvalDate) return ''
    const match = sortedVersions.find((v) => v.effective_date && v.effective_date <= approvalDate)
    return match?.version_label ?? ''
  }, [sortedVersions, approvalDate])

  // 預選自動帶入版本（未手動選過才帶入；functional setState 避免把 versionLabel 放進 deps）
  useEffect(() => {
    if (autoMatched) setVersionLabel((cur) => cur || autoMatched)
  }, [autoMatched])

  const finalizeMutation = useMutation({
    mutationFn: () => finalizeImport(protocolId, versionLabel.trim()),
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocolPages.importReview.finalize.success') })
      queryClient.invalidateQueries({ queryKey: ['protocol', protocolId] })
      navigate(`/protocols/${protocolId}`)
    },
    onError: (err: unknown) =>
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('protocolPages.importReview.finalize.failed')), variant: 'destructive' }),
  })

  const handleConfirm = () => {
    if (!versionLabel.trim()) {
      toast({ title: t('common.error'), description: t('protocolPages.importReview.finalize.versionRequired'), variant: 'destructive' })
      return
    }
    setConfirming(true)
  }

  return (
    <Card>
      <CardHeader><CardTitle className="text-base">{t('protocolPages.importReview.finalize.title')}</CardTitle></CardHeader>
      <CardContent className="space-y-3">
        <div className="grid gap-1 max-w-sm">
          <Label>{t('protocolPages.importReview.finalize.versionLabel')}</Label>
          {versionsLoading ? (
            // 載入中：先顯示停用下拉，避免閃現手動輸入框再切換成下拉
            <Select disabled value="">
              <SelectTrigger><SelectValue placeholder={t('protocolPages.shared.loadingEllipsis')} /></SelectTrigger>
              <SelectContent />
            </Select>
          ) : sortedVersions.length > 0 ? (
            <Select value={versionLabel} onValueChange={setVersionLabel}>
              <SelectTrigger>
                <SelectValue placeholder={t('protocolPages.importReview.finalize.versionPlaceholder')} />
              </SelectTrigger>
              <SelectContent>
                {sortedVersions.map((v) => (
                  <SelectItem key={v.id} value={v.version_label}>
                    {v.version_label}
                    {v.effective_date ? t('protocolPages.importReview.finalize.effectiveOn', { date: formatDate(v.effective_date) }) : ''}
                    {v.is_current ? t('protocolPages.importReview.finalize.currentTag') : ''}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : (
            // 院區尚未登記任何範本版本時，退回手動輸入（仍必填）
            <Input value={versionLabel} onChange={(e) => setVersionLabel(e.target.value)} placeholder={t('protocolPages.importReview.finalize.versionManualPlaceholder')} />
          )}
          {autoMatched && (
            <p className="text-xs text-muted-foreground">
              {t('protocolPages.importReview.finalize.autoMatched', { version: autoMatched })}
            </p>
          )}
        </div>
        <p className="text-sm text-muted-foreground">
          {t('protocolPages.importReview.finalize.description')}
        </p>
        {confirming ? (
          <div className="flex items-center gap-3 rounded-md border border-status-warning-border bg-status-warning-bg p-3">
            <span className="text-sm text-status-warning-text">{t('protocolPages.importReview.finalize.confirmPrompt')}</span>
            <Button className="ml-auto" onClick={() => finalizeMutation.mutate()} disabled={finalizeMutation.isPending}>
              {finalizeMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('protocolPages.importReview.finalize.confirmButton')}
            </Button>
            <Button variant="outline" onClick={() => setConfirming(false)} disabled={finalizeMutation.isPending}>
              {t('common.cancel')}
            </Button>
          </div>
        ) : (
          <Button onClick={handleConfirm}>{t('protocolPages.importReview.finalize.finalizeButton')}</Button>
        )}
      </CardContent>
    </Card>
  )
}
