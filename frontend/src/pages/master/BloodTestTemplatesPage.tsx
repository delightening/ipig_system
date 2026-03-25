import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Plus, Search, ArrowLeft, Settings, Star } from 'lucide-react'
import { PanelIcon } from '@/components/ui/panel-icon'
import { useBloodTestTemplates } from './hooks/useBloodTestTemplates'
import { BloodTestTemplateTable } from './components/BloodTestTemplateTable'
import { BloodTestTemplateFormDialog } from './components/BloodTestTemplateFormDialog'
import { BloodTestPanelFormDialog } from './components/BloodTestPanelFormDialog'
import type { ShowFilter } from './hooks/useBloodTestTemplates'

export function BloodTestTemplatesPage() {
  const navigate = useNavigate()
  const m = useBloodTestTemplates()

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => navigate('/animals')}
          aria-label="返回"
        >
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <PageHeader
          title="血檢項目"
          description={`管理血檢項目模板（共 ${m.totalCount} 個，啟用 ${m.activeCount} 個）`}
          className="flex-1"
          actions={
            <div className="flex gap-2">
              <Button variant="outline" onClick={() => navigate('/blood-test-panels')}>
                <Settings className="mr-2 h-4 w-4" />
                管理分類
              </Button>
              <Button variant="outline" onClick={() => navigate('/blood-test-presets')}>
                <Star className="mr-2 h-4 w-4" />
                管理常用組合
              </Button>
              <Button
                onClick={() => {
                  m.resetForm()
                  m.setDialogOpen(true)
                }}
              >
                <Plus className="mr-2 h-4 w-4" />
                新增項目
              </Button>
            </div>
          }
        />
      </div>

      <div className="flex flex-wrap gap-1.5">
        <Button
          variant={m.selectedPanel === 'all' ? 'default' : 'outline'}
          size="sm"
          onClick={() => m.setSelectedPanel('all')}
          className="gap-1"
        >
          全部
        </Button>
        {m.panels?.map((p) => (
          <Button
            key={p.key}
            variant={m.selectedPanel === p.key ? 'default' : 'outline'}
            size="sm"
            onClick={() => m.setSelectedPanel(p.key)}
            className="gap-1"
          >
            <PanelIcon icon={p.icon} />
            {p.name}
            <Badge variant="secondary" className="ml-1 px-1.5 py-0 text-xs">
              {p.items.length}
            </Badge>
          </Button>
        ))}
      </div>

      <div className="flex gap-4 items-center">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜尋代碼或名稱..."
            value={m.search}
            onChange={(e) => m.setSearch(e.target.value)}
            className="pl-9"
          />
        </div>
        <div className="flex gap-1">
          {(['all', 'active', 'inactive'] as ShowFilter[]).map((f) => (
            <Button
              key={f}
              variant={m.showFilter === f ? 'default' : 'outline'}
              size="sm"
              onClick={() => m.setShowFilter(f)}
            >
              {f === 'all' ? '全部' : f === 'active' ? '啟用中' : '已停用'}
            </Button>
          ))}
        </div>
      </div>

      <BloodTestTemplateTable
        groupedData={m.groupedData}
        flatFiltered={m.flatFiltered}
        isLoading={m.isLoading}
        search={m.search}
        sortField={m.sortField}
        onSort={m.handleSort}
        onEdit={m.handleEdit}
        onToggle={(template) =>
          m.toggleMutation.mutate({
            id: template.id,
            is_active: !template.is_active,
          })
        }
      />

      <BloodTestTemplateFormDialog
        open={m.dialogOpen}
        onOpenChange={m.setDialogOpen}
        editingTemplate={m.editingTemplate}
        form={m.templateForm}
        panels={m.panels}
        isCreatePending={m.createMutation.isPending}
        isUpdatePending={m.updateMutation.isPending}
        onSubmit={m.handleSubmit}
      />

      <BloodTestPanelFormDialog
        open={m.panelDialogOpen}
        onOpenChange={m.setPanelDialogOpen}
        form={m.panelForm}
        isPending={m.createPanelMutation.isPending}
        onSubmit={m.panelForm.handleSubmit((data) => m.createPanelMutation.mutate(data))}
      />
    </div>
  )
}
