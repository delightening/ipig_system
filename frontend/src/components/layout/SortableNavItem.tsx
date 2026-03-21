import { memo } from 'react'
import { Link } from 'react-router-dom'
import { ChevronDown, GripVertical } from 'lucide-react'
import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'

import { cn } from '@/lib/utils'

import type { NavItem } from './sidebarNavConfig'

export interface SortableNavItemProps {
  item: NavItem
  isEditMode: boolean
  sidebarOpen: boolean
  isActive: (href: string) => boolean
  isChildActive: (item: NavItem) => boolean
  expandedItems: string[]
  toggleExpand: (title: string) => void
  navigate: (path: string) => void
  translateTitle: (item: { title: string; translate?: boolean }) => string
}

function DragHandle(props: Record<string, unknown>) {
  return (
    <button
      {...props}
      className="p-1 mr-1 cursor-grab active:cursor-grabbing text-slate-500 hover:text-white"
    >
      <GripVertical className="h-4 w-4" />
    </button>
  )
}

const ChildrenList = memo(function ChildrenList({
  item,
  isActive,
  translateTitle,
}: {
  item: NavItem
  isActive: (href: string) => boolean
  translateTitle: (item: { title: string; translate?: boolean }) => string
}) {
  if (!item.children) return null

  return (
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
  )
})

export const SortableNavItem = memo(function SortableNavItem({
  item,
  isEditMode,
  sidebarOpen,
  isActive,
  isChildActive,
  expandedItems,
  toggleExpand,
  navigate,
  translateTitle,
}: SortableNavItemProps) {
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

  const showDragHandle = isEditMode && sidebarOpen

  if (item.href) {
    return (
      <li ref={setNodeRef} style={style}>
        <div className="flex items-center">
          {showDragHandle && <DragHandle {...attributes} {...listeners} />}
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
      </li>
    )
  }

  return (
    <li ref={setNodeRef} style={style}>
      <div className="flex items-center">
        {showDragHandle && <DragHandle {...attributes} {...listeners} />}
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
      {sidebarOpen && expandedItems.includes(item.title) && (
        <ChildrenList
          item={item}
          isActive={isActive}
          translateTitle={translateTitle}
        />
      )}
    </li>
  )
})
