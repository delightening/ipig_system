import { useTranslation } from 'react-i18next'
import { Key, LogOut, Menu, UserCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'

interface UserInfo {
  display_name?: string
  email?: string
  roles?: string[]
}

interface SidebarUserPanelProps {
  sidebarOpen: boolean
  setSidebarOpen: (open: boolean) => void
  user: UserInfo | null
  isAdmin: boolean
  onChangePassword: () => void
  onLogout: () => void
  onNavigateProfile: () => void
}

function UserAvatar({ user, size }: { user: UserInfo | null; size: 'sm' | 'md' }) {
  const sizeClass = size === 'sm' ? 'h-8 w-8' : 'h-9 w-9'
  return (
    <div className={`flex ${sizeClass} items-center justify-center rounded-full bg-blue-600 font-semibold text-sm`}>
      {user?.display_name?.[0] || user?.email?.[0] || 'U'}
    </div>
  )
}

function ExpandedPanel({
  user,
  isAdmin,
  onChangePassword,
  onLogout,
  onNavigateProfile,
}: Pick<SidebarUserPanelProps, 'user' | 'isAdmin' | 'onChangePassword' | 'onLogout' | 'onNavigateProfile'>) {
  const { t } = useTranslation()

  return (
    <div className="space-y-1.5 p-1.5">
      <div className="flex items-center space-x-2">
        <UserAvatar user={user} size="md" />
        <div className="flex-1 min-w-0">
          <p className="truncate text-sm font-medium">{user?.display_name || user?.email}</p>
          {isAdmin && (
            <p className="truncate text-xs text-slate-400">{user?.roles?.join(', ')}</p>
          )}
        </div>
      </div>
      <div className="flex flex-col gap-0.5">
        <Button
          variant="ghost"
          size="sm"
          onClick={onNavigateProfile}
          className="justify-start text-slate-400 hover:text-white hover:bg-slate-800 text-xs h-7"
        >
          <UserCircle className="h-4 w-4 mr-1.5 shrink-0" />
          {t('profile.settings')}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={onChangePassword}
          className="justify-start text-slate-400 hover:text-white hover:bg-slate-800 text-xs h-7"
        >
          <Key className="h-4 w-4 mr-1.5 shrink-0" />
          {t('common.changePassword')}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={onLogout}
          className="justify-start text-slate-400 hover:text-white hover:bg-slate-800 text-xs h-7"
        >
          <LogOut className="h-4 w-4 mr-1.5 shrink-0" />
          {t('common.logout')}
        </Button>
      </div>
    </div>
  )
}

function CollapsedPanel({
  user,
  setSidebarOpen,
}: Pick<SidebarUserPanelProps, 'user' | 'setSidebarOpen'>) {
  return (
    <div className="flex flex-col items-center space-y-2">
      <UserAvatar user={user} size="sm" />
      <Button
        variant="ghost"
        size="icon"
        onClick={() => setSidebarOpen(true)}
        className="text-slate-400 hover:text-white hover:bg-slate-800"
        aria-label="展開側邊欄"
      >
        <Menu className="h-5 w-5" />
      </Button>
    </div>
  )
}

export function SidebarUserPanel({
  sidebarOpen,
  setSidebarOpen,
  user,
  isAdmin,
  onChangePassword,
  onLogout,
  onNavigateProfile,
}: SidebarUserPanelProps) {
  return (
    <div className="border-t border-slate-700 p-1.5">
      {sidebarOpen ? (
        <ExpandedPanel
          user={user}
          isAdmin={isAdmin}
          onChangePassword={onChangePassword}
          onLogout={onLogout}
          onNavigateProfile={onNavigateProfile}
        />
      ) : (
        <CollapsedPanel user={user} setSidebarOpen={setSidebarOpen} />
      )}
    </div>
  )
}
