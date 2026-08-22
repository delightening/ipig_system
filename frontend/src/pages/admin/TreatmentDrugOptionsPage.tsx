/**
 * 藥物選單管理頁 — 後台管理介面
 *
 * 功能：
 * - CRUD 藥物選項
 * - 搜尋/篩選（關鍵字、分類、啟用狀態）
 * - 從 ERP 匯入藥物
 */

import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Plus, Upload } from 'lucide-react'

import { useDrugOptions } from './TreatmentDrugOptions/hooks/useDrugOptions'
import { DrugFilterBar } from './TreatmentDrugOptions/components/DrugFilterBar'
import { DrugTable } from './TreatmentDrugOptions/components/DrugTable'
import { DrugFormDialog } from './TreatmentDrugOptions/components/DrugFormDialog'
import { ErpImportDialog } from './TreatmentDrugOptions/components/ErpImportDialog'

export function TreatmentDrugOptionsPage() {
    const {
        keyword,
        setKeyword,
        filterCategory,
        setFilterCategory,
        filterActive,
        setFilterActive,
        drugs,
        isLoading,
        dialogs,
        setEditingDrug,
        form,
        setForm,
        resetForm,
        toggleUnit,
        openEditDialog,
        handleCreate,
        handleUpdate,
        handleToggleActive,
        handleDelete,
        createMutation,
        updateMutation,
    } = useDrugOptions()

    return (
        <div className="space-y-6">
            <PageHeader
                title="藥物選單管理"
                description="管理治療方式用藥的下拉選單選項"
                actions={
                    <div className="flex gap-2">
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={() => dialogs.open('import')}
                        >
                            <Upload className="h-4 w-4 mr-2" /> 從 ERP 匯入
                        </Button>
                        <Button size="sm" onClick={() => { resetForm(); dialogs.open('create') }}>
                            <Plus className="h-4 w-4 mr-2" /> 新增藥物
                        </Button>
                    </div>
                }
            />

            {/* 篩選列 */}
            <DrugFilterBar
                keyword={keyword}
                onKeywordChange={setKeyword}
                filterCategory={filterCategory}
                onCategoryChange={setFilterCategory}
                filterActive={filterActive}
                onActiveChange={setFilterActive}
            />

            {/* 藥物列表 */}
            <DrugTable
                drugs={drugs}
                isLoading={isLoading}
                onEdit={openEditDialog}
                onToggleActive={handleToggleActive}
                onDelete={handleDelete}
            />

            {/* 新增 / 編輯 Dialog */}
            <DrugFormDialog
                open={dialogs.isOpen('create')}
                onOpenChange={dialogs.setOpen('create')}
                title="新增藥物選項"
                form={form}
                setForm={setForm}
                onSubmit={handleCreate}
                isLoading={createMutation.isPending}
                toggleUnit={toggleUnit}
            />
            <DrugFormDialog
                open={dialogs.isOpen('edit')}
                onOpenChange={(open) => { dialogs.setOpen('edit')(open); if (!open) setEditingDrug(null) }}
                title="編輯藥物選項"
                form={form}
                setForm={setForm}
                onSubmit={handleUpdate}
                isLoading={updateMutation.isPending}
                toggleUnit={toggleUnit}
            />

            {/* ERP 匯入 Dialog */}
            <ErpImportDialog
                open={dialogs.isOpen('import')}
                onOpenChange={dialogs.setOpen('import')}
            />
        </div>
    )
}
