import { useTranslation } from 'react-i18next'

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

/**
 * 匯入檔的 CSV 欄位標題。⚠️ 這是後端（product_parser.rs）依欄名比對的**契約值**，
 * 必須維持中文，不可隨語系翻譯；說明文字只是把它們原樣帶進句子。
 */
const IMPORT_HEADERS = {
  sku: 'SKU編碼',
  name: '名稱',
  spec: '規格',
  category: '品類代碼',
  subcategory: '子類代碼',
  unit: '單位',
  trackBatch: '追蹤批號',
  trackExpiry: '追蹤效期',
  safetyStock: '安全庫存',
  remark: '備註',
} as const

export function ProductImportDialog({ open, onOpenChange }: Props) {
  const { t } = useTranslation()
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
      <DialogContent size={previewRows?.length ? 'xl' : 'md'}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Upload className="h-5 w-5" />
            {t('erpMaster.import.product.title')}
          </DialogTitle>
          <DialogDescription>
            {t('erpMaster.import.product.description')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Template Download */}
          <div className="flex items-center justify-between p-3 bg-status-info-bg rounded-lg">
            <div className="flex items-center gap-2">
              <FileSpreadsheet className="h-5 w-5 text-status-info-text" />
              <span className="text-sm text-status-info-text">{t('erpMaster.import.downloadTemplateHint')}</span>
            </div>
            <Button
              variant="outline"
              size="sm"
              className="border-primary text-status-info-text hover:bg-status-info-bg"
              onClick={() => downloadTemplateMutation.mutate()}
              disabled={downloadTemplateMutation.isPending}
            >
              <Download className="h-4 w-4 mr-1" />
              {t('erpMaster.import.downloadTemplate')}
            </Button>
          </div>

          {/* File Upload */}
          {!result && (
            <label className="block space-y-2">
              <span className="block text-sm font-medium leading-none">{t('erpMaster.import.selectFile')}</span>
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
              <p className="font-medium">{t('erpMaster.import.notesTitle')}</p>
              <ul className="list-disc list-inside space-y-0.5">
                <li>{t('erpMaster.import.product.noteName', { name: IMPORT_HEADERS.name })}</li>
                <li>{t('erpMaster.import.product.noteUnit', { unit: IMPORT_HEADERS.unit })}</li>
                <li>{t('erpMaster.import.product.noteCategory', { category: IMPORT_HEADERS.category, subcategory: IMPORT_HEADERS.subcategory })}</li>
                <li>{t('erpMaster.import.product.noteTrack', { trackBatch: IMPORT_HEADERS.trackBatch, trackExpiry: IMPORT_HEADERS.trackExpiry })}</li>
                <li>{t('erpMaster.import.product.noteColumns', { columns: Object.values(IMPORT_HEADERS).join(t('erpMaster.common.listSeparator')) })}</li>
              </ul>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={closeDialog}>
            {result ? t('common.closeDialog') : t('common.cancel')}
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
              {t('erpMaster.import.startImport')}
            </Button>
          )}
          {result && result.error_count === 0 && (
            <Button onClick={closeDialog} className="bg-status-success-solid hover:bg-green-700">
              {t('erpMaster.common.done')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
