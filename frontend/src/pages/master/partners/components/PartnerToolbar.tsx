import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
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
  const { t } = useTranslation()

  return (
    <>
      <PageHeader
        title={t('erpMaster.partners.title')}
        description={t('erpMaster.partners.description')}
        actions={
          <>
            <Can permission={PERMISSIONS.ERP_PARTNER_CREATE}>
              <Button variant="outline" size="sm" onClick={onImport}>
                <Upload className="mr-2 h-4 w-4" />
                {t('erpMaster.common.import')}
              </Button>
            </Can>
            <Button variant="outline" size="sm" onClick={onExport} disabled={!hasPartners}>
              <Download className="mr-2 h-4 w-4" />
              {t('erpMaster.common.export')}
            </Button>
            <Can permission={PERMISSIONS.ERP_PARTNER_CREATE}>
              <Button size="sm" onClick={onAdd}>
                <Plus className="mr-2 h-4 w-4" />
                {t('erpMaster.partners.add')}
              </Button>
            </Can>
          </>
        }
      />

      <FilterBar
        search={search}
        onSearchChange={onSearchChange}
        searchPlaceholder={t('erpMaster.partners.searchPlaceholder')}
      >
        <Select value={typeFilter} onValueChange={onTypeFilterChange}>
          <SelectTrigger className="w-40">
            <SelectValue placeholder={t('erpMaster.partners.allTypes')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t('erpMaster.partners.allTypes')}</SelectItem>
            <SelectItem value="supplier">{t('erpMaster.partners.type.supplier')}</SelectItem>
            <SelectItem value="customer">{t('erpMaster.partners.type.customer')}</SelectItem>
          </SelectContent>
        </Select>
      </FilterBar>
    </>
  )
}
