import { Mail, MessageSquare, Radio } from 'lucide-react'

export const channelOptions = [
    { value: 'in_app', label: '站內通知', icon: MessageSquare },
    { value: 'email', label: 'Email', icon: Mail },
    { value: 'both', label: '站內 + Email', icon: Radio },
] as const

export const GROUP_KEYS = ['AUP', 'Animal', 'ERP', 'HR'] as const

export type GroupKey = (typeof GROUP_KEYS)[number]
