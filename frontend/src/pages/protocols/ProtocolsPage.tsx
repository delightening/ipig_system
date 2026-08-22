import { useTranslation } from 'react-i18next'
import { FileText, History, UserPlus, ClipboardCheck } from 'lucide-react'

import { PageHeader } from '@/components/ui/page-header'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import { useAuthHasRole } from '@/stores/auth'

import { ProtocolListTab } from './components/ProtocolListTab'
import { ProtocolTemplateVersionsTab } from './components/template-versions/ProtocolTemplateVersionsTab'
import { ApplicationNoticesTab } from './components/application-notices/ApplicationNoticesTab'
import { PiAccountInvitesTab } from './components/PiAccountInvitesTab'

/**
 * 計畫書管理：計畫書清單分頁 + 計畫書範本版本登記分頁（後者僅系統管理員可見）。
 */
export function ProtocolsPage() {
  const { t } = useTranslation()
  const hasRole = useAuthHasRole()
  const isAdmin = hasRole('SYSTEM_ADMIN') || hasRole('admin')

  const tabs = [
    { value: 'list', label: t('protocols.title'), icon: FileText },
    { value: 'templates', label: '計畫書範本版本', icon: History, hidden: !isAdmin },
    { value: 'app-notices', label: '申請須知版本', icon: ClipboardCheck, hidden: !isAdmin },
    { value: 'pi-invites', label: 'PI 帳號開通', icon: UserPlus, hidden: !isAdmin },
  ]

  return (
    <div className="space-y-6">
      <PageHeader title={t('protocols.title')} description={t('protocols.subtitle')} />
      <PageTabs tabs={tabs} defaultTab="list">
        <PageTabContent value="list">
          <ProtocolListTab />
        </PageTabContent>
        <PageTabContent value="templates">
          <ProtocolTemplateVersionsTab />
        </PageTabContent>
        <PageTabContent value="app-notices">
          <ApplicationNoticesTab />
        </PageTabContent>
        <PageTabContent value="pi-invites">
          <PiAccountInvitesTab />
        </PageTabContent>
      </PageTabs>
    </div>
  )
}
