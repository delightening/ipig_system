import { useState, useMemo } from 'react'
import { Link, useLocation, useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useAuthStore } from '@/stores/auth'
import { STALE_TIME } from '@/lib/query'
import { cn } from '@/lib/utils'
import api, { deleteResource } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import {
  LayoutDashboard,
  Package,
  Settings,
  LogOut,
  Menu,
  X,
  ChevronDown,
  Key,
  FileText,
  FolderOpen,
  Users,
  Stethoscope,
  GripVertical,
  RotateCcw,
  UserCircle,
  ClipboardCheck,
} from 'lucide-react'
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from '@dnd-kit/core'
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'

interface NavItem {
  title: string
  href?: string
  icon: React.ReactNode
  children?: { title: string; href: string; permission?: string; translate?: boolean }[]
  permission?: string
  badge?: number
  translate?: boolean
}

const DEFAULT_NAV_ORDER = [
  'dashboard',
  'QAU 品質保證',
  'myProjects',
  'aupReview',
  'animalManagement',
  '人員管理',
  'ERP',
  '系統管理',
]

const navItemsConfig: NavItem[] = [
  {
    title: 'dashboard',
    href: '/dashboard',
    icon: <LayoutDashboard className="h-6 w-6" />,
    permission: 'dashboard.view',
    translate: true,
  },
  {
    title: 'QAU 品質保證',
    href: '/admin/qau',
    icon: <ClipboardCheck className="h-6 w-6" />,
    permission: 'qau.dashboard.view',
    translate: false,
  },
  {
    title: 'myProjects',
    href: '/my-projects',
    icon: <FolderOpen className="h-6 w-6" />,
    translate: true,
  },
  {
    title: 'aupReview',
    icon: <FileText className="h-6 w-6" />,
    translate: true,
    children: [
      { title: 'protocolManagement', href: '/protocols', translate: true },
      { title: 'newProtocol', href: '/protocols/new', translate: true },
      { title: 'myAmendments', href: '/my-amendments', translate: true },
    ],
  },
  {
    title: '人員管理',
    icon: <Users className="h-6 w-6" />,
    translate: false,
    children: [
      { title: '出勤打卡', href: '/hr/attendance', translate: false },
      { title: '請假管理', href: '/hr/leaves', translate: false },
      { title: '加班管理', href: '/hr/overtime', translate: false },
      { title: '特休管理', href: '/hr/annual-leave', permission: 'hr.balance.manage', translate: false },
      { title: '人員訓練', href: '/hr/training-records', permission: 'training.view', translate: false },
      { title: '日曆', href: '/hr/calendar', translate: false },
    ],
  },
  {
    title: 'animalManagement',
    icon: <Stethoscope className="h-6 w-6" />,
    translate: true,
    children: [
      { title: 'animalList', href: '/animals', translate: true },
      { title: '血檢分析', href: '/blood-test-analysis', translate: false },
      { title: '血檢項目', href: '/blood-test-templates', translate: false },
      { title: '來源管理', href: '/animal-sources', permission: 'animal.source.manage', translate: false },
      { title: '動物欄位修正審核', href: '/animals/animal-field-corrections', permission: 'admin', translate: false },
    ],
  },
  {
    title: 'ERP',
    icon: <Package className="h-6 w-6" />,
    translate: false,
    permission: 'erp',
    children: [
      { title: '採購管理', href: '/erp?tab=purchasing', translate: false },
      { title: '銷貨管理', href: '/erp?tab=sales', translate: false },
      { title: '倉儲作業', href: '/erp?tab=warehouse', translate: false },
      { title: '設備維護', href: '/erp?tab=equipment', permission: 'equipment.view', translate: false },
      { title: '報表中心', href: '/erp?tab=reports', translate: false },
      { title: '產品管理', href: '/erp?tab=products', translate: false },
      { title: '供應商/客戶', href: '/erp?tab=partners', translate: false },
    ],
  },
  {
    title: '系統管理',
    icon: <Settings className="h-6 w-6" />,
    translate: false,
    children: [
      { title: '使用者管理', href: '/admin/users', translate: false },
      { title: '角色權限', href: '/admin/roles', translate: false },
      { title: '系統設定', href: '/admin/settings', translate: false },
      { title: '操作日誌', href: '/admin/audit-logs', translate: false },
      { title: '安全審計', href: '/admin/audit', translate: false },
      { title: '通知路由', href: '/admin/notification-routing', translate: false },
      { title: '藥物選單', href: '/admin/treatment-drugs', translate: false },
    ],
    permission: 'admin',
  },
]

