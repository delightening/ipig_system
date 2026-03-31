import { useCallback } from 'react'
import { useSearchParams } from 'react-router-dom'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { cn } from '@/lib/utils'

interface TabConfig {
  value: string
  label: string
  icon?: React.ComponentType<{ className?: string }>
  badge?: number
  /** 需要特定角色才顯示此 Tab */
  hidden?: boolean
}

interface PageTabsProps {
  /** Tab 定義陣列 */
  tabs: TabConfig[]
  /** URL search param 鍵名（預設 'tab'） */
  paramKey?: string
  /** 預設 Tab（不帶 param 時使用） */
  defaultTab?: string
  /** 子元素：TabsContent 或 render function */
  children: React.ReactNode
  className?: string
}

/**
 * URL 同步的 Tab 導航。
 * Tab 選擇自動同步到 URL search params（?tab=xxx），支援瀏覽器前進/後退。
 */
export function PageTabs({
  tabs,
  paramKey = 'tab',
  defaultTab,
  children,
  className,
}: PageTabsProps) {
  const [searchParams, setSearchParams] = useSearchParams()

  const resolvedDefault = defaultTab ?? tabs[0]?.value ?? ''
  const validValues = new Set(tabs.map((t) => t.value))
  const paramValue = searchParams.get(paramKey)
  const activeTab = paramValue && validValues.has(paramValue) ? paramValue : resolvedDefault

  const setActiveTab = useCallback(
    (value: string) => {
      setSearchParams((prev) => {
        const next = new URLSearchParams(prev)
        if (value === resolvedDefault) {
          next.delete(paramKey)
        } else {
          next.set(paramKey, value)
        }
        return next
      }, { replace: true })
    },
    [setSearchParams, paramKey, resolvedDefault]
  )

  const visibleTabs = tabs.filter((t) => !t.hidden)

  return (
    <Tabs value={activeTab} onValueChange={setActiveTab} className={cn('w-full', className)}>
      <TabsList>
        {visibleTabs.map((tab) => (
          <TabsTrigger key={tab.value} value={tab.value} className="gap-1.5">
            {tab.icon && <tab.icon className="h-4 w-4" />}
            {tab.label}
            {tab.badge !== undefined && tab.badge > 0 && (
              <span className="ml-1 inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary/10 px-1.5 text-xs font-medium text-primary">
                {tab.badge}
              </span>
            )}
          </TabsTrigger>
        ))}
      </TabsList>
      {children}
    </Tabs>
  )
}

// Re-export TabsContent for convenience
export { TabsContent as PageTabContent } from '@/components/ui/tabs'
