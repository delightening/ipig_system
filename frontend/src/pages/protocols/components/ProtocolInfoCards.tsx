import { useTranslation } from 'react-i18next'
import { FileText, User as UserIcon, UserCog, Upload, Building, Calendar } from 'lucide-react'

import { formatDate } from '@/lib/utils'
import type { Protocol } from '@/lib/api'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

interface ProtocolInfoCardsProps {
  protocol: Protocol
  piName?: string
  piOrganization?: string
  sdName?: string
  createdByName?: string
}

export function ProtocolInfoCards({
  protocol,
  piName,
  piOrganization,
  sdName,
  createdByName,
}: ProtocolInfoCardsProps) {
  const { t } = useTranslation()

  const basic = protocol.working_content?.basic
  // PI（客人）以研究資料 basic.pi 為準（外部 PI 匯入時與 FK 匯入者不同），fallback FK 使用者
  const realPiName = basic?.pi?.name?.trim() || piName
  // PI email 只認 basic.pi.email；不 fallback 到 FK（匯入者/負責人）email，避免顯示錯人的信箱。
  // 未填時顯示「待填」紅字提示補登。
  const realPiEmail = basic?.pi?.email?.trim()
  // 委託單位：研究資料 sponsor 單位名稱，fallback PI 機構
  const sponsorName = basic?.sponsor?.name?.trim() || piOrganization

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <FileText className="h-4 w-4 text-primary" />
            {protocol.iacuc_no?.startsWith('APIG-') ? t('protocols.detail.info.apigNo') : t('protocols.detail.info.iacucNo')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-xl font-bold text-status-warning-text">
            {protocol.iacuc_no || t('protocols.detail.info.notIssued')}
          </p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <UserIcon className="h-4 w-4 text-status-success-text" />
            {t('protocols.detail.info.pi')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-lg font-semibold">{realPiName || '-'}</p>
          {realPiEmail ? (
            <p className="text-sm text-muted-foreground">{realPiEmail}</p>
          ) : (
            <p className="text-sm text-destructive">待填</p>
          )}
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <UserCog className="h-4 w-4 text-primary" />
            {t('protocols.detail.info.sd')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-lg font-semibold">{sdName || '-'}</p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Upload className="h-4 w-4 text-muted-foreground" />
            {t('protocols.detail.info.creator')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-lg font-semibold">{createdByName || '-'}</p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Building className="h-4 w-4 text-primary" />
            {t('protocols.detail.info.sponsor')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-lg font-semibold">{sponsorName || '-'}</p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Calendar className="h-4 w-4 text-status-warning-text" />
            {t('protocols.detail.info.period')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-lg font-semibold">
            {protocol.start_date && protocol.end_date
              ? `${formatDate(protocol.start_date)} ~ ${formatDate(protocol.end_date)}`
              : t('protocols.detail.info.notSet')}
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