function SortableNavItem({
  item,
  isEditMode,
  sidebarOpen,
  isActive,
  isChildActive,
  expandedItems,
  toggleExpand,
  navigate,
  translateTitle,
}: {
  item: NavItem
  isEditMode: boolean
  sidebarOpen: boolean
  isActive: (href: string) => boolean
  isChildActive: (item: NavItem) => boolean
  expandedItems: string[]
  toggleExpand: (title: string) => void
  navigate: (path: string) => void
  translateTitle: (item: { title: string; translate?: boolean }) => string
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: item.title })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  }

  return (
    <li ref={setNodeRef} style={style}>
      {item.href ? (
        <div className="flex items-center">
          {isEditMode && sidebarOpen && (
            <button
              {...attributes}
              {...listeners}
              className="p-1 mr-1 cursor-grab active:cursor-grabbing text-slate-500 hover:text-white"
            >
              <GripVertical className="h-4 w-4" />
            </button>
          )}
          <Link
            to={item.href}
            title={!sidebarOpen ? translateTitle(item) : undefined}
            className={cn(
              'flex flex-1 items-center rounded-lg py-2.5 transition-colors',
              isActive(item.href)
                ? 'bg-blue-600 text-white'
                : 'text-slate-300 hover:bg-slate-800 hover:text-white'
            )}
          >
            <span className="w-16 flex items-center justify-center shrink-0">{item.icon}</span>
            {sidebarOpen && (
              <span className="flex-1 flex items-center justify-between min-w-0 pr-3">
                <span className="truncate">{translateTitle(item)}</span>
                {item.badge && item.badge > 0 && (
                  <span className="ml-2 inline-flex items-center justify-center px-2 py-0.5 text-xs font-medium rounded-full bg-red-500 text-white shrink-0">
                    {item.badge > 99 ? '99+' : item.badge}
                  </span>
                )}
              </span>
            )}
          </Link>
        </div>
      ) : (
        <>
          <div className="flex items-center">
            {isEditMode && sidebarOpen && (
              <button
                {...attributes}
                {...listeners}
                className="p-1 mr-1 cursor-grab active:cursor-grabbing text-slate-500 hover:text-white"
              >
                <GripVertical className="h-4 w-4" />
              </button>
            )}
            <button
              onClick={() => {
                if (!sidebarOpen && item.children?.[0]?.href) {
                  navigate(item.children[0].href)
                } else {
                  toggleExpand(item.title)
                }
              }}
              title={!sidebarOpen ? translateTitle(item) : undefined}
              className={cn(
                'flex flex-1 items-center rounded-lg py-2.5 transition-colors',
                (!sidebarOpen && isChildActive(item))
                  ? 'bg-blue-600 text-white'
                  : 'text-slate-300 hover:bg-slate-800 hover:text-white'
              )}
            >
              <span className="w-16 flex items-center justify-center shrink-0">{item.icon}</span>
              {sidebarOpen && (
                <span className="flex-1 flex items-center justify-between min-w-0 pr-3">
                  <span className="truncate">{translateTitle(item)}</span>
                  <ChevronDown
                    className={cn(
                      'h-4 w-4 shrink-0 transition-transform',
                      expandedItems.includes(item.title) && 'rotate-180'
                    )}
                  />
                </span>
              )}
            </button>
          </div>
          {sidebarOpen && expandedItems.includes(item.title) && item.children && (
            <ul className="ml-4 mt-1 space-y-1 border-l border-slate-700 pl-4">
              {item.children.map((child) => (
                <li key={child.href}>
                  <Link
                    to={child.href}
                    className={cn(
                      'block rounded-lg px-3 py-2 text-sm transition-colors',
                      isActive(child.href)
                        ? 'bg-blue-600 text-white'
                        : 'text-slate-400 hover:bg-slate-800 hover:text-white'
                    )}
                  >
                    {translateTitle(child)}
                  </Link>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </li>
  )
}

interface SidebarProps {
  sidebarOpen: boolean
  setSidebarOpen: (open: boolean) => void
  mobileSidebarOpen: boolean
  setMobileSidebarOpen: (open: boolean) => void
  onChangePassword: () => void
}

export function Sidebar({
  sidebarOpen,
  setSidebarOpen,
  mobileSidebarOpen,
  setMobileSidebarOpen,
  onChangePassword,
}: SidebarProps) {
  const location = useLocation()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { user, hasRole, hasPermission, logout } = useAuthStore()
  const { t } = useTranslation()

  const [expandedItems, setExpandedItems] = useState<string[]>([])
  const [isEditMode, setIsEditMode] = useState(false)

  const translateTitle = (item: { title: string; translate?: boolean }) => {
    if (item.translate === false) return item.title
    return t(`nav.${item.title}`) || item.title
  }

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  )

  const { data: navOrderData } = useQuery({
    queryKey: ['user-preferences', 'nav_order'],
    queryFn: async () => {
      const res = await api.get<{ key: string; value: string[] }>('/me/preferences/nav_order')
      return res.data.value
    },
    staleTime: STALE_TIME.SETTINGS,
  })

  const saveNavOrderMutation = useMutation({
    mutationFn: async (order: string[]) => {
      return api.put('/me/preferences/nav_order', { value: order })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user-preferences', 'nav_order'] })
    },
  })

  const resetNavOrderMutation = useMutation({
    mutationFn: async () => {
      return deleteResource('/me/preferences/nav_order')
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user-preferences', 'nav_order'] })
      toast({ title: t('common.success'), description: t('common.resetSuccess') })
    },
  })

  const sortedNavItems = useMemo(() => {
    const order = navOrderData || DEFAULT_NAV_ORDER
    return [...navItemsConfig].sort((a, b) => {
      const indexA = order.indexOf(a.title)
      const indexB = order.indexOf(b.title)
      const posA = indexA === -1 ? 999 : indexA
      const posB = indexB === -1 ? 999 : indexB
      return posA - posB
    })
  }, [navOrderData])

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    if (over && active.id !== over.id) {
      const oldIndex = sortedNavItems.findIndex((item) => item.title === active.id)
      const newIndex = sortedNavItems.findIndex((item) => item.title === over.id)
      const newOrder = arrayMove(sortedNavItems.map(i => i.title), oldIndex, newIndex)
      saveNavOrderMutation.mutate(newOrder)
    }
  }

  const handleResetNavOrder = () => {
    resetNavOrderMutation.mutate()
  }

  const { data: pendingAmendmentsCount } = useQuery({
    queryKey: ['amendments-pending-count'],
    queryFn: async () => {
      const res = await api.get<{ count: number }>('/amendments/pending-count')
      return res.data.count
    },
    staleTime: STALE_TIME.LIST,
    refetchInterval: 60000,
    enabled: !!user,
  })

  const toggleExpand = (title: string) => {
    setExpandedItems((prev) =>
      prev.includes(title)
        ? prev.filter((item) => item !== title)
        : [...prev, title]
    )
  }

  const isActive = (href: string) => {
    if (href.includes('?')) {
      return location.pathname + location.search === href
    }
    return location.pathname === href
  }

  const isChildActive = (item: NavItem): boolean => {
    return item.children?.some((child) => isActive(child.href)) || false
  }

  const filteredNavItems = useMemo(() => {
    const rolesWithoutHrAccess = ['REVIEWER', 'VET', 'IACUC_CHAIR', 'PI']
    const shouldHideHr = user?.roles?.every(r =>
      rolesWithoutHrAccess.includes(r)
    ) && user?.roles?.some(r => rolesWithoutHrAccess.includes(r))

    return sortedNavItems
      .filter((item) => {
        if (item.title === '人員管理' && shouldHideHr) {
          return false
        }
        if (item.permission === 'erp') {
          const hasErpAccess = hasRole('admin') ||
            user?.permissions?.some(p => p.startsWith('erp.')) ||
            user?.permissions?.some(p => p.startsWith('equipment.'))
          return hasErpAccess
        }
        if (item.permission && !hasPermission(item.permission) && !hasRole(item.permission)) {
          return false
        }
        return true
      })
      .map((item) => {
        if (item.children) {
          const filteredChildren = item.children.filter(child => {
            if (child.permission) {
              return hasPermission(child.permission) || hasRole('admin')
            }
            return true
          })
          return { ...item, children: filteredChildren }
        }
        return item
      })
      .filter((item) => {
        if (item.children && item.children.length === 0) {
          return false
        }
        return true
      })
      .map((item) => {
        if (item.title === '我的變更申請' && pendingAmendmentsCount) {
          return { ...item, badge: pendingAmendmentsCount }
        }
        return item
      })
  }, [sortedNavItems, hasRole, hasPermission, user, pendingAmendmentsCount])

  return (
    <>
      {mobileSidebarOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/50 md:hidden"
          onClick={() => setMobileSidebarOpen(false)}
        />
      )}

      <aside
        className={cn(
          'fixed inset-y-0 left-0 z-50 flex flex-col bg-slate-900 text-white transition-[width,transform] duration-300 ease-in-out overflow-hidden',
          mobileSidebarOpen ? 'translate-x-0' : '-translate-x-full',
          'md:translate-x-0 md:relative',
          sidebarOpen ? 'w-52' : 'md:w-16 w-52'
        )}
      >
        <div className="flex h-16 items-center border-b border-slate-700">
          <button
            onClick={() => {
              if (window.innerWidth < 768) {
                setMobileSidebarOpen(false)
              } else {
                setSidebarOpen(!sidebarOpen)
              }
            }}
            className="flex items-center hover:opacity-80 transition-opacity"
            title={sidebarOpen ? t('nav.collapseSidebar') : t('nav.expandSidebar')}
          >
            <span className="w-16 flex items-center justify-center shrink-0">
              <img src="/pigmodel%20logo%20dark.png" alt="Logo" className="h-9 w-9 object-contain" />
            </span>
            {sidebarOpen && (
              <span className="text-xl font-bold whitespace-nowrap">ipig system</span>
            )}
          </button>
        </div>

        <nav className="flex-1 overflow-y-auto py-4">
          {isEditMode && sidebarOpen && (
            <div className="mb-3 p-2 bg-blue-600/20 rounded-lg text-xs text-blue-300 text-center">
              {t('nav.editModeHint')}
            </div>
          )}
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragEnd={handleDragEnd}
          >
            <SortableContext
              items={filteredNavItems.map(i => i.title)}
              strategy={verticalListSortingStrategy}
            >
              <ul className="space-y-1">
                {filteredNavItems.map((item) => (
                  <SortableNavItem
                    key={item.title}
                    item={item}
                    isEditMode={isEditMode}
                    sidebarOpen={sidebarOpen}
                    isActive={isActive}
                    isChildActive={isChildActive}
                    expandedItems={expandedItems}
                    toggleExpand={toggleExpand}
                    navigate={navigate}
                    translateTitle={translateTitle}
                  />
                ))}
              </ul>
            </SortableContext>
          </DndContext>

          {sidebarOpen && (
            <div className="mt-4 pt-4 border-t border-slate-700 space-y-2">
              {isEditMode ? (
                <>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setIsEditMode(false)}
                    className="w-full text-slate-400 hover:text-white hover:bg-slate-800 text-xs"
                  >
                    <X className="h-4 w-4 mr-1" />
                    {t('common.finishEdit')}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleResetNavOrder}
                    disabled={resetNavOrderMutation.isPending}
                    className="w-full text-slate-400 hover:text-white hover:bg-slate-800 text-xs"
                  >
                    <RotateCcw className="h-4 w-4 mr-1" />
                    {t('common.resetToDefault')}
                  </Button>
                </>
              ) : (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setIsEditMode(true)}
                  className="w-full text-slate-400 hover:text-white hover:bg-slate-800 text-xs"
                >
                  <GripVertical className="h-4 w-4 mr-1" />
                  {t('common.editMenuOrder')}
                </Button>
              )}
            </div>
          )}
        </nav>

        <div className="border-t border-slate-700 p-1.5">
          {sidebarOpen ? (
            <div className="space-y-1.5 p-1.5">
              <div className="flex items-center space-x-2">
                <div className="flex h-9 w-9 items-center justify-center rounded-full bg-blue-600 font-semibold text-sm">
                  {user?.display_name?.[0] || user?.email?.[0] || 'U'}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="truncate text-sm font-medium">{user?.display_name || user?.email}</p>
                  {hasRole('admin') && (
                    <p className="truncate text-xs text-slate-400">{user?.roles?.join(', ')}</p>
                  )}
                </div>
              </div>
              <div className="flex flex-col gap-0.5">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => navigate('/profile/settings')}
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
                  onClick={logout}
                  className="justify-start text-slate-400 hover:text-white hover:bg-slate-800 text-xs h-7"
                >
                  <LogOut className="h-4 w-4 mr-1.5 shrink-0" />
                  {t('common.logout')}
                </Button>
              </div>
            </div>
          ) : (
            <div className="flex flex-col items-center space-y-2">
              <div className="flex h-8 w-8 items-center justify-center rounded-full bg-blue-600 font-semibold text-sm">
                {user?.display_name?.[0] || user?.email?.[0] || 'U'}
              </div>
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
          )}
        </div>
      </aside>
    </>
  )
}
