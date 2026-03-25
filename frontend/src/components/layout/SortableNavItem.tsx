import { memo, useState } from 'react'
import { Link } from 'react-router-dom'
import { ChevronDown, GripVertical } from 'lucide-react'
import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'

import { cn } from '@/lib/utils'

import type { NavItem, SubsystemKey } from './sidebarNavConfig'

const subsystemActiveClass: Record<NonNullable<SubsystemKey>, string> = {
  aup: 'bg-subsystem-aup text-white',
  erp: 'bg-subsystem-erp text-white',
  animal: 'bg-subsystem-animal text-white',
  hr: 'bg-subsystem-hr text-white',
  admin: 'bg-subsystem-admin text-white',
}

function getActiveClass(subsystem?: SubsystemKey | null): string {
  if (subsystem && subsystemActiveClass[subsystem]) {
    return subsystemActiveClass[subsystem]
  }
  return 'bg-primary text-white'
}

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
  const activeClass = getActiveClass(item.subsystem)

  return (
    <ul className="ml-4 mt-1 space-y-1 border-l border-slate-700 pl-4">
      {item.children.map((child) =>
        child.children ? (
          <NestedGroup
            key={child.title}
            child={child}
            isActive={isActive}
            translateTitle={translateTitle}
            activeClass={activeClass}
          />
        ) : (
          <li key={child.href}>
            <Link
              to={child.href!}
              className={cn(
                'block rounded-lg px-3 py-2 text-sm transition-colors',
                isActive(child.href!)
                  ? activeClass
                  : 'text-slate-400 hover:bg-slate-800 hover:text-white'
              )}
            >
              {translateTitle(child)}
            </Link>
          </li>
        )
      )}
    </ul>
  )
})

function NestedGroup({
  child,
  isActive,
  translateTitle,
  activeClass,
}: {
  child: { title: string; translate?: boolean; children?: { title: string; href?: string; translate?: boolean }[] }
  isActive: (href: string) => boolean
  translateTitle: (item: { title: string; translate?: boolean }) => string
  activeClass: string
}) {
  const hasActiveChild = child.children?.some(c => c.href && isActive(c.href)) ?? false
  const [expanded, setExpanded] = useState(hasActiveChild)

  return (
    <li>
      <button
        onClick={() => setExpanded(prev => !prev)}
        className={cn(
          'flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm transition-colors',
          hasActiveChild
            ? 'text-white'
            : 'text-slate-400 hover:bg-slate-800 hover:text-white'
        )}
      >
        <span>{translateTitle(child)}</span>
        <ChevronDown
          className={cn(
            'h-3.5 w-3.5 shrink-0 transition-transform',
            expanded && 'rotate-180'
          )}
        />
      </button>
      {expanded && child.children && (
        <ul className="ml-3 mt-1 space-y-1 border-l border-slate-700 pl-3">
          {child.children.map(sub => (
            <li key={sub.href}>
              <Link
                to={sub.href!}
                className={cn(
                  'block rounded-lg px-3 py-1.5 text-xs transition-colors',
                  isActive(sub.href!)
                    ? activeClass
                    : 'text-slate-400 hover:bg-slate-800 hover:text-white'
                )}
              >
                {translateTitle(sub)}
              </Link>
            </li>
          ))}
        </ul>
      )}
    </li>
  )
}

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

  const activeClass = getActiveClass(item.subsystem)

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
                ? activeClass
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
              ? activeClass
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
