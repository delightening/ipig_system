import { useState, useEffect, useRef, useMemo } from 'react' // 引入 React 核心 Hook：狀態、副作用、引用、效能優化
import { Outlet, Link, useLocation, useNavigate } from 'react-router-dom' // 引入路由組件與導覽 Hook
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query' // 引入資料獲取與變更管理工具
import { useTranslation } from 'react-i18next' // 引入 i18n 翻譯 Hook
import { useAuthStore } from '@/stores/auth' // 引入權限管理 Store (Zustand)
import { cn } from '@/lib/utils' // 引入 CSS 類名合併工具
import api, { ChangeOwnPasswordRequest, NotificationItem } from '@/lib/api' // 引入 API 定義與類型
import { Button } from '@/components/ui/button' // 引入按鈕組件
import { Input } from '@/components/ui/input' // 引入輸入框組件
import { Label } from '@/components/ui/label' // 引入標籤組件
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select' // 引入選擇器組件
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog' // 引入對話框組件
import { toast } from '@/components/ui/use-toast' // 引入通知提醒工具
import { // 引入一系列 Lucide 圖示
  LayoutDashboard,
  Package,
  Warehouse,
  BarChart3,
  Settings,
  LogOut,
  Menu,
  X,
  ChevronDown,
  Truck,
  ShoppingCart,
  Globe,
  Key,
  Loader2,
  FileText,
  FolderOpen,
  Users,
  Stethoscope,
  Bell,
  CheckCheck,
  ExternalLink,
  GripVertical,
  RotateCcw,
  FileEdit,
  UserCircle,
  ArrowLeft,
} from 'lucide-react'
// 引入 dnd-kit 拖曳排序相關函式庫
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

// 定義導覽選單項目的介面規格
interface NavItem {
  title: string // 顯示名稱
  href?: string // 跳轉連結（如果有）
  icon: React.ReactNode // 顯示圖示
  children?: { title: string; href: string; permission?: string }[] // 子選單（選填）
  permission?: string // 需要的權限代碼（選填）
  badge?: number // 待處理數量徽章（選填）
}

// 預設導覽選單順序（使用者可自行調整）
const DEFAULT_NAV_ORDER = [
  '儀表板',
  '我的計劃',
  'AUP',
  '人員管理',
  '實驗動物管理',
  '採購管理',
  '銷售管理',
  '倉儲作業',
  '報表中心',
  '基礎資料',
  '系統管理',
]

