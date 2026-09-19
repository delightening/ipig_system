import { useTranslation } from 'react-i18next'

import { Badge } from '@/components/ui/badge'

import { channelOptions } from '../constants'

interface ChannelBadgeProps {
    channel: string
}

export function ChannelBadge({ channel }: ChannelBadgeProps) {
    const { t } = useTranslation()
    const opt = channelOptions.find((o) => o.value === channel)
    if (!opt) return <Badge variant="outline">{channel}</Badge>
    const Icon = opt.icon
    return (
        <Badge variant="secondary" className="gap-1">
            <Icon className="h-3 w-3" />
            {t(opt.labelKey)}
        </Badge>
    )
}
