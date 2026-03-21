import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'

import {
  Clock,
  History,
  MessageSquare,
  FileText,
  UserPlus,
  Paperclip,
  Users,
  ClipboardList,
  FileEdit,
} from 'lucide-react'
import type { TabKey } from '../constants'

interface ProtocolTabNavProps {
  activeTab: TabKey
  onTabChange: (tab: TabKey) => void
  shouldAnonymizeReviewers: boolean
}

export function ProtocolTabNav({ activeTab, onTabChange, shouldAnonymizeReviewers }: ProtocolTabNavProps) {
  const { t } = useTranslation()

  const tabItems = useMemo<{ key: TabKey; label: string; icon: typeof FileText }[]>(() => [
    { key: 'content', label: t('protocols.detail.tabs.content'), icon: FileText },
    { key: 'animals', label: t('protocols.detail.tabs.animals'), icon: ClipboardList },
    { key: 'versions', label: t('protocols.detail.tabs.versions'), icon: History },
    { key: 'history', label: t('protocols.detail.tabs.history'), icon: Clock },
    { key: 'comments', label: t('protocols.detail.tabs.comments'), icon: MessageSquare },
    { key: 'reviewers', label: t('protocols.detail.tabs.reviewers'), icon: Users },
    { key: 'coeditors', label: t('protocols.detail.tabs.coeditors'), icon: UserPlus },
    { key: 'attachments', label: t('protocols.detail.tabs.attachments'), icon: Paperclip },
    { key: 'amendments', label: t('protocols.detail.tabs.amendments'), icon: FileEdit },
  ], [t])

  return (
    <div className="border-b">
      <nav className="flex flex-wrap gap-4">
        {tabItems
          .filter(tab => {
            if (tab.key === 'reviewers') return !shouldAnonymizeReviewers
            return true
          })
          .map((tab) => (
            <button
              key={tab.key}
              onClick={() => onTabChange(tab.key)}
              className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors ${activeTab === tab.key
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-muted-foreground hover:text-foreground'
                }`}
            >
              <tab.icon className="h-4 w-4" />
              {tab.label}
            </button>
          ))}
      </nav>
    </div>
  )
}