// 靜態定義側邊導覽列的所有項目（以 id 識別）
const navItemsConfig: NavItem[] = [
  {
    title: 'dashboard',
    href: '/dashboard',
    icon: <LayoutDashboard className="h-5 w-5" />,
    permission: 'erp',
  },
  {
    title: 'myProjects',
    href: '/my-projects',
    icon: <FolderOpen className="h-5 w-5" />,
  },
  {
    title: 'aupReview',
    icon: <FileText className="h-5 w-5" />,
    children: [
      { title: 'protocols', href: '/protocols' },
      { title: 'newProtocol', href: '/protocols/new' },
      { title: 'myAmendments', href: '/my-amendments' },
    ],
  },
  {
    title: 'hrManagement',
    icon: <Users className="h-5 w-5" />,
    children: [
      { title: 'attendance', href: '/hr/attendance' },
      { title: 'leaves', href: '/hr/leaves' },
      { title: 'overtime', href: '/hr/overtime' },
      { title: 'annualLeave', href: '/hr/annual-leave', permission: 'hr.balance.manage' },
      { title: 'calendar', href: '/hr/calendar' },
    ],
  },
  {
    title: 'animalManagement',
    icon: <Stethoscope className="h-5 w-5" />,
    children: [
      { title: 'animalList', href: '/pigs' },
      { title: 'sourceManagement', href: '/pig-sources', permission: 'pig.source.manage' },
    ],
  },
  {
    title: 'purchasing',
    icon: <Truck className="h-5 w-5" />,
    children: [
      { title: 'purchaseOrder', href: '/documents?type=PO' },
      { title: 'purchaseReturn', href: '/documents?type=PR' },
    ],
    permission: 'erp',
  },
  {
    title: 'sales',
    icon: <ShoppingCart className="h-5 w-5" />,
    children: [
      { title: 'salesOrder', href: '/documents?type=SO' },
      { title: 'deliveryOrder', href: '/documents?type=DO' },
    ],
    permission: 'erp',
  },
  {
    title: 'warehouse',
    icon: <Warehouse className="h-5 w-5" />,
    children: [
      { title: 'inventory', href: '/inventory' },
      { title: 'inventoryLedger', href: '/inventory/ledger' },
      { title: 'transfer', href: '/documents?type=TR' },
      { title: 'stocktaking', href: '/documents?type=STK' },
      { title: 'adjustment', href: '/documents?type=ADJ' },
    ],
    permission: 'erp',
  },
  {
    title: 'reports',
    icon: <BarChart3 className="h-5 w-5" />,
    children: [
      { title: 'stockOnHand', href: '/reports/stock-on-hand' },
      { title: 'stockLedger', href: '/reports/stock-ledger' },
      { title: 'purchaseLines', href: '/reports/purchase-lines' },
      { title: 'salesLines', href: '/reports/sales-lines' },
      { title: 'costSummary', href: '/reports/cost-summary' },
    ],
    permission: 'erp',
  },
  {
    title: 'masterData',
    icon: <Package className="h-5 w-5" />,
    children: [
      { title: 'products', href: '/products' },
      { title: 'warehouses', href: '/warehouses' },
      { title: 'partners', href: '/partners' },
    ],
    permission: 'erp',
  },
  {
    title: 'system',
    icon: <Settings className="h-5 w-5" />,
    children: [
      { title: 'users', href: '/admin/users' },
      { title: 'roles', href: '/admin/roles' },
      { title: 'settings', href: '/admin/settings' },
      { title: 'auditLogs', href: '/admin/audit-logs' },
      { title: 'securityAudit', href: '/admin/audit' },
    ],
    permission: 'admin',
  },
]

// localStorage 鍵名
const NAV_ORDER_STORAGE_KEY = 'ipig-nav-order'

