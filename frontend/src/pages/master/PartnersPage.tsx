import { useQuery } from '@tanstack/react-query'

import api, { Partner } from '@/lib/api'
import { useListFilters } from '@/hooks/useListFilters'
import { useDialogSet } from '@/hooks/useDialogSet'
import { useDebounce } from '@/hooks/useDebounce'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { STALE_TIME } from '@/lib/query'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { PartnerImportDialog } from '@/components/partner/PartnerImportDialog'

import { PartnerToolbar } from './partners/components/PartnerToolbar'
import { PartnerTable } from './partners/components/PartnerTable'
import { PartnerFormDialog } from './partners/components/PartnerFormDialog'
import { usePartnerForm } from './partners/hooks/usePartnerForm'
import { usePartnerExport } from './partners/hooks/usePartnerExport'

export function PartnersPage() {
  const { dialogState, confirm } = useConfirmDialog()
  const listFilters = useListFilters<{ type: string }>({ initialFilters: { type: 'all' } })
  const debouncedSearch = useDebounce(listFilters.search, 400)
  const dialogs = useDialogSet(['form', 'import'] as const)

  const { data: partners, isLoading } = useQuery({
    queryKey: ['partners', debouncedSearch, listFilters.filters.type],
    staleTime: STALE_TIME.LIST,
    queryFn: async () => {
      let params = ''
      if (debouncedSearch) params += `keyword=${encodeURIComponent(debouncedSearch)}&`
      if (listFilters.filters.type && listFilters.filters.type !== 'all') {
        params += `partner_type=${listFilters.filters.type}&`
      }
      const response = await api.get<Partner[]>(`/partners?${params}`)
      return response.data
    },
  })

  const partnerForm = usePartnerForm(() => dialogs.close('form'))
  const { handleExportCSV } = usePartnerExport(partners)

  const handleAdd = () => {
    partnerForm.resetForm()
    dialogs.open('form')
  }

  const handleEdit = (partner: Partner) => {
    partnerForm.handleEdit(partner, () => dialogs.open('form'))
  }

  const handleDelete = (partner: Partner, hard: boolean) => {
    partnerForm.deleteMutation.mutate({ id: partner.id, hard })
  }

  return (
    <div className="space-y-6">
      <PartnerToolbar
        search={listFilters.search}
        onSearchChange={listFilters.setSearch}
        typeFilter={listFilters.filters.type}
        onTypeFilterChange={(v) => listFilters.setFilter('type', v)}
        hasPartners={!!partners && partners.length > 0}
        onImport={() => dialogs.open('import')}
        onExport={handleExportCSV}
        onAdd={handleAdd}
      />

      <PartnerTable
        partners={partners}
        isLoading={isLoading}
        onEdit={handleEdit}
        onDelete={handleDelete}
        confirm={confirm}
      />

      <PartnerFormDialog
        open={dialogs.isOpen('form')}
        onOpenChange={dialogs.setOpen('form')}
        formData={partnerForm.formData}
        setFormData={partnerForm.setFormData}
        isEditing={!!partnerForm.editingPartner}
        isGeneratingCode={partnerForm.isGeneratingCode}
        isPending={partnerForm.isPending}
        onPartnerTypeChange={partnerForm.handlePartnerTypeChange}
        onSupplierCategoryChange={partnerForm.handleSupplierCategoryChange}
        onSubmit={partnerForm.handleSubmit}
        onClose={() => dialogs.close('form')}
      />

      <ConfirmDialog state={dialogState} />
      <PartnerImportDialog
        open={dialogs.isOpen('import')}
        onOpenChange={dialogs.setOpen('import')}
      />
    </div>
  )
}
