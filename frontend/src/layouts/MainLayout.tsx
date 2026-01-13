import { useState, useEffect, useRef } from 'react' // 引入 React 核心 Hook：狀態、副作用、引用
import { Outlet, Link, useLocation, useNavigate } from 'react-router-dom' // 引入路由組件與導覽 Hook
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query' // 引入資料獲取與變更管理工具
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
  ClipboardList,
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
} from 'lucide-react'

// 定義導覽選單項目的介面規格
interface NavItem {
  title: string // 顯示名稱
  href?: string // 跳轉連結（如果有）
  icon: React.ReactNode // 顯示圖示
  children?: { title: string; href: string }[] // 子選單（選填）
  permission?: string // 需要的權限代碼（選填）
}

// 靜態定義側邊導覽列的所有項目
const navItems: NavItem[] = [
  {
    title: '儀表板',
    href: '/dashboard',
    icon: <LayoutDashboard className="h-5 w-5" />,
    permission: 'erp',
  },
  // AUP 審查系統模組
  {
    title: '我的計劃',
    href: '/my-projects',
    icon: <FolderOpen className="h-5 w-5" />,
  },
  {
    title: 'AUP 審查系統',
    icon: <FileText className="h-5 w-5" />,
    children: [
      { title: '計畫書管理', href: '/protocols' },
      { title: '新增計畫書', href: '/protocols/new' },
    ],
  },
  // 實驗動物管理系統模組
  {
    title: '實驗動物管理',
    icon: <Stethoscope className="h-5 w-5" />,
    children: [
      { title: '動物列表', href: '/pigs' },
      { title: '來源管理', href: '/pig-sources' },
    ],
  },
  // iPig ERP：基礎資料
  {
    title: '基礎資料',
    icon: <Package className="h-5 w-5" />,
    children: [
      { title: '產品管理', href: '/products' },
      { title: '倉庫管理', href: '/warehouses' },
      { title: '供應商/客戶', href: '/partners' },
    ],
    permission: 'erp',
  },
  // iPig ERP：採購管理
  {
    title: '採購管理',
    icon: <Truck className="h-5 w-5" />,
    children: [
      { title: '採購單', href: '/documents?type=PO' },
      { title: '採購入庫', href: '/documents?type=GRN' },
      { title: '採購退貨', href: '/documents?type=PR' },
    ],
    permission: 'erp',
  },
  // iPig ERP：銷售管理
  {
    title: '銷售管理',
    icon: <ShoppingCart className="h-5 w-5" />,
    children: [
      { title: '銷售單', href: '/documents?type=SO' },
      { title: '銷售出庫', href: '/documents?type=DO' },
      { title: '銷售退貨', href: '/documents?type=SR' },
    ],
    permission: 'erp',
  },
  // iPig ERP：倉儲作業
  {
    title: '倉儲作業',
    icon: <Warehouse className="h-5 w-5" />,
    children: [
      { title: '庫存查詢', href: '/inventory' },
      { title: '庫存流水', href: '/inventory/ledger' },
      { title: '調撥單', href: '/documents?type=TR' },
      { title: '盤點單', href: '/documents?type=STK' },
      { title: '調整單', href: '/documents?type=ADJ' },
    ],
    permission: 'erp',
  },
  // 報表中心模組
  {
    title: '報表中心',
    icon: <BarChart3 className="h-5 w-5" />,
    children: [
      { title: '庫存現況報表', href: '/reports/stock-on-hand' },
      { title: '庫存流水報表', href: '/reports/stock-ledger' },
      { title: '採購明細報表', href: '/reports/purchase-lines' },
      { title: '銷售明細報表', href: '/reports/sales-lines' },
      { title: '成本摘要報表', href: '/reports/cost-summary' },
    ],
    permission: 'erp',
  },
  // 系統管理模組（權限限制為 admin）
  {
    title: '系統管理',
    icon: <Settings className="h-5 w-5" />,
    children: [
      { title: '使用者管理', href: '/admin/users' },
      { title: '角色權限', href: '/admin/roles' },
      { title: '系統設定', href: '/admin/settings' },
      { title: '審計日誌', href: '/admin/audit-logs' },
    ],
    permission: 'admin',
  },
]

