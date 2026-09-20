import React from 'react'
import { useSearchParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { Trans, useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Loader2, AlertTriangle } from 'lucide-react'
import { Skeleton } from '@/components/ui/skeleton'
import type { DocType } from '@/lib/api'
import type { Document as ErpDocument } from '@/types/erp'
import { DocumentFormHeader } from './components/DocumentFormHeader'
import { DocumentPreview } from './components/DocumentPreview'
import { DocumentLineEditor } from './components/DocumentLineEditor'
import { WarehouseShelfTreeSelect, type WarehouseShelfValue } from '@/components/inventory/WarehouseShelfTreeSelect'
import { SearchableMultiSelect } from '@/components/ui/searchable-multi-select'
import { StocktakeExemptNotice } from '@/components/warehouse/StocktakeExemptNotice'
import { useSkuCategories } from '@/hooks/useSkuCategories'
import { useDocumentForm } from './hooks/useDocumentForm'
import { buildStocktakeScope, stocktakeBlockReason } from './stocktakeScope'
import { DOC_TYPE_NAMES } from './types'

export type AdjMode = 'add' | 'modify'

export function DocumentEditPage() {
  const { t } = useTranslation()
  const [searchParams] = useSearchParams()
  const defaultType = (searchParams.get('type') as DocType) || ''
  const [adjMode, setAdjMode] = React.useState<AdjMode>('modify')

  const {
    isEdit,
    formData,
    updateField,
    productSearchOpen,
    setProductSearchOpen,
    productSearch,
    setProductSearch,
    showUnsavedDialog,
    setShowUnsavedDialog,
    confirmNavigation,
    lineAmounts,
    inputRefs,
    loadingDocument,
    loadingProtocols,
    products,
    activeProtocols,
    filteredPartners,
    needsPartner,
    needsProtocol,
    isTransfer,
    totalAmount,
    addLine,
    removeLine,
    selectProduct,
    openProductSearch,
    handleBatchChange,
    handleLineBlur,
    handleBack,
    handleProtocolSelect,
    handleIacucNoSelect,
    updateLineAmount,
    updateLineField,
    saveMutation,
    submitMutation,
    setFormData,
    showIacucWarning,
    setShowIacucWarning,
    iacucWarningData,
    iacucDisabled,
    needsShelf: needsShelf,
    batchStorageLocationId,
    batchStorageLocationFromId,
    batchStorageLocationToId,
    handleBatchShelfSelect,
    handleBatchShelfSelectFrom,
    handleBatchShelfSelectTo,
    poReceiptStatus,
    categoryCode,
    setCategoryCode,
  } = useDocumentForm({ defaultType })

  // 盤點品類選單的資料源，與新增/編輯產品、產品清單篩選同一份（GET /sku/categories）。
  // 條件看 formData.doc_type 而非 URL 的 defaultType——單別在頁內可由下拉切換
  // （見下方 doc_type 的 Select），直接進 /documents/new 再選「盤點單」時 defaultType
  // 是空的，若照它判斷就會出現「欄位顯示得出來、品類一個都選不到」的空清單。
  const needsSkuCategories = formData.doc_type === 'STK' && !isEdit
  const {
    categories: skuCategories,
    categoriesLoading: skuCategoriesLoading,
    categoriesError: skuCategoriesError,
    refetchCategories: refetchSkuCategories,
  } = useSkuCategories({ enabled: needsSkuCategories })

  // 清單沒成功載入之前不准建單（理由見 stocktakeBlockReason 的說明）
  const stocktakeBlockedReason = stocktakeBlockReason({
    needed: needsSkuCategories,
    loading: skuCategoriesLoading,
    error: skuCategoriesError,
  })

  const { data: allDocuments } = useQuery({
    queryKey: ['documents', { doc_type: 'PO', status: 'approved' }],
    queryFn: async () => {
      const response = await api.get('/documents?doc_type=PO&status=approved')
      return response.data || []
    },
    enabled: formData.doc_type === 'GRN',
    staleTime: 60000,
  })

  const availableSourcePos = React.useMemo(() => {
    if (!allDocuments || !formData.partner_id) return []
    return (allDocuments as ErpDocument[]).filter((d) => d.partner_id === formData.partner_id)
  }, [allDocuments, formData.partner_id])

  const showTotalAmount = ['PO', 'GRN'].includes(formData.doc_type)

  if (isEdit && loadingDocument) {
    return <Skeleton variant="form" fields={6} />
  }

  return (
    <div className="space-y-6">
      <DocumentFormHeader
        isEdit={isEdit}
        docTypeName={formData.doc_type ? DOC_TYPE_NAMES[formData.doc_type] : ''}
        onBack={handleBack}
        onSave={() => saveMutation.mutate()}
        onSubmit={() => submitMutation.mutate()}
        isSaving={saveMutation.isPending}
        isSubmitting={submitMutation.isPending}
        hasLines={formData.lines.length > 0}
        blockedReason={stocktakeBlockedReason}
      />

      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>{t('erpDocs.shared.docInfo')}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>{t('erpDocs.shared.docType')}</Label>
                <Select
                  value={formData.doc_type || undefined}
                  onValueChange={(v) => updateField('doc_type', v as DocType)}
                  disabled={isEdit}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('erpDocs.documents.form.selectType')} />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(DOC_TYPE_NAMES)
                      .map(([key, name]) => (
                        <SelectItem key={key} value={key}>
                          {name}
                        </SelectItem>
                      ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>{t('erpDocs.shared.docDate')}</Label>
                <Input
                  type="date"
                  value={formData.doc_date}
                  onChange={(e) => updateField('doc_date', e.target.value)}
                  disabled={!formData.doc_type}
                />
              </div>
            </div>

            {formData.doc_type && (
              <>
            {isTransfer ? (
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>{t('erpDocs.documents.form.sourceWarehouseRequired')}</Label>
                  <WarehouseShelfTreeSelect
                    value={formData.warehouse_from_id ? `wh:${formData.warehouse_from_id}` : ''}
                    onValueChange={(v: WarehouseShelfValue) => {
                      const id = v.startsWith('wh:') ? v.slice(3) : ''
                      updateField('warehouse_from_id', id)
                    }}
                    selectLevel="warehouse"
                    allowAll={false}
                    className="w-full"
                    placeholder={t('erpDocs.documents.form.selectSourceWarehouse')}
                  />
                </div>
                <div className="space-y-2">
                  <Label>{t('erpDocs.documents.form.targetWarehouseRequired')}</Label>
                  <WarehouseShelfTreeSelect
                    value={formData.warehouse_to_id ? `wh:${formData.warehouse_to_id}` : ''}
                    onValueChange={(v: WarehouseShelfValue) => {
                      const id = v.startsWith('wh:') ? v.slice(3) : ''
                      updateField('warehouse_to_id', id)
                    }}
                    selectLevel="warehouse"
                    allowAll={false}
                    className="w-full"
                    placeholder={t('erpDocs.documents.form.selectTargetWarehouse')}
                  />
                </div>
                {formData.warehouse_from_id && (
                  <div className="space-y-2">
                    <Label>{t('erpDocs.documents.form.batchSourceLocation')}</Label>
                    <WarehouseShelfTreeSelect
                      value={batchStorageLocationFromId ? `loc:${batchStorageLocationFromId}` : ''}
                      onValueChange={(v: WarehouseShelfValue) => {
                        const shelfId = v.startsWith('loc:') ? v.slice(4) : ''
                        handleBatchShelfSelectFrom(shelfId)
                      }}
                      selectLevel="shelf"
                      parentId={formData.warehouse_from_id}
                      allowAll={false}
                      className="w-full"
                      placeholder={t('erpDocs.shared.selectSourceLocation')}
                    />
                  </div>
                )}
                {formData.warehouse_to_id && (
                  <div className="space-y-2">
                    <Label>{t('erpDocs.documents.form.batchTargetLocation')}</Label>
                    <WarehouseShelfTreeSelect
                      value={batchStorageLocationToId ? `loc:${batchStorageLocationToId}` : ''}
                      onValueChange={(v: WarehouseShelfValue) => {
                        const shelfId = v.startsWith('loc:') ? v.slice(4) : ''
                        handleBatchShelfSelectTo(shelfId)
                      }}
                      selectLevel="shelf"
                      parentId={formData.warehouse_to_id}
                      allowAll={false}
                      className="w-full"
                      placeholder={t('erpDocs.shared.selectTargetLocation')}
                    />
                  </div>
                )}
              </div>
            ) : (
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  {/* SO 跨倉（#1004）：表頭倉庫僅為品項搜尋/批次套用的預設過濾，非必填；
                      每行實際倉庫＝該行儲位所屬倉。其他單據仍必填。 */}
                  <Label>
                    {formData.doc_type === 'SO'
                      ? t('erpDocs.documents.form.defaultWarehouseOptional')
                      : t('erpDocs.documents.form.warehouseRequired')}
                  </Label>
                  <WarehouseShelfTreeSelect
                    value={formData.warehouse_id ? `wh:${formData.warehouse_id}` : ''}
                    onValueChange={(v: WarehouseShelfValue) => {
                      const id = v.startsWith('wh:') ? v.slice(3) : ''
                      updateField('warehouse_id', id)
                    }}
                    selectLevel="warehouse"
                    allowAll={formData.doc_type === 'SO'}
                    className="w-full"
                    placeholder={
                      formData.doc_type === 'SO'
                        ? t('erpDocs.documents.form.allWarehousesCross')
                        : t('erpDocs.shared.selectWarehouse')
                    }
                  />
                </div>
                {formData.warehouse_id && needsShelf && (
                  <div className="space-y-2">
                    <Label>{t('erpDocs.documents.form.batchLocation')}</Label>
                    <WarehouseShelfTreeSelect
                      value={batchStorageLocationId ? `loc:${batchStorageLocationId}` : ''}
                      onValueChange={(v: WarehouseShelfValue) => {
                        const shelfId = v.startsWith('loc:') ? v.slice(4) : ''
                        handleBatchShelfSelect(shelfId)
                      }}
                      selectLevel="shelf"
                      parentId={formData.warehouse_id}
                      allowAll={false}
                      className="w-full"
                      placeholder={t('erpDocs.shared.selectStorageLocation')}
                    />
                  </div>
                )}
                {/* 盤點單選到「不排例行盤點」的倉庫（如儲藏室）時說明一句。
                    只提示不阻擋——那些倉庫正是要在缺貨或異狀時單獨盤，
                    擋下來會把唯一的校正路徑一起封死。 */}
                {formData.doc_type === 'STK' && (
                  <StocktakeExemptNotice warehouseId={formData.warehouse_id} />
                )}
              </div>
            )}

            {/* 盤點單：限定品類（例：準備室只盤藥品，耗材不列入底稿）。
                只在**建立**時顯示——改單時明細已存在，後端不會重新產生底稿，
                此時給一個不生效的欄位只會誤導。 */}
            {needsSkuCategories && (
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>{t('erpDocs.documents.form.stocktakeCategories')}</Label>
                  {/* 載入中／失敗時一律停用：空清單與「沒有品類」在畫面上長得一樣，
                      讓人以為無從篩選而直接送出，就是一次非預期的全盤。 */}
                  <SearchableMultiSelect
                    options={skuCategories.map((c) => ({ value: c.code, label: `${c.name}（${c.code}）` }))}
                    value={formData.stocktake_scope?.category_codes ?? []}
                    onValueChange={(codes) =>
                      updateField('stocktake_scope', buildStocktakeScope(codes))
                    }
                    disabled={skuCategoriesLoading || skuCategoriesError}
                    placeholder={
                      skuCategoriesLoading
                        ? t('erpDocs.documents.form.categoriesLoading')
                        : skuCategoriesError
                          ? t('erpDocs.documents.form.categoriesFailed')
                          : t('erpDocs.documents.form.allCategories')
                    }
                    searchPlaceholder={t('erpDocs.documents.form.searchCategories')}
                    className="w-full"
                  />
                  {skuCategoriesError ? (
                    <div className="flex items-center gap-2">
                      <p className="text-xs text-destructive">
                        {t('erpDocs.documents.form.categoriesLoadFailedHint')}
                      </p>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        onClick={() => void refetchSkuCategories()}
                      >
                        {t('common.retry')}
                      </Button>
                    </div>
                  ) : (
                    <p className="text-xs text-muted-foreground">
                      {t('erpDocs.documents.form.categoriesHint')}
                    </p>
                  )}
                </div>
              </div>
            )}

            {/* 採購類：選擇供應商 */}
            {needsPartner && (
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>{t('erpDocs.documents.form.supplierRequired')}</Label>
                  <Select
                    value={formData.partner_id}
                    onValueChange={(v) => {
                      updateField('partner_id', v)
                      if (formData.doc_type === 'GRN') {
                        updateField('source_doc_id', '')
                      }
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder={t('erpDocs.documents.form.selectSupplier')} />
                    </SelectTrigger>
                    <SelectContent>
                      {!filteredPartners ? (
                        <div className="flex items-center justify-center p-2 text-sm text-muted-foreground">
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          {t('common.loading')}
                        </div>
                      ) : filteredPartners.length === 0 ? (
                        <div className="p-2 text-sm text-muted-foreground text-center">
                          {t('erpDocs.documents.form.noSuppliers')}
                        </div>
                      ) : (
                        filteredPartners.map((partner) => (
                          <SelectItem key={partner.id} value={partner.id}>
                            {partner.name}
                          </SelectItem>
                        ))
                      )}
                    </SelectContent>
                  </Select>
                </div>

                {formData.doc_type === 'GRN' && (
                  <div className="space-y-2">
                    <Label>{t('erpDocs.documents.form.sourcePoRequired')}</Label>
                    <Select
                      value={formData.source_doc_id || ''}
                      onValueChange={(v) => updateField('source_doc_id', v)}
                      disabled={!formData.partner_id}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={formData.partner_id ? t('erpDocs.documents.form.selectPo') : t('erpDocs.documents.form.selectSupplierFirst')} />
                      </SelectTrigger>
                      <SelectContent>
                        {availableSourcePos.length === 0 ? (
                          <div className="p-2 text-sm text-muted-foreground text-center">
                            {t('erpDocs.documents.form.noPos')}
                          </div>
                        ) : (
                          availableSourcePos.map((doc) => (
                            <SelectItem key={doc.id} value={doc.id}>
                              {doc.doc_no} ({doc.doc_date})
                            </SelectItem>
                          ))
                        )}
                      </SelectContent>
                    </Select>
                  </div>
                )}
              </div>
            )}

            {/* 銷貨類：直接選已核准計畫（計畫即客戶） */}
            {needsProtocol && (
              <div className="space-y-2">
                <Label>{t('erpDocs.documents.form.salesProtocolRequired')}</Label>
                <Select
                  value={formData.protocol_id || ''}
                  onValueChange={handleProtocolSelect}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('erpDocs.documents.form.selectSalesProtocol')} />
                  </SelectTrigger>
                  <SelectContent>
                    {loadingProtocols ? (
                      <div className="flex items-center justify-center p-2 text-sm text-muted-foreground">
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        {t('common.loading')}
                      </div>
                    ) : activeProtocols && activeProtocols.length > 0 ? (
                      activeProtocols.map((protocol) => (
                        <SelectItem key={protocol.id} value={protocol.id}>
                          {protocol.iacuc_no || protocol.protocol_no} - {protocol.title}
                        </SelectItem>
                      ))
                    ) : (
                      <div className="p-2 text-sm text-muted-foreground text-center">
                        {t('erpDocs.documents.form.noActiveProtocols')}
                      </div>
                    )}
                  </SelectContent>
                </Select>
              </div>
            )}

            {/* 採購類：選填 IACUC 費用歸屬計畫 */}
            {needsPartner && !iacucDisabled && (
              <div className="space-y-2">
                <Label>{t('erpDocs.documents.form.costProtocol')}</Label>
                <Select
                  value={formData.protocol_no || ''}
                  onValueChange={handleIacucNoSelect}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('erpDocs.documents.form.selectIacucNo')} />
                  </SelectTrigger>
                  <SelectContent>
                    {loadingProtocols ? (
                      <div className="flex items-center justify-center p-2 text-sm text-muted-foreground">
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        {t('common.loading')}
                      </div>
                    ) : activeProtocols && activeProtocols.length > 0 ? (
                      <>
                        <SelectItem value="PUBLIC">
                          {t('erpDocs.documents.form.publicOption')}
                        </SelectItem>
                        {activeProtocols.map((protocol) => (
                          <SelectItem
                            key={protocol.iacuc_no}
                            value={protocol.iacuc_no || ''}
                          >
                            {protocol.iacuc_no} - {protocol.title}
                          </SelectItem>
                        ))}
                      </>
                    ) : (
                      <div className="p-2 text-sm text-muted-foreground text-center">
                        {t('erpDocs.documents.form.noProtocols')}
                      </div>
                    )}
                  </SelectContent>
                </Select>
              </div>
            )}

            {formData.doc_type === 'ADJ' && (
              <div className="space-y-2">
                <Label>{t('erpDocs.documents.form.adjMode')}</Label>
                <Select value={adjMode} onValueChange={(v) => setAdjMode(v as AdjMode)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="modify">{t('erpDocs.documents.form.adjModify')}</SelectItem>
                    <SelectItem value="add">{t('erpDocs.documents.form.adjAdd')}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            )}

            <div className="space-y-2">
              <Label>{t('erpDocs.shared.remark')}</Label>
              <Input
                value={formData.remark}
                onChange={(e) => updateField('remark', e.target.value)}
                placeholder={t('erpDocs.documents.form.remarkPlaceholder')}
              />
            </div>
            </>
            )}
          </CardContent>
        </Card>

        <DocumentPreview
          formData={formData}
          totalAmount={totalAmount}
          showTotalAmount={showTotalAmount}
        />
      </div>

      {formData.doc_type && (
        <>
          <DocumentLineEditor
            formData={formData}
            lineAmounts={lineAmounts}
            inputRefs={inputRefs}
            productSearchOpen={productSearchOpen}
            setProductSearchOpen={setProductSearchOpen}
            productSearch={productSearch}
            setProductSearch={setProductSearch}
            products={products}
            addLine={addLine}
            removeLine={removeLine}
            selectProduct={selectProduct}
            openProductSearch={openProductSearch}
            handleBatchChange={handleBatchChange}
            handleLineBlur={handleLineBlur}
            updateLineAmount={updateLineAmount}
            updateLineField={updateLineField}
            setFormData={setFormData}
            needsShelf={needsShelf}
            poReceiptStatus={poReceiptStatus}
            categoryCode={categoryCode}
            setCategoryCode={setCategoryCode}
            adjMode={formData.doc_type === 'ADJ' ? adjMode : undefined}
            batchStorageLocationFromId={batchStorageLocationFromId}
            batchStorageLocationId={batchStorageLocationId}
          />
        </>
      )}

      <Dialog open={showUnsavedDialog} onOpenChange={setShowUnsavedDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-status-warning-solid" />
              {t('erpDocs.documents.form.unsaved.title')}
            </DialogTitle>
            <DialogDescription>
              {t('erpDocs.documents.form.unsaved.description')}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowUnsavedDialog(false)}>
              {t('erpDocs.documents.form.unsaved.keepEditing')}
            </Button>
            <Button variant="destructive" onClick={confirmNavigation}>
              {t('erpDocs.documents.form.unsaved.discard')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={showIacucWarning} onOpenChange={setShowIacucWarning}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-status-warning-solid" />
              {t('erpDocs.documents.form.iacucWarning.title')}
            </DialogTitle>
            <DialogDescription>
              <Trans
                i18nKey="erpDocs.documents.form.iacucWarning.description"
                values={{
                  batchNo: iacucWarningData?.batch_no ?? '',
                  sourceIacuc: iacucWarningData?.source_iacuc ?? '',
                  currentProtocol: formData.protocol_id
                    ? activeProtocols?.find((p) => p.id === formData.protocol_id)?.iacuc_no
                      || activeProtocols?.find((p) => p.id === formData.protocol_id)?.protocol_no
                      || formData.protocol_id
                    : t('erpDocs.shared.unspecified'),
                }}
                components={{
                  hl: <span className="font-bold text-primary" />,
                  cur: <span className="font-bold text-destructive" />,
                }}
              />
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowIacucWarning(false)}>
              {t('erpDocs.documents.form.iacucWarning.goBack')}
            </Button>
            <Button onClick={() => setShowIacucWarning(false)}>
              {t('erpDocs.documents.form.iacucWarning.continue')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
