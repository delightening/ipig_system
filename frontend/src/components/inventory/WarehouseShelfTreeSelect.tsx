/**
 * 倉庫與貨架樹狀選單（兩層：倉庫 → 貨架）
 * 值格式：all | wh:{warehouse_id} | loc:{storage_location_id}
 */
import { useState } from 'react'
import { Popover, PopoverContent, PopoverTrigger } from '@radix-ui/react-popover'
import { ChevronDown, ChevronRight, FolderOpen } from 'lucide-react'
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
  className?: string
}

export function WarehouseShelfTreeSelect({
  value,
  onValueChange,
  className,
}: WarehouseShelfTreeSelectProps) {
  const [open, setOpen] = useState(false)
  const { data: tree, isLoading } = useQuery({
    queryKey: ['warehouses-with-shelves'],
    queryFn: async () => {
      const res = await api.get<WarehouseTreeNode[]>('/warehouses/with-shelves')
      return res.data
    },
  })

  const displayLabel = formatDisplayLabel(tree, value)
  const handleSelect = (v: WarehouseShelfValue) => {
    onValueChange(v)
    setOpen(false)
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          className={cn('w-48 justify-between font-normal', className)}
          disabled={isLoading}
        >
          <span className="truncate">{isLoading ? '載入中...' : displayLabel}</span>
          <ChevronDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-64 p-0" align="start">
        <div className="max-h-80 overflow-y-auto py-1">
          <button
            type="button"
            className={cn(
              'flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground',
              value === 'all' && 'bg-accent',
            )}
            onClick={() => handleSelect('all')}
          >
            <span>全部倉庫</span>
          </button>
          {tree?.map((wh) => (
            <div key={wh.id}>
              <button
                type="button"
                className={cn(
                  'flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground',
                  value === `wh:${wh.id}` && 'bg-accent',
                )}
                onClick={() => handleSelect(`wh:${wh.id}`)}
              >
                <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
                <FolderOpen className="h-4 w-4 shrink-0 text-muted-foreground" />
                <span className="truncate">{wh.name}</span>
              </button>
              {wh.shelves.map((shelf) => (
                <button
                  key={shelf.id}
                  type="button"
                  className={cn(
                    'flex w-full items-center gap-2 pl-10 pr-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground',
                    value === `loc:${shelf.id}` && 'bg-accent',
                  )}
                  onClick={() => handleSelect(`loc:${shelf.id}`)}
                >
                  <span className="truncate">{shelf.name || shelf.code}</span>
                </button>
              ))}
            </div>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  )
}
