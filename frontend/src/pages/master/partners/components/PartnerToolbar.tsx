import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Plus, Search, Upload, Download } from 'lucide-react'

interface PartnerToolbarProps {
  search: string
  onSearchChange: (value: string) => void
  typeFilter: string
  onTypeFilterChange: (value: string) => void
  hasPartners: boolean
  onImport: () => void
  onExport: () => void
  onAdd: () => void
}

export function PartnerToolbar({
  search,
  onSearchChange,
  typeFilter,
  onTypeFilterChange,
  hasPartners,
  onImport,
  onExport,
  onAdd,
}: PartnerToolbarProps) {
  return (
    <>
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">供應商/客戶管理</h1>
          <p className="text-muted-foreground">管理系統中的供應商與客戶資料</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={onImport}>
            <Upload className="mr-2 h-4 w-4" />
            匯入
          </Button>
          <Button variant="outline" size="sm" onClick={onExport} disabled={!hasPartners}>
            <Download className="mr-2 h-4 w-4" />
            匯出
          </Button>
          <Button onClick={onAdd}>
            <Plus className="mr-2 h-4 w-4" />
            新增夥伴
          </Button>
        </div>
      </div>

      <div className="flex gap-4">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜尋夥伴..."
            aria-label="搜尋夥伴"
            value={search}
            onChange={(e) => onSearchChange(e.target.value)}
            className="pl-9"
          />
        </div>
        <Select value={typeFilter} onValueChange={onTypeFilterChange}>
          <SelectTrigger className="w-40">
            <SelectValue placeholder="全部類型" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">全部類型</SelectItem>
            <SelectItem value="supplier">供應商</SelectItem>
            <SelectItem value="customer">客戶</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </>
  )
}
