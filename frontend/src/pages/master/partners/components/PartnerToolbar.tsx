import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { FilterBar } from '@/components/ui/filter-bar'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Plus, Upload, Download } from 'lucide-react'

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
      <PageHeader
        title="供應商/客戶管理"
        description="管理系統中的供應商與客戶資料"
        actions={
          <>
            <Button variant="outline" size="sm" onClick={onImport}>
              <Upload className="mr-2 h-4 w-4" />
              匯入
            </Button>
            <Button variant="outline" size="sm" onClick={onExport} disabled={!hasPartners}>
              <Download className="mr-2 h-4 w-4" />
              匯出
            </Button>
            <Button size="sm" onClick={onAdd}>
              <Plus className="mr-2 h-4 w-4" />
              新增夥伴
            </Button>
          </>
        }
      />

      <FilterBar
        search={search}
        onSearchChange={onSearchChange}
        searchPlaceholder="搜尋夥伴..."
      >
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
      </FilterBar>
    </>
  )
}
