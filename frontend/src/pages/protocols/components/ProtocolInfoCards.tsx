import { useTranslation } from 'react-i18next'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { FileText, User as UserIcon, Building, Calendar } from 'lucide-react'
import { formatDate } from '@/lib/utils'
import type { Protocol } from '@/lib/api'

interface ProtocolInfoCardsProps {
  protocol: Protocol
  piName?: string
  piEmail?: string
  piOrganization?: string
}

export function ProtocolInfoCards({ protocol, piName, piEmail, piOrganization }: ProtocolInfoCardsProps) {
  const { t } = useTranslation()

  return (
    <div className="grid gap-4 md:grid-cols-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <FileText className="h-4 w-4 text-blue-500" />
            {protocol.iacuc_no?.startsWith('APIG-') ? t('protocols.detail.info.apigNo') : t('protocols.detail.info.iacucNo')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-xl font-bold text-orange-600">
            {protocol.iacuc_no || t('protocols.detail.info.notIssued')}
          </p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <UserIcon className="h-4 w-4 text-green-500" />
            {t('protocols.detail.info.pi')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-lg font-semibold">{piName || '-'}</p>
          <p className="text-sm text-muted-foreground">{piEmail}</p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Building className="h-4 w-4 text-purple-500" />
            {t('protocols.detail.info.organization')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-lg font-semibold">{piOrganization || '-'}</p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Calendar className="h-4 w-4 text-yellow-500" />
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
