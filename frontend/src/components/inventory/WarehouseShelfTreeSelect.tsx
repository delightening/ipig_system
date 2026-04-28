/**
 * 倉庫與貨架樹狀選單（兩層：倉庫 → 貨架）
 * 值格式：all | wh:{warehouse_id} | loc:{storage_location_id}
 */
import { useState } from 'react'
import * as Popover from '@radix-ui/react-popover'
import { ChevronDown, FolderOpen, Warehouse, LayoutGrid, Check } from 'lucide-react'
import { useQuery } from '@tanstack/react-query'
import api, { WarehouseTreeNode } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

export type WarehouseShelfValue = 'all' | `wh:${string}` | `loc:${string}`

function parseValue(value: string): { type: 'all' | 'wh' | 'loc'; id?: string } {
  if (!value || value === 'all') return { type: 'all' }
  if (value.startsWith('wh:')) return { type: 'wh', id: value.slice(3) }
  if (value.startsWith('loc:')) return { type: 'loc', id: value.slice(4) }
  return { type: 'all' }
}

function formatDisplayLabel(
  tree: WarehouseTreeNode[] | undefined,
  value: string,
): string {
  const parsed = parseValue(value)
  if (parsed.type === 'all') return '全部倉庫'
  if (!tree || !parsed.id) return '全部倉庫'

  if (parsed.type === 'wh') {
    const wh = tree.find((w) => w.id === parsed.id)
    return wh ? wh.name : '全部倉庫'
  }

  if (parsed.type === 'loc') {
    for (const wh of tree) {
      const shelf = wh.shelves.find((s) => s.id === parsed.id)
      if (shelf) return `${wh.name} - ${shelf.name || shelf.code}`
    }
  }
  return '全部倉庫'
}

interface WarehouseShelfTreeSelectProps {
  value: string
  onValueChange: (value: WarehouseShelfValue) => void
  /**
   * 選擇層級：
   * - shelf: 預設，可選到具體貨架 (wh:id 或 loc:id)
   * - warehouse: 僅能選到倉庫 (僅 wh:id)
   */
  selectLevel?: 'warehouse' | 'shelf'
  /** 是否允許「全部」選項 */
  allowAll?: boolean
  className?: string
  placeholder?: string
  /** 父級倉庫 ID，若提供則僅顯示該倉庫下的儲位（且隱藏該倉庫按鈕內容，僅留儲位列表） */
  parentId?: string
}

export function WarehouseShelfTreeSelect({
  value,
  onValueChange,
  selectLevel = 'shelf',
  allowAll = true,
  className,
  placeholder,
  parentId,
}: WarehouseShelfTreeSelectProps) {
  const [open, setOpen] = useState(false)
  const { data: tree, isLoading } = useQuery({
    queryKey: ['warehouses-with-shelves'],
    queryFn: async () => {
      const res = await api.get<WarehouseTreeNode[]>('/warehouses/with-shelves')
      return res.data
    },
    staleTime: 5 * 60 * 1000, // 5 min
  })

  const displayLabel = formatDisplayLabel(tree, value)
  const handleSelect = (v: WarehouseShelfValue) => {
    onValueChange(v)
    setOpen(false)
  }

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <Popover.Trigger asChild>
        <Button
          variant="outline"
          role="combobox"
          className={cn(
            'w-64 justify-between font-normal transition-all hover:bg-accent/50',
            open && 'ring-2 ring-primary/20 border-primary/50',
            className
          )}
          disabled={isLoading}
        >
          <div className="flex items-center gap-2 overflow-hidden">
            <Warehouse className="h-4 w-4 shrink-0 text-muted-foreground" />
            <span className="truncate">
              {isLoading ? '載入中...' : (displayLabel === '全部倉庫' && placeholder) ? placeholder : displayLabel}
            </span>
          </div>
          <ChevronDown className={cn(
            "h-4 w-4 shrink-0 opacity-50 transition-transform duration-200",
            open && "rotate-180"
          )} />
        </Button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          className={cn(
            "z-[100] w-72 overflow-hidden rounded-xl border bg-popover/95 p-1 text-popover-foreground shadow-2xl outline-hidden backdrop-blur-sm",
            "animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2"
          )}
          align="start"
          sideOffset={4}
        >
          <div className="max-h-[400px] overflow-y-auto scrollbar-thin scrollbar-thumb-muted-foreground/20">
            {allowAll && !parentId && (
              <>
                <button
                  type="button"
                  className={cn(
                    'flex w-full items-center justify-between gap-2 px-3 py-2.5 text-sm font-medium rounded-md transition-colors hover:bg-accent hover:text-accent-foreground',
                    value === 'all' ? 'bg-accent text-accent-foreground' : 'text-foreground/70'
                  )}
                  onClick={() => handleSelect('all')}
                >
                  <div className="flex items-center gap-2">
                    <Warehouse className="h-4 w-4" />
                    <span>全部倉庫</span>
                  </div>
                  {value === 'all' && <Check className="h-4 w-4" />}
                </button>
                <div className="my-1 h-px bg-muted" />
              </>
            )}
            {tree?.filter(wh => !parentId || wh.id === parentId).map((wh) => (
              <div key={wh.id} className="space-y-0.5">
                {!parentId && (
                  <button
                    type="button"
                    className={cn(
                      'flex w-full items-center justify-between gap-2 px-3 py-2 text-sm font-medium rounded-md transition-colors hover:bg-accent/50 hover:text-accent-foreground',
                      value === `wh:${wh.id}` ? 'bg-accent text-accent-foreground' : 'text-foreground/80'
                    )}
                    onClick={() => handleSelect(`wh:${wh.id}`)}
                  >
                    <div className="flex items-center gap-2 overflow-hidden">
                      <div className="flex h-5 w-5 items-center justify-center rounded-sm bg-primary/10 text-primary">
                        <FolderOpen className="h-3.5 w-3.5" />
                      </div>
                      <span className="truncate">{wh.name}</span>
                    </div>
                    {value === `wh:${wh.id}` && <Check className="h-4 w-4" />}
                  </button>
                )}
                  {selectLevel === 'shelf' && (
                    <div className={cn(
                      "space-y-1 my-1",
                      !parentId && "pl-4 border-l border-muted/50 ml-5"
                    )}>
                      {wh.shelves.map((shelf) => (
                        <button
                          key={shelf.id}
                          type="button"
                          className={cn(
                            'flex w-full items-center justify-between gap-2 px-3 py-2 text-xs rounded-lg transition-all',
                            'hover:bg-primary/5 hover:text-primary active:scale-[0.98]',
                            value === `loc:${shelf.id}` ? 'bg-primary/10 text-primary font-semibold shadow-xs' : 'text-muted-foreground'
                          )}
                          onClick={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            handleSelect(`loc:${shelf.id}`);
                          }}
                        >
                          <div className="flex items-center gap-2 overflow-hidden">
                            <LayoutGrid className={cn(
                              "h-3.5 w-3.5 shrink-0 transition-opacity",
                              value === `loc:${shelf.id}` ? "opacity-100" : "opacity-40"
                            )} />
                            <span className="truncate">{shelf.name || shelf.code}</span>
                          </div>
                          {value === `loc:${shelf.id}` && <Check className="h-3.5 w-3.5" />}
                        </button>
                      ))}
                    </div>
                  )}
              </div>
            ))}
          </div>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  )
}
