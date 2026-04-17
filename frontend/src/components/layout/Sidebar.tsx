import { useState, useCallback, startTransition, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
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
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'

import { useAuthStore } from '@/stores/auth'
import { cn } from '@/lib/utils'

import { SortableNavItem } from './SortableNavItem'
import { SidebarUserPanel } from './SidebarUserPanel'
import { SidebarNavEditControls } from './SidebarNavEditControls'
import { useSidebarNav } from './useSidebarNav'

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
  const navigate = useNavigate()
  const { user, hasRole, logout } = useAuthStore()
  const { t } = useTranslation()

  const [expandedItems, setExpandedItems] = useState<string[]>([])
  const [isEditMode, setIsEditMode] = useState(false)
  const touchStartX = useRef<number>(0)

  const handleTouchStart = useCallback((e: React.TouchEvent) => {
    touchStartX.current = e.touches[0].clientX
  }, [])

  const handleTouchEnd = useCallback((e: React.TouchEvent) => {
    const delta = e.changedTouches[0].clientX - touchStartX.current
    if (delta < -48 && window.innerWidth < 768) {
      setMobileSidebarOpen(false)
    }
  }, [setMobileSidebarOpen])

  const {
    filteredNavItems,
    isActive,
    isChildActive,
    translateTitle,
    handleDragEnd,
    handleResetNavOrder,
    isResettingNavOrder,
  } = useSidebarNav()

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  )

  const toggleExpand = (title: string) => {
    setExpandedItems((prev) =>
      prev.includes(title)
        ? prev.filter((item) => item !== title)
        : [...prev, title]
    )
  }

  const transitionNavigate = useCallback((path: string) => {
    startTransition(() => {
      navigate(path)
    })
  }, [navigate])

  const onDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    if (over && active.id !== over.id) {
      handleDragEnd(String(active.id), String(over.id))
    }
  }

  const handleToggleSidebar = () => {
    if (window.innerWidth < 768) {
      setMobileSidebarOpen(false)
    } else {
      setSidebarOpen(!sidebarOpen)
    }
  }

  return (
    <>
      {mobileSidebarOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/50 md:hidden"
          onClick={() => setMobileSidebarOpen(false)}
        />
      )}

      <aside
        style={{ contain: 'layout style' }}
        onTouchStart={handleTouchStart}
        onTouchEnd={handleTouchEnd}
        className={cn(
          'fixed inset-y-0 left-0 z-50 flex flex-col bg-slate-900 text-white transition-[width,transform] duration-300 ease-in-out overflow-hidden',
          mobileSidebarOpen ? 'translate-x-0' : '-translate-x-full',
          'md:translate-x-0 md:relative',
          sidebarOpen ? 'w-52' : 'md:w-16 w-52'
        )}
      >
        <div className="flex h-16 items-center border-b border-slate-700">
          <button
            onClick={handleToggleSidebar}
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
            <div className="mb-3 p-2 bg-primary/20 rounded-lg text-xs text-primary-foreground/80 text-center">
              {t('nav.editModeHint')}
            </div>
          )}
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragEnd={onDragEnd}
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
                    navigate={transitionNavigate}
                    translateTitle={translateTitle}
                  />
                ))}
              </ul>
            </SortableContext>
          </DndContext>

          {sidebarOpen && (
            <SidebarNavEditControls
              isEditMode={isEditMode}
              setIsEditMode={setIsEditMode}
              onResetNavOrder={handleResetNavOrder}
              isResetting={isResettingNavOrder}
            />
          )}
        </nav>

        <SidebarUserPanel
          sidebarOpen={sidebarOpen}
          setSidebarOpen={setSidebarOpen}
          user={user}
          isAdmin={hasRole('admin')}
          onChangePassword={onChangePassword}
          onLogout={logout}
          onNavigateProfile={() => transitionNavigate('/profile/settings')}
        />
      </aside>
    </>
  )
}