// 可排序的導覽項目元件
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
  translateTitle: (title: string) => string
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
        // 情況 A：直接連結項目
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
            title={!sidebarOpen ? translateTitle(item.title) : undefined}
            className={cn(
              'flex flex-1 items-center rounded-lg px-3 py-2.5 transition-colors',
              sidebarOpen ? 'space-x-3' : 'justify-center',
              isActive(item.href)
                ? 'bg-blue-600 text-white'
                : 'text-slate-300 hover:bg-slate-800 hover:text-white'
            )}
          >
            <span className="shrink-0">{item.icon}</span>
            {sidebarOpen && (
              <span className="flex-1 flex items-center justify-between">
                <span>{translateTitle(item.title)}</span>
                {item.badge && item.badge > 0 && (
                  <span className="ml-2 inline-flex items-center justify-center px-2 py-0.5 text-xs font-medium rounded-full bg-red-500 text-white">
                    {item.badge > 99 ? '99+' : item.badge}
                  </span>
                )}
              </span>
            )}
            {!sidebarOpen && item.badge && item.badge > 0 && (
              <span className="absolute -top-1 -right-1 inline-flex items-center justify-center w-5 h-5 text-xs font-medium rounded-full bg-red-500 text-white">
                {item.badge > 9 ? '9+' : item.badge}
              </span>
            )}
          </Link>
        </div>
      ) : (
        // 情況 B：包含子選單的折疊項目
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
              title={!sidebarOpen ? translateTitle(item.title) : undefined}
              className={cn(
                'flex flex-1 items-center rounded-lg px-3 py-2.5 transition-colors',
                sidebarOpen ? 'justify-between' : 'justify-center',
                (!sidebarOpen && isChildActive(item))
                  ? 'bg-blue-600 text-white'
                  : 'text-slate-300 hover:bg-slate-800 hover:text-white'
              )}
            >
              <div className={cn(
                'flex items-center',
                sidebarOpen ? 'space-x-3' : ''
              )}>
                <span className="shrink-0">{item.icon}</span>
                {sidebarOpen && <span>{translateTitle(item.title)}</span>}
              </div>
              {sidebarOpen && (
                <ChevronDown
                  className={cn(
                    'h-4 w-4 transition-transform',
                    expandedItems.includes(item.title) && 'rotate-180'
                  )}
                />
              )}
            </button>
          </div>
          {/* 子選單內容 */}
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
                    {translateTitle(child.title)}
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

export function MainLayout() {
  const location = useLocation() // 取得當前 URL 路徑資訊
  const navigate = useNavigate() // 用於程式化導覽跳轉
  const queryClient = useQueryClient() // TanStack Query 快取管理器
  const { user, logout, hasRole, hasPermission, isImpersonating, stopImpersonating } = useAuthStore() // 從 Auth Store 取得用戶資訊、登出方法與權限檢查
  const { t, i18n } = useTranslation() // 引入翻譯函數和 i18n 實例
  const [sidebarOpen, setSidebarOpen] = useState(true) // 控制側邊欄展開/縮小的狀態
  const [expandedItems, setExpandedItems] = useState<string[]>([]) // 控制側邊欄摺疊選單展開項目的清單（預設全部收合）

  // 語言切換處理函數
  const handleLanguageChange = (lang: string) => {
    i18n.changeLanguage(lang)
  }

  // 導覽項目翻譯映射表
  const navTitleMap: Record<string, string> = {
    '儀表板': t('nav.dashboard'),
    '我的計劃': t('nav.myProjects'),
    'AUP': t('nav.aupReview'),
    '計畫書管理': t('nav.protocols'),
    '新增計畫書': t('nav.newProtocol'),
    '變更申請': t('nav.myAmendments'),
    '人員管理': t('nav.hrManagement'),
    '出勤打卡': t('nav.attendance'),
    '請假管理': t('nav.leaves'),
    '加班管理': t('nav.overtime'),
    '特休額度管理': t('nav.annualLeave'),
    '日曆': t('nav.calendar'),
    '實驗動物管理': t('nav.animalManagement'),
    '動物列表': t('nav.animalList'),
    '來源管理': t('nav.sourceManagement'),
    '採購管理': t('nav.purchasing'),
    '採購單': t('nav.purchaseOrder'),
    '採購入庫': t('nav.goodsReceipt'),
    '採購退貨': t('nav.purchaseReturn'),
    '銷售管理': t('nav.sales'),
    '銷售單': t('nav.salesOrder'),
    '銷售出庫': t('nav.deliveryOrder'),
    '倉儲作業': t('nav.warehouse'),
    '庫存查詢': t('nav.inventory'),
    '庫存流水': t('nav.inventoryLedger'),
    '調撥單': t('nav.transfer'),
    '盤點單': t('nav.stocktaking'),
    '調整單': t('nav.adjustment'),
    '報表中心': t('nav.reports'),
    '庫存現況報表': t('nav.stockOnHand'),
    '庫存流水報表': t('nav.stockLedger'),
    '採購明細報表': t('nav.purchaseLines'),
    '銷售明細報表': t('nav.salesLines'),
    '成本摘要報表': t('nav.costSummary'),
    '基礎資料': t('nav.masterData'),
    '產品管理': t('nav.products'),
    '倉庫管理': t('nav.warehouses'),
    '供應商/客戶': t('nav.partners'),
    '系統管理': t('nav.system'),
    '使用者管理': t('nav.users'),
    '角色權限': t('nav.roles'),
    '系統設定': t('nav.settings'),
    '審計日誌': t('nav.auditLogs'),
    '安全審計': t('nav.securityAudit'),
  }

  // 翻譯導覽項目標題
  const translateTitle = (title: string) => t(`nav.${title}`) || title

  // 通知下拉選單的顯示狀態與引用
  const [showNotificationDropdown, setShowNotificationDropdown] = useState(false)
  const notificationRef = useRef<HTMLDivElement>(null)

  // 修改密碼對話框的相關狀態
  const [showPasswordDialog, setShowPasswordDialog] = useState(false)
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')

  // 側邊欄編輯排序模式狀態
  const [isEditMode, setIsEditMode] = useState(false)

  // dnd-kit sensors 配置
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  )

  // 從後端取得導覽列排序偏好
  const { data: navOrderData } = useQuery({
    queryKey: ['user-preferences', 'nav_order'],
    queryFn: async () => {
      const res = await api.get<{ key: string; value: string[] }>('/me/preferences/nav_order')
      return res.data.value
    },
  })

  // 儲存導覽列排序偏好
  const saveNavOrderMutation = useMutation({
    mutationFn: async (order: string[]) => {
      return api.put('/me/preferences/nav_order', { value: order })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user-preferences', 'nav_order'] })
    },
  })

  // 重置導覽列排序
  const resetNavOrderMutation = useMutation({
    mutationFn: async () => {
      return api.delete('/me/preferences/nav_order')
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user-preferences', 'nav_order'] })
      toast({ title: t('common.success'), description: t('common.resetSuccess') })
    },
  })

  // 根據偏好設定排序導覽項目
  const sortedNavItems = useMemo(() => {
    const order = navOrderData || DEFAULT_NAV_ORDER
    return [...navItemsConfig].sort((a, b) => {
      const indexA = order.indexOf(a.title)
      const indexB = order.indexOf(b.title)
      // 如果找不到，放到最後
      const posA = indexA === -1 ? 999 : indexA
      const posB = indexB === -1 ? 999 : indexB
      return posA - posB
    })
  }, [navOrderData])

  // 處理拖曳結束事件
  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    if (over && active.id !== over.id) {
      const oldIndex = sortedNavItems.findIndex((item) => item.title === active.id)
      const newIndex = sortedNavItems.findIndex((item) => item.title === over.id)
      const newOrder = arrayMove(sortedNavItems.map(i => i.title), oldIndex, newIndex)
      saveNavOrderMutation.mutate(newOrder)
    }
  }

  // 重置排序
  const handleResetNavOrder = () => {
    resetNavOrderMutation.mutate()
  }

  // 取得未讀通知數量的 API 查詢
  const { data: unreadCount } = useQuery({
    queryKey: ['notifications-unread-count'],
    queryFn: async () => {
      const res = await api.get<{ count: number }>('/notifications/unread-count')
      return res.data.count
    },
    refetchInterval: 60000, // 每 60 秒自動重新獲取最新數量
  })

  // 取得待處理變更申請數量
  const { data: pendingAmendmentsCount } = useQuery({
    queryKey: ['amendments-pending-count'],
    queryFn: async () => {
      const res = await api.get<{ count: number }>('/amendments/pending-count')
      return res.data.count
    },
    refetchInterval: 60000, // 每 60 秒自動重新獲取
  })

  // 取得最近 10 筆通知的 API 查詢 (僅在選單開啟時觸發)
  const { data: notificationsData } = useQuery({
    queryKey: ['notifications-recent'],
    queryFn: async () => {
      const res = await api.get<{ data: NotificationItem[] }>('/notifications?per_page=10')
      return res.data.data
    },
    enabled: showNotificationDropdown,
  })

  // 標記特定通知為已讀的變更操作
  const markReadMutation = useMutation({
    mutationFn: async (ids: string[]) => {
      return api.post('/notifications/mark-read', { notification_ids: ids })
    },
    onSuccess: () => {
      // 成功後刷新未讀數量與最近通知列表
      queryClient.invalidateQueries({ queryKey: ['notifications-unread-count'] })
      queryClient.invalidateQueries({ queryKey: ['notifications-recent'] })
    },
  })

  // 標記「全部」通知為已讀的變更操作
  const markAllReadMutation = useMutation({
    mutationFn: async () => {
      return api.post('/notifications/mark-all-read')
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notifications-unread-count'] })
      queryClient.invalidateQueries({ queryKey: ['notifications-recent'] })
      toast({ title: t('common.success'), description: t('common.saved') })
    },
  })

  // 副作用：點擊通知選單以外的區域時，自動關閉選單
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (notificationRef.current && !notificationRef.current.contains(event.target as Node)) {
        setShowNotificationDropdown(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  // 處理通知項目點擊邏輯
  const handleNotificationClick = (notification: NotificationItem) => {
    if (!notification.is_read) {
      markReadMutation.mutate([notification.id]) // 若未讀則標記為已讀
    }
    setShowNotificationDropdown(false) // 關閉下拉選單

    // 根據通知關聯的實體類型，自動導航到對應頁面
    if (notification.related_entity_type && notification.related_entity_id) {
      switch (notification.related_entity_type) {
        case 'protocol':
          navigate(`/protocols/${notification.related_entity_id}`) // 計畫書詳細頁
          break
        case 'document':
          navigate(`/documents/${notification.related_entity_id}`) // iPig ERP 單據頁
          break
        case 'pig':
          navigate(`/pigs/${notification.related_entity_id}`) // 動物詳細頁
          break
      }
    }
  }

  // 格式化顯示通知時間 (如：幾分鐘前)
  const formatNotificationTime = (dateStr: string) => {
    const date = new Date(dateStr)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const minutes = Math.floor(diff / 60000)
    const hours = Math.floor(diff / 3600000)
    const days = Math.floor(diff / 86400000)

    if (minutes < 1) return t('common.justNow')
    if (minutes < 60) return t('common.minutesAgo', { count: minutes })
    if (hours < 24) return t('common.hoursAgo', { count: hours })
    if (days < 7) return t('common.daysAgo', { count: days })
    return date.toLocaleDateString(i18n.language)
  }

  // 修改密碼的 API 變更操作
  const changePasswordMutation = useMutation({
    mutationFn: async (data: ChangeOwnPasswordRequest) => {
      return api.put('/me/password', data)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('password.success') })
      setShowPasswordDialog(false)
      resetPasswordForm()
      logout() // 改完密碼強制登出，要求使用者重新驗證
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('password.failed'),
        variant: 'destructive',
      })
    },
  })

  // 重設密碼表單內容
  const resetPasswordForm = () => {
    setCurrentPassword('')
    setNewPassword('')
    setConfirmPassword('')
  }

  // 客戶端驗證並觸發修改密碼 API
  const handleChangePassword = () => {
    if (!currentPassword || !newPassword || !confirmPassword) {
      toast({ title: t('common.error'), description: t('password.fillAllFields'), variant: 'destructive' })
      return
    }
    if (newPassword.length < 6) {
      toast({ title: t('common.error'), description: t('password.minLength'), variant: 'destructive' })
      return
    }
    if (newPassword !== confirmPassword) {
      toast({ title: t('common.error'), description: t('password.mismatch'), variant: 'destructive' })
      return
    }
    changePasswordMutation.mutate({
      current_password: currentPassword,
      new_password: newPassword,
    })
  }

  // 切換側邊欄子選單展開/縮合狀態
  const toggleExpand = (title: string) => {
    setExpandedItems((prev) =>
      prev.includes(title)
        ? prev.filter((item) => item !== title)
        : [...prev, title]
    )
  }

  // 判斷該路徑是否為目前啟動中的頁面
  const isActive = (href: string) => {
    if (href.includes('?')) {
      return location.pathname + location.search === href
    }
    return location.pathname === href
  }

  // 判斷該項目下的任何子項目是否處於啟動狀態 (用於縮合時高亮父圖示)
  const isChildActive = (item: NavItem): boolean => {
    return item.children?.some((child) => isActive(child.href)) || false
  }

  // 根據使用者權限過濾導覽列顯示項目
  const filteredNavItems = useMemo(() => {
    // 檢查是否為純審查角色（只有 REVIEWER, VET, IACUC_CHAIR，沒有其他管理角色）
    const isReviewerOrChairOnly = user?.roles?.every(r =>
      ['REVIEWER', 'VET', 'IACUC_CHAIR'].includes(r)
    ) && user?.roles?.some(r => ['REVIEWER', 'VET', 'IACUC_CHAIR'].includes(r))

    return sortedNavItems
      .filter((item) => {
        // 審查委員/獸醫/主席不顯示人員管理
        if (item.title === '人員管理' && isReviewerOrChairOnly) {
          return false
        }
        if (item.permission === 'erp') {
          const hasErpAccess = hasRole('admin') ||
            user?.roles.some(r => ['purchasing', 'approver', 'WAREHOUSE_MANAGER', 'EXPERIMENT_STAFF'].includes(r)) ||
            user?.permissions.some(p => p.startsWith('erp.'))
          return hasErpAccess
        }
        if (item.permission && !hasPermission(item.permission) && !hasRole(item.permission)) {
          return false
        }
        return true
      })
      .map((item) => {
        // 過濾子選單項目的權限
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
        // 如果有子選單但全部被過濾掉，則隱藏整個項目
        if (item.children && item.children.length === 0) {
          return false
        }
        return true
      })
      .map((item) => {
        // 動態注入待處理變更申請數量到「我的變更申請」項目
        if (item.title === '我的變更申請' && pendingAmendmentsCount) {
          return { ...item, badge: pendingAmendmentsCount }
        }
        return item
      })
  }, [sortedNavItems, hasRole, hasPermission, user, pendingAmendmentsCount])

  return (
    <div className="flex h-screen bg-slate-50">
      {/* 側邊欄容器 */}
      <aside
        className={cn(
          'fixed inset-y-0 left-0 z-50 flex flex-col bg-slate-900 text-white transition-all duration-300 lg:relative overflow-hidden',
          sidebarOpen ? 'w-64' : 'w-16' // 根據 sidebarOpen 狀態切換寬度 (64px 或 16px)
        )}
      >
        {/* Logo 區域 */}
        <div className={cn(
          "flex h-16 items-center border-b border-slate-700",
          sidebarOpen ? "justify-between px-4" : "justify-center px-2"
        )}>
          {sidebarOpen ? ( // 展開狀態：顯示完整 Logo 與文字
            <Link to="/" className="flex items-center space-x-2">
              <img src="/pigmodel-logo.png" alt="Logo" className="h-10 w-auto" />
              <span className="text-xl font-bold">ipig system</span>
            </Link>
          ) : ( // 縮合狀態：僅顯示圖示，點擊可打開側邊欄
            <button
              onClick={() => setSidebarOpen(true)}
              className="flex items-center justify-center hover:bg-slate-800 rounded-lg transition-colors"
              title={t('nav.expandSidebar')}
            >
              <img src="/pigmodel-logo.png" alt="Logo" className="h-8 w-auto" />
            </button>
          )}
          {sidebarOpen && ( // 展開狀態下顯示關閉按鈕 (適用於行動裝置)
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setSidebarOpen(!sidebarOpen)}
              className="text-slate-400 hover:text-white hover:bg-slate-800"
            >
              <X className="h-5 w-5" />
            </Button>
          )}
        </div>

        {/* 導覽選單清單 */}
        <nav className="flex-1 overflow-y-auto p-4">
          {/* 編輯模式提示 */}
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

          {/* 編輯排序按鈕 */}
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

        {/* 底部使用者資訊及切換按鍵區域 */}
        <div className="border-t border-slate-700 p-2">
          {sidebarOpen ? ( // 展開狀態：顯示頭像、姓名、角色及操作按鈕
            <div className="space-y-2 p-2">
              <div className="flex items-center space-x-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-blue-600 font-semibold">
                  {user?.display_name?.[0] || user?.email?.[0] || 'U'}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="truncate text-sm font-medium">{user?.display_name || user?.email}</p>
                  {hasRole('admin') && (
                    <p className="truncate text-xs text-slate-400">{user?.roles?.join(', ')}</p>
                  )}
                </div>
              </div>
              <div className="grid grid-cols-2 gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => navigate('/profile/settings')}
                  className="text-slate-400 hover:text-white hover:bg-slate-800 text-xs"
                >
                  <UserCircle className="h-4 w-4 mr-1" />
                  {t('profile.settings')}
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setShowPasswordDialog(true)}
                  className="text-slate-400 hover:text-white hover:bg-slate-800 text-xs"
                >
                  <Key className="h-4 w-4 mr-1" />
                  {t('common.changePassword')}
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={logout}
                  className="col-span-2 text-slate-400 hover:text-white hover:bg-slate-800 text-xs"
                >
                  <LogOut className="h-4 w-4 mr-1" />
                  {t('common.logout')}
                </Button>
              </div>
            </div>
          ) : ( // 縮合狀態：僅顯示頭像與展開按鈕
            <div className="flex flex-col items-center space-y-2">
              <div className="flex h-8 w-8 items-center justify-center rounded-full bg-blue-600 font-semibold text-sm">
                {user?.display_name?.[0] || user?.email?.[0] || 'U'}
              </div>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setSidebarOpen(true)}
                className="text-slate-400 hover:text-white hover:bg-slate-800"
              >
                <Menu className="h-5 w-5" />
              </Button>
            </div>
          )}
        </div>
      </aside>

      {/* 主要內容區域 */}
      <main className={cn(
        "flex-1 overflow-y-auto transition-all duration-300 relative",
        sidebarOpen ? 'ml-64 lg:ml-0' : 'ml-16 lg:ml-0' // 桌機版 (lg) 設為 ml-0 避免重複間距
      )}>
        {/* 模擬登入提示 Banner */}
        {isImpersonating && (
          <div className="bg-blue-600 text-white px-4 py-2 flex items-center justify-between sticky top-0 z-[60] shadow-md">
            <div className="flex items-center gap-2">
              <UserCircle className="h-5 w-5" />
              <span className="text-sm font-medium">
                {t('common.impersonating')}：<span className="font-bold underline">{user?.display_name || user?.email}</span> ({user?.roles?.join(', ')})
              </span>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={stopImpersonating}
              className="bg-white/20 border-white text-white hover:bg-white hover:text-blue-600 h-8 font-semibold transition-all"
            >
              <ArrowLeft className="h-4 w-4 mr-1" />
              {t('common.backToAdmin')}
            </Button>
          </div>
        )}
        {/* 頂部導覽列 */}
        <header className="sticky top-0 z-40 flex h-16 items-center justify-end border-b bg-white px-4 shadow-sm">
          <div className="flex items-center space-x-4">
            {/* 顯示目前日期 */}
            <span className="text-sm text-muted-foreground">
              {new Date().toLocaleDateString(i18n.language, {
                weekday: 'long',
                year: 'numeric',
                month: 'long',
                day: 'numeric',
              })}
            </span>

            {/* 通知鈴鐺圖示與計數 */}
            <div className="relative" ref={notificationRef}>
              <Button
                variant="ghost"
                size="icon"
                className="relative"
                onClick={() => setShowNotificationDropdown(!showNotificationDropdown)}
              >
                <Bell className="h-5 w-5" />
                {unreadCount && unreadCount > 0 && ( // 若有未讀通知，顯示紅點計數
                  <span className="absolute -top-1 -right-1 h-5 w-5 flex items-center justify-center rounded-full bg-red-500 text-white text-xs font-bold">
                    {unreadCount > 99 ? '99+' : unreadCount}
                  </span>
                )}
              </Button>

              {/* 通知下拉選單內容 */}
              {showNotificationDropdown && (
                <div className="absolute right-0 top-12 w-96 bg-white rounded-lg shadow-xl border z-50 overflow-hidden">
                  {/* 選單標頭：標題與全部標為已讀按鈕 */}
                  <div className="flex items-center justify-between px-4 py-3 border-b bg-slate-50">
                    <h3 className="font-semibold text-slate-900">{t('common.notifications')}</h3>
                    {unreadCount && unreadCount > 0 && (
                      <button
                        onClick={() => markAllReadMutation.mutate()}
                        className="text-sm text-blue-600 hover:text-blue-800 flex items-center gap-1"
                        disabled={markAllReadMutation.isPending}
                      >
                        <CheckCheck className="h-4 w-4" />
                        {t('common.markAllRead')}
                      </button>
                    )}
                  </div>

                  {/* 通知列表捲動區域 */}
                  <div className="max-h-[400px] overflow-y-auto">
                    {notificationsData && notificationsData.length > 0 ? (
                      notificationsData.map((notification) => (
                        <div
                          key={notification.id}
                          onClick={() => handleNotificationClick(notification)}
                          className={cn(
                            "px-4 py-3 border-b last:border-b-0 cursor-pointer hover:bg-slate-50 transition-colors",
                            !notification.is_read && "bg-blue-50" // 未讀項目的背景色區隔
                          )}
                        >
                          <div className="flex items-start gap-3">
                            {/* 未讀藍點標記 */}
                            <div className={cn(
                              "w-2 h-2 rounded-full mt-2 shrink-0",
                              !notification.is_read ? "bg-blue-500" : "bg-transparent"
                            )} />
                            <div className="flex-1 min-w-0">
                              <p className={cn(
                                "text-sm truncate",
                                !notification.is_read ? "font-semibold text-slate-900" : "text-slate-700"
                              )}>
                                {notification.title}
                              </p>
                              {notification.content && (
                                <p className="text-sm text-slate-500 truncate mt-0.5">
                                  {notification.content}
                                </p>
                              )}
                              <p className="text-xs text-slate-400 mt-1">
                                {formatNotificationTime(notification.created_at)}
                              </p>
                            </div>
                            {/* 外部連結圖示 */}
                            {notification.related_entity_type && (
                              <ExternalLink className="h-4 w-4 text-slate-400 shrink-0 mt-1" />
                            )}
                          </div>
                        </div>
                      ))
                    ) : (
                      <div className="px-4 py-8 text-center text-slate-500">
                        <Bell className="h-8 w-8 mx-auto mb-2 text-slate-300" />
                        <p>{t('common.noNotifications')}</p>
                      </div>
                    )}
                  </div>

                  {/* 下拉選單底部連結 */}
                  {notificationsData && notificationsData.length > 0 && (
                    <div className="px-4 py-2 border-t bg-slate-50">
                      <button
                        onClick={() => {
                          setShowNotificationDropdown(false)
                          navigate('/admin/settings')
                        }}
                        className="text-sm text-blue-600 hover:text-blue-800 w-full text-center"
                      >
                        {t('common.viewAll')}
                      </button>
                    </div>
                  )}
                </div>
              )}
            </div>

            {/* 語言切換選擇器 */}
            <Select value={i18n.language} onValueChange={handleLanguageChange}>
              <SelectTrigger className="w-[120px] h-9">
                <Globe className="h-4 w-4 mr-2" />
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="zh-TW">{t('language.zhTW')}</SelectItem>
                <SelectItem value="en">{t('language.en')}</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </header>

        {/* 頁面內容渲染區域 (React Router Outlet) */}
        <div className="p-4">
          <Outlet />
        </div>
      </main>

      {/* 修改密碼彈窗 */}
      <Dialog open={showPasswordDialog} onOpenChange={(open) => {
        setShowPasswordDialog(open)
        if (!open) resetPasswordForm() // 關閉時清空表單
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Key className="h-5 w-5" />
              {t('password.title')}
            </DialogTitle>
            <DialogDescription>
              {t('password.description')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="current-password">{t('password.currentPassword')}</Label>
              <Input
                id="current-password"
                type="password"
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
                placeholder={t('password.currentPassword')}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="new-password">{t('password.newPassword')}</Label>
              <Input
                id="new-password"
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder={t('password.minLength')}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="confirm-password">{t('password.confirmPassword')}</Label>
              <Input
                id="confirm-password"
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder={t('password.confirmPassword')}
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setShowPasswordDialog(false)
                resetPasswordForm()
              }}
              disabled={changePasswordMutation.isPending}
            >
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleChangePassword}
              disabled={changePasswordMutation.isPending}
            >
              {changePasswordMutation.isPending && ( // 正在提交時顯示載入動畫
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              )}
              {t('password.submit')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