export function MainLayout() {
  const location = useLocation() // 取得當前 URL 路徑資訊
  const navigate = useNavigate() // 用於程式化導覽跳轉
  const queryClient = useQueryClient() // TanStack Query 快取管理器
  const { user, logout, hasRole, hasPermission } = useAuthStore() // 從 Auth Store 取得用戶資訊、登出方法與權限檢查
  const [sidebarOpen, setSidebarOpen] = useState(true) // 控制側邊欄展開/縮小的狀態
  const [expandedItems, setExpandedItems] = useState<string[]>(['基礎資料', '倉儲作業']) // 控制側邊欄摺疊選單展開項目的清單
  const [language, setLanguage] = useState<string>('zh-TW') // 當前系統語言介面狀態

  // 通知下拉選單的顯示狀態與引用
  const [showNotificationDropdown, setShowNotificationDropdown] = useState(false)
  const notificationRef = useRef<HTMLDivElement>(null)

  // 修改密碼對話框的相關狀態
  const [showPasswordDialog, setShowPasswordDialog] = useState(false)
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')

  // 取得未讀通知數量的 API 查詢
  const { data: unreadCount } = useQuery({
    queryKey: ['notifications-unread-count'],
    queryFn: async () => {
      const res = await api.get<{ count: number }>('/notifications/unread-count')
      return res.data.count
    },
    refetchInterval: 60000, // 每 60 秒自動重新獲取最新數量
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
      toast({ title: '成功', description: '已標記所有通知為已讀' })
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

    if (minutes < 1) return '剛剛'
    if (minutes < 60) return `${minutes} 分鐘前`
    if (hours < 24) return `${hours} 小時前`
    if (days < 7) return `${days} 天前`
    return date.toLocaleDateString('zh-TW')
  }

  // 修改密碼的 API 變更操作
  const changePasswordMutation = useMutation({
    mutationFn: async (data: ChangeOwnPasswordRequest) => {
      return api.put('/me/password', data)
    },
    onSuccess: () => {
      toast({ title: '成功', description: '密碼已修改，請使用新密碼重新登入' })
      setShowPasswordDialog(false)
      resetPasswordForm()
      logout() // 改完密碼強制登出，要求使用者重新驗證
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '密碼修改失敗',
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
      toast({ title: '錯誤', description: '請填寫所有欄位', variant: 'destructive' })
      return
    }
    if (newPassword.length < 6) {
      toast({ title: '錯誤', description: '新密碼至少需要 6 個字元', variant: 'destructive' })
      return
    }
    if (newPassword !== confirmPassword) {
      toast({ title: '錯誤', description: '新密碼與確認密碼不一致', variant: 'destructive' })
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
  const isChildActive = (item: NavItem) => {
    return item.children?.some((child) => isActive(child.href))
  }

  // 根據使用者權限過濾導覽列顯示項目
  const filteredNavItems = navItems.filter((item) => {
    if (item.permission === 'erp') {
      const hasErpAccess = hasRole('admin') ||
        user?.roles.some(r => ['warehouse', 'purchasing', 'sales', 'approver'].includes(r)) ||
        user?.permissions.some(p => p.startsWith('erp.'))
      return hasErpAccess
    }
    if (item.permission && !hasPermission(item.permission) && !hasRole(item.permission)) {
      return false
    }
    return true
  })

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
              title="展開側邊欄"
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
          <ul className="space-y-1">
            {filteredNavItems.map((item) => ( // 遍歷過濾後的導覽項目
              <li key={item.title}>
                {item.href ? ( // 情況 A：直接連結項目
                  <Link
                    to={item.href}
                    title={!sidebarOpen ? item.title : undefined} // 縮合時顯示提示文字
                    className={cn(
                      'flex items-center rounded-lg px-3 py-2.5 transition-colors',
                      sidebarOpen ? 'space-x-3' : 'justify-center',
                      isActive(item.href) // 如果是當前頁面，使用藍色高亮
                        ? 'bg-blue-600 text-white'
                        : 'text-slate-300 hover:bg-slate-800 hover:text-white'
                    )}
                  >
                    <span className="shrink-0">{item.icon}</span>
                    {sidebarOpen && <span>{item.title}</span>}
                  </Link>
                ) : ( // 情況 B：包含子選單的折疊項目
                  <>
                    <button
                      onClick={() => {
                        // 縮合狀態且有子選單時，點擊直接進入第一個子頁面
                        if (!sidebarOpen && item.children?.[0]?.href) {
                          navigate(item.children[0].href)
                        } else {
                          toggleExpand(item.title)
                        }
                      }}
                      title={!sidebarOpen ? item.title : undefined} // 縮合時顯示提示文字
                      className={cn(
                        'flex w-full items-center rounded-lg px-3 py-2.5 transition-colors',
                        sidebarOpen ? 'justify-between' : 'justify-center',
                        // 縮合時若子頁面活動，圖示顯示藍色高亮
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
                        {sidebarOpen && <span>{item.title}</span>}
                      </div>
                      {sidebarOpen && ( // 顯示箭頭圖示
                        <ChevronDown
                          className={cn(
                            'h-4 w-4 transition-transform',
                            expandedItems.includes(item.title) && 'rotate-180' // 展開時旋轉 180 度
                          )}
                        />
                      )}
                    </button>
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
                              {child.title}
                            </Link>
                          </li>
                        ))}
                      </ul>
                    )}
                  </>
                )}
              </li>
            ))}
          </ul>
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
                  <p className="truncate text-xs text-slate-400">{user?.roles?.join(', ')}</p>
                </div>
              </div>
              <div className="flex gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setShowPasswordDialog(true)}
                  className="flex-1 text-slate-400 hover:text-white hover:bg-slate-800 text-xs"
                >
                  <Key className="h-4 w-4 mr-1" />
                  修改密碼
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={logout}
                  className="flex-1 text-slate-400 hover:text-white hover:bg-slate-800 text-xs"
                >
                  <LogOut className="h-4 w-4 mr-1" />
                  登出
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
        "flex-1 overflow-y-auto transition-all duration-300",
        sidebarOpen ? 'ml-64 lg:ml-0' : 'ml-16 lg:ml-0' // 桌機版 (lg) 設為 ml-0 避免重複間距
      )}>
        {/* 頂部導覽列 */}
        <header className="sticky top-0 z-40 flex h-16 items-center justify-end border-b bg-white px-4 shadow-sm">
          <div className="flex items-center space-x-4">
            {/* 顯示目前日期 */}
            <span className="text-sm text-muted-foreground">
              {new Date().toLocaleDateString('zh-TW', {
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
                    <h3 className="font-semibold text-slate-900">通知</h3>
                    {unreadCount && unreadCount > 0 && (
                      <button
                        onClick={() => markAllReadMutation.mutate()}
                        className="text-sm text-blue-600 hover:text-blue-800 flex items-center gap-1"
                        disabled={markAllReadMutation.isPending}
                      >
                        <CheckCheck className="h-4 w-4" />
                        全部標為已讀
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
                        <p>沒有通知</p>
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
                        查看所有通知
                      </button>
                    </div>
                  )}
                </div>
              )}
            </div>

            {/* 語言切換選擇器 */}
            <Select value={language} onValueChange={setLanguage}>
              <SelectTrigger className="w-[120px] h-9">
                <Globe className="h-4 w-4 mr-2" />
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="zh-TW">繁體中文</SelectItem>
                <SelectItem value="zh-CN">简体中文</SelectItem>
                <SelectItem value="en">English</SelectItem>
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
              修改密碼
            </DialogTitle>
            <DialogDescription>
              請輸入目前密碼和新密碼。修改後需要重新登入。
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="current-password">目前密碼</Label>
              <Input
                id="current-password"
                type="password"
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
                placeholder="請輸入目前密碼"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="new-password">新密碼</Label>
              <Input
                id="new-password"
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                placeholder="至少 6 個字元"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="confirm-password">確認新密碼</Label>
              <Input
                id="confirm-password"
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                placeholder="再次輸入新密碼"
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
              取消
            </Button>
            <Button
              onClick={handleChangePassword}
              disabled={changePasswordMutation.isPending}
            >
              {changePasswordMutation.isPending && ( // 正在提交時顯示載入動畫
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              )}
              確認修改
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
