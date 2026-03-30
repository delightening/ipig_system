import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Loader2, Upload, Download, FileSpreadsheet } from 'lucide-react'

import { SkuPreviewTable } from './SkuPreviewTable'
import { DuplicateWarning } from './DuplicateWarning'
import { ImportResultSummary } from './ImportResultSummary'
import { NoSkuColumnPrompt } from './NoSkuColumnPrompt'
import { useProductImport } from './useProductImport'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function ProductImportDialog({ open, onOpenChange }: Props) {
  const {
    file, result, checkResult, previewRows, skuOverrides, setSkuOverrides,
    rowCategoryCode, setRowCategoryCode, rowSubcategoryCode, setRowSubcategoryCode,
    skuCategories, subcategoriesByCategory,
    setCategorySubcategoryOverrides, setUserAcceptedNoSku,
    generateSkuMutation, checkMutation, previewMutation, importMutation,
    showNoSkuPrompt, showDuplicateWarning,
    handleFileInputChange, handleImport, handleClose, handleConfirmImportWithSku,
    downloadTemplateMutation, resetPreviewState, doImport,
  } = useProductImport(open)

  const closeDialog = () => handleClose(onOpenChange)

  return (
    <Dialog open={open} onOpenChange={closeDialog}>
      <DialogContent className={previewRows?.length ? 'max-w-4xl' : 'max-w-lg'}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Upload className="h-5 w-5" />
            匯入產品
          </DialogTitle>
          <DialogDescription>
            支援 Excel (.xlsx, .xls) 或 CSV 格式，批次匯入多筆產品資料
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Template Download */}
          <div className="flex items-center justify-between p-3 bg-status-info-bg rounded-lg">
            <div className="flex items-center gap-2">
              <FileSpreadsheet className="h-5 w-5 text-status-info-text" />
              <span className="text-sm text-status-info-text">下載範本檔案</span>
            </div>
            <Button
              variant="outline"
              size="sm"
              className="border-primary text-status-info-text hover:bg-status-info-bg"
              onClick={() => downloadTemplateMutation.mutate()}
              disabled={downloadTemplateMutation.isPending}
            >
              <Download className="h-4 w-4 mr-1" />
              下載範本 (XLSX)
            </Button>
          </div>

          {/* File Upload */}
          {!result && (
            <label className="block space-y-2">
              <span className="block text-sm font-medium leading-none">選擇檔案</span>
              <input
                type="file"
                accept=".xlsx,.xls,.csv"
                onChange={handleFileInputChange}
                className="block w-full text-sm text-muted-foreground
                  file:mr-4 file:py-2 file:px-4
                  file:rounded-lg file:border-0
                  file:text-sm file:font-semibold
                  file:bg-status-purple-bg file:text-status-purple-text
                  hover:file:bg-status-purple-bg
                  file:cursor-pointer"
              />
              {file && (
                <div className="mt-2 p-2 bg-muted rounded-lg">
                  <p className="text-sm font-medium">{file.name}</p>
                  <p className="text-xs text-muted-foreground">{(file.size / 1024).toFixed(1)} KB</p>
                </div>
              )}
            </label>
          )}

          {/* No SKU column prompt */}
          {showNoSkuPrompt && checkResult && (
            <NoSkuColumnPrompt
              previewMutationIsPending={previewMutation.isPending}
              importIsPending={importMutation.isPending}
              hasDuplicates={checkResult.duplicate_count > 0}
              onSetSkuManually={() => { if (file) previewMutation.mutate(file) }}
              onAutoGenerateSku={() => {
                if (checkResult.duplicate_count === 0 && file) {
                  doImport(file, false)
                } else {
                  setUserAcceptedNoSku(true)
                }
              }}
              onDownloadTemplate={() => {
                downloadTemplateMutation.mutate()
              }}
            />
          )}

          {/* SKU Preview Table */}
          {previewRows && previewRows.length > 0 && !result && (
            <SkuPreviewTable
              previewRows={previewRows}
              skuOverrides={skuOverrides}
              setSkuOverrides={setSkuOverrides}
              rowCategoryCode={rowCategoryCode}
              setRowCategoryCode={setRowCategoryCode}
              rowSubcategoryCode={rowSubcategoryCode}
              setRowSubcategoryCode={setRowSubcategoryCode}
              skuCategories={skuCategories}
              subcategoriesByCategory={subcategoriesByCategory}
              generateSkuIsPending={generateSkuMutation.isPending}
              importIsPending={importMutation.isPending}
              onGenerateSku={(row, category, subcategory, onSuccess) => {
                generateSkuMutation.mutate(
                  { category, subcategory },
                  { onSuccess: (data) => onSuccess(data.sku) }
                )
              }}
              onConfirmImport={handleConfirmImportWithSku}
              onBack={resetPreviewState}
              setCategorySubcategoryOverrides={setCategorySubcategoryOverrides}
            />
          )}

          {/* Duplicate Warning */}
          {showDuplicateWarning && checkResult && (
            <DuplicateWarning
              checkResult={checkResult}
              importIsPending={importMutation.isPending}
              onSkipDuplicates={() => { if (file) doImport(file, true, false) }}
              onImportWithNewSku={() => { if (file) doImport(file, false, true) }}
              onImportAnyway={() => { if (file) doImport(file, false, false) }}
            />
          )}

          {/* Import Result */}
          {result && <ImportResultSummary result={result} />}

          {/* Instructions */}
          {!result && (
            <div className="text-sm text-muted-foreground space-y-1">
              <p className="font-medium">注意事項：</p>
              <ul className="list-disc list-inside space-y-0.5">
                <li>名稱為必填欄位</li>
                <li>單位為必填欄位（預設 PCS）</li>
                <li>品類代碼、子類代碼可選，未填時預設為 GEN-OTH；於編輯頁變更分類後將自動產生新 SKU（僅 GEN-OTH 可改動 SKU）</li>
                <li>追蹤批號、追蹤效期：true/false 或 是/否</li>
                <li>CSV 欄位順序：SKU編碼、名稱、規格、品類代碼、子類代碼、單位、追蹤批號、追蹤效期、安全庫存、備註（SKU 可留空由系統自動產生）</li>
              </ul>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={closeDialog}>
            {result ? '關閉' : '取消'}
          </Button>
          {!result && !checkResult && !previewRows && (
            <Button
              onClick={handleImport}
              disabled={checkMutation.isPending || importMutation.isPending || !file}
              className="bg-purple-600 hover:bg-purple-700"
            >
              {(checkMutation.isPending || importMutation.isPending) && (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              )}
              開始匯入
            </Button>
          )}
          {result && result.error_count === 0 && (
            <Button onClick={closeDialog} className="bg-status-success-solid hover:bg-green-700">
              完成
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
