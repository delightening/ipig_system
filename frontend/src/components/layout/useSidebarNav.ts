import { useMemo } from 'react'
import { useLocation } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { arrayMove } from '@dnd-kit/sortable'

import { useAuthStore } from '@/stores/auth'
import { STALE_TIME, shouldPoll } from '@/lib/query'
import api, { deleteResource } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'

import {
  DEFAULT_NAV_ORDER,
  GUEST_NAV_ORDER,
  navItemsConfig,
  CLIENT_ONLY_NAV_IDS,
  GUEST_HIDDEN_CHILD_IDS,
  LEGACY_TITLE_TO_ID,
} from './sidebarNavConfig'
import type { NavItem } from './sidebarNavConfig'

export function useSidebarNav() {
  const location = useLocation()
  const queryClient = useQueryClient()
  const { user, hasRole, hasPermission, isInitialized } = useAuthStore()
  const { t } = useTranslation()

  const translateTitle = (item: { title: string; translate?: boolean }) => {
    if (item.translate === false) return item.title
    return t(`nav.${item.title}`) || item.title
  }

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

  const isGuest = user?.roles?.includes('GUEST')

  const sortedNavItems = useMemo(() => {
    // 舊版 nav_order 可能存中文 title — 對齊到新 id 後再參與比較
    const rawOrder = isGuest ? GUEST_NAV_ORDER : (navOrderData || DEFAULT_NAV_ORDER)
    const order = rawOrder.map((s) => LEGACY_TITLE_TO_ID[s] ?? s)
    return [...navItemsConfig].sort((a, b) => {
      const posA = order.indexOf(a.id)
      const posB = order.indexOf(b.id)
      return (posA === -1 ? 999 : posA) - (posB === -1 ? 999 : posB)
    })
  }, [navOrderData, isGuest])

  const { data: pendingAmendmentsCount } = useQuery({
    queryKey: ['amendments-pending-count'],
    queryFn: async () => {
      const res = await api.get<{ count: number }>('/amendments/pending-count')
      return res.data.count
    },
    staleTime: STALE_TIME.LIST,
    refetchInterval: () => shouldPoll(60_000),
    enabled: !!user && isInitialized,
  })

  const filteredNavItems = useMemo(() => {
    // Guest：開放大部分業務功能 read-only + 部分系統管理 read-only（GLP 賣點）。
    // 寫入操作由 <GuestHide /> 與 handler-level guard 阻擋，sidebar 不過濾。
    if (isGuest) {
      return sortedNavItems
        .map(item => {
          if (!item.children) return item
          const filtered = item.children.filter(c => !c.id || !GUEST_HIDDEN_CHILD_IDS.has(c.id))
          return filtered.length > 0 ? { ...item, children: filtered } : null
        })
        .filter(Boolean) as NavItem[]
    }

    // R19-9: 客戶（僅有 PI 角色）只顯示「我的計劃書」
    const isClientOnly = user?.roles?.length === 1 && user.roles[0] === 'PI'
    if (isClientOnly) {
      return sortedNavItems.filter(item => CLIENT_ONLY_NAV_IDS.has(item.id))
    }

    const rolesWithoutHrAccess = ['REVIEWER', 'VET', 'IACUC_CHAIR', 'PI']
    const shouldHideHr = user?.roles?.every(r =>
      rolesWithoutHrAccess.includes(r)
    ) && user?.roles?.some(r => rolesWithoutHrAccess.includes(r))

    return sortedNavItems
      .filter((item) => {
        if (item.id === 'hr' && shouldHideHr) return false
        if (item.permission === 'erp') {
          return hasRole('admin') ||
            user?.permissions?.some(p => p.startsWith('erp.')) ||
            user?.permissions?.some(p => p.startsWith('equipment.'))
        }
        if (item.permission && !hasPermission(item.permission) && !hasRole(item.permission)) {
          return false
        }
        return true
      })
      .map((item) => {
        if (!item.children) return item
        const filteredChildren = item.children
          .filter(child => {
            if (child.permission) return hasPermission(child.permission) || hasRole('admin')
            return true
          })
          .map(child => {
            if (!child.children) return child
            const filteredSubs = child.children.filter(sub => {
              if (sub.permission) return hasPermission(sub.permission) || hasRole('admin')
              return true
            })
            return filteredSubs.length > 0 ? { ...child, children: filteredSubs } : null
          })
          .filter(Boolean) as typeof item.children
        return { ...item, children: filteredChildren }
      })
      .filter((item) => !item.children || item.children.length > 0)
      .map((item) => {
        // myAmendments 為子項，未來若改為頂層 nav 可在此加 badge — 用 id 而非 title 比對
        if (item.id === 'myAmendments' && pendingAmendmentsCount) {
          return { ...item, badge: pendingAmendmentsCount }
        }
        return item
      })
  }, [sortedNavItems, hasRole, hasPermission, user, pendingAmendmentsCount, isGuest])

  const isActive = (href: string) => {
    if (href.includes('?')) {
      return location.pathname + location.search === href
    }
    return location.pathname === href
  }

  const isChildActive = (item: NavItem): boolean => {
    return item.children?.some((child) => {
      if (child.href && isActive(child.href)) return true
      return child.children?.some(sub => sub.href && isActive(sub.href)) ?? false
    }) ?? false
  }

  const handleDragEnd = (activeId: string, overId: string) => {
    const oldIndex = sortedNavItems.findIndex((item) => item.id === activeId)
    const newIndex = sortedNavItems.findIndex((item) => item.id === overId)
    const newOrder = arrayMove(sortedNavItems.map(i => i.id), oldIndex, newIndex)
    saveNavOrderMutation.mutate(newOrder)
  }

  return {
    filteredNavItems,
    isActive,
    isChildActive,
    translateTitle,
    handleDragEnd,
    handleResetNavOrder: () => resetNavOrderMutation.mutate(),
    isResettingNavOrder: resetNavOrderMutation.isPending,
  }
}
