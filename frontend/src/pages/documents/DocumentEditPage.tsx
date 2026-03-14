import React from 'react'
import { useSearchParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
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
import type { DocType } from '@/lib/api'
import { DocumentFormHeader } from './components/DocumentFormHeader'
import { DocumentPreview } from './components/DocumentPreview'
import { DocumentLineEditor } from './components/DocumentLineEditor'
import { WarehouseShelfTreeSelect, type WarehouseShelfValue } from '@/components/inventory/WarehouseShelfTreeSelect'
import { useDocumentForm } from './hooks/useDocumentForm'
import { DOC_TYPE_NAMES } from './types'

export function DocumentEditPage() {
  const [searchParams] = useSearchParams()
  const defaultType = (searchParams.get('type') as DocType) || 'PO'

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
    warehouses,
    partners,
    activeProtocols,
    filteredPartners,
    needsPartner,
    isTransfer,
    partnerType,
    totalAmount,
    addLine,
    removeLine,
    selectProduct,
    openProductSearch,
    handleBatchChange,
    handleLineBlur,
    handleBack,
    handleIacucNoSelect,
    updateLineAmount,
    createOrFindCustomerMutation,
    saveMutation,
    submitMutation,
    setFormData,
    showIacucWarning,
    setShowIacucWarning,
    iacucWarningData,
    isIacucRequired,
    iacucDisabled,
    needsShelf: needsShelf,
    batchStorageLocationId,
    batchStorageLocationFromId,
    batchStorageLocationToId,
    handleBatchShelfSelect,
    handleBatchShelfSelectFrom,
    handleBatchShelfSelectTo,
    poReceiptStatus,
    source_doc_id: _ignored_source_doc_id,
    categoryId,
    setCategoryId,
  } = useDocumentForm({ defaultType })

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
    return (allDocuments as any[]).filter(d => d.partner_id === formData.partner_id)
  }, [allDocuments, formData.partner_id])

  const showTotalAmount = ['PO', 'GRN', 'DO'].includes(formData.doc_type)

  if (isEdit && loadingDocument) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <DocumentFormHeader
        isEdit={isEdit}
        docTypeName={DOC_TYPE_NAMES[formData.doc_type]}
        onBack={handleBack}
        onSave={() => saveMutation.mutate()}
        onSubmit={() => submitMutation.mutate()}
        isSaving={saveMutation.isPending}
        isSubmitting={submitMutation.isPending}
        hasLines={formData.lines.length > 0}
      />

      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>單據資訊</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>單據類型</Label>
                <Select
                  value={formData.doc_type}
                  onValueChange={(v) => updateField('doc_type', v as DocType)}
                  disabled={isEdit}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(DOC_TYPE_NAMES)
                      .filter(([key]) => !['RM', 'DO'].includes(key) || isEdit) // 新增時隱藏已棄用類型
                      .map(([key, name]) => (
                        <SelectItem key={key} value={key}>
                          {name}
                        </SelectItem>
                      ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>單據日期</Label>
                <Input
                  type="date"
                  value={formData.doc_date}
                  onChange={(e) => updateField('doc_date', e.target.value)}
                />
              </div>
            </div>

            {isTransfer ? (
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>來源倉庫 *</Label>
                  <WarehouseShelfTreeSelect
                    value={formData.warehouse_from_id ? `wh:${formData.warehouse_from_id}` : ''}
                    onValueChange={(v: WarehouseShelfValue) => {
                      const id = v.startsWith('wh:') ? v.slice(3) : ''
                      updateField('warehouse_from_id', id)
                    }}
                    selectLevel="warehouse"
                    allowAll={false}
                    className="w-full"
                    placeholder="選擇來源倉庫"
                  />
                </div>
                <div className="space-y-2">
                  <Label>目標倉庫 *</Label>
                  <WarehouseShelfTreeSelect
                    value={formData.warehouse_to_id ? `wh:${formData.warehouse_to_id}` : ''}
                    onValueChange={(v: WarehouseShelfValue) => {
                      const id = v.startsWith('wh:') ? v.slice(3) : ''
                      updateField('warehouse_to_id', id)
                    }}
                    selectLevel="warehouse"
                    allowAll={false}
                    className="w-full"
                    placeholder="選擇目標倉庫"
                  />
                </div>
                {formData.warehouse_from_id && (
                  <div className="space-y-2">
                    <Label>來源儲位/貨架 *</Label>
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
                      placeholder="選擇來源儲位"
                    />
                  </div>
                )}
                {formData.warehouse_to_id && (
                  <div className="space-y-2">
                    <Label>目標儲位/貨架 *</Label>
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
                      placeholder="選擇目標儲位"
                    />
                  </div>
                )}
              </div>
            ) : (
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>倉庫 *</Label>
                  <WarehouseShelfTreeSelect
                    value={formData.warehouse_id ? `wh:${formData.warehouse_id}` : ''}
                    onValueChange={(v: WarehouseShelfValue) => {
                      const id = v.startsWith('wh:') ? v.slice(3) : ''
                      updateField('warehouse_id', id)
                    }}
                    selectLevel="warehouse"
                    allowAll={false}
                    className="w-full"
                    placeholder="選擇倉庫"
                  />
                </div>
                {formData.warehouse_id && needsShelf && (
                  <div className="space-y-2">
                    <Label>儲位/貨架 *</Label>
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
                      placeholder="選擇儲位"
                    />
                  </div>
                )}
              </div>
            )}

            {needsPartner && !isIacucRequired && (
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>
                    {partnerType === 'supplier' ? '供應商 *' : '客戶 *'}
                  </Label>
                  <Select
                    value={formData.partner_id}
                    onValueChange={(v) => {
                      updateField('partner_id', v)
                      if (formData.doc_type === 'GRN') {
                        updateField('source_doc_id', '') // 切換供應商時重設來源單據
                      }
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue
                        placeholder={`選擇${partnerType === 'supplier' ? '供應商' : '客戶'}`}
                      />
                    </SelectTrigger>
                    <SelectContent>
                      {!filteredPartners ? (
                        <div className="flex items-center justify-center p-2 text-sm text-muted-foreground">
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          載入中...
                        </div>
                      ) : filteredPartners.length === 0 ? (
                        <div className="p-2 text-sm text-muted-foreground text-center">
                          無可用數據
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
                    <Label>來源採購單 *</Label>
                    <Select
                      value={formData.source_doc_id || ''}
                      onValueChange={(v) => updateField('source_doc_id', v)}
                      disabled={!formData.partner_id}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={formData.partner_id ? "選擇採購單" : "請先選擇供應商"} />
                      </SelectTrigger>
                      <SelectContent>
                        {availableSourcePos.length === 0 ? (
                          <div className="p-2 text-sm text-muted-foreground text-center">
                            無可用採購單
                          </div>
                        ) : (
                          availableSourcePos.map((doc: any) => (
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

            {needsPartner && (
              <div className="grid grid-cols-1 gap-4">

                {!iacucDisabled && (
                  <div className="space-y-2">
                    <Label>
                      {isIacucRequired ? 'IACUC No. *' : '專屬計畫 (選填)'}
                    </Label>
                    <Select
                      value={formData.protocol_no || ''}
                      onValueChange={handleIacucNoSelect}
                      disabled={createOrFindCustomerMutation.isPending}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="選擇IACUC No." />
                      </SelectTrigger>
                      <SelectContent>
                        {loadingProtocols ? (
                          <div className="flex items-center justify-center p-2 text-sm text-muted-foreground">
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            載入中...
                          </div>
                        ) : activeProtocols && activeProtocols.length > 0 ? (
                          <>
                            {!isIacucRequired && (
                              <SelectItem value="PUBLIC">
                                --- 公用 (無特定計畫) ---
                              </SelectItem>
                            )}
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
                            無可用計畫
                          </div>
                        )}
                      </SelectContent>
                    </Select>
                  </div>
                )}
              </div>
            )}

            <div className="space-y-2">
              <Label>備註</Label>
              <Input
                value={formData.remark}
                onChange={(e) => updateField('remark', e.target.value)}
                placeholder="輸入備註..."
              />
            </div>
          </CardContent>
        </Card>

        <DocumentPreview
          formData={formData}
          totalAmount={totalAmount}
          showTotalAmount={showTotalAmount}
        />
      </div>

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
        setFormData={setFormData}
        needsShelf={needsShelf}
        poReceiptStatus={poReceiptStatus}
        categoryId={categoryId}
        setCategoryId={setCategoryId}
      />

      <Dialog open={showUnsavedDialog} onOpenChange={setShowUnsavedDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-yellow-500" />
              尚有未儲存的變更
            </DialogTitle>
            <DialogDescription>
              您有尚未儲存的變更，確定要離開嗎？離開後變更將會遺失。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowUnsavedDialog(false)}>
              繼續編輯
            </Button>
            <Button variant="destructive" onClick={confirmNavigation}>
              放棄變更
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={showIacucWarning} onOpenChange={setShowIacucWarning}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-yellow-500" />
              專屬採購計畫不符警告
            </DialogTitle>
            <DialogDescription>
              此批次產品（批號：{iacucWarningData?.batch_no}）是專門為計畫{' '}
              <span className="font-bold text-primary">
                {iacucWarningData?.source_iacuc}
              </span>{' '}
              採購的。
              <br />
              <br />
              您目前選擇的銷貨計畫為{' '}
              <span className="font-bold text-destructive">
                {formData.partner_id ? partners?.find(p => p.id === formData.partner_id)?.code || formData.partner_id : '未指定'}
              </span>
              。確定要繼續使用此批次嗎？
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowIacucWarning(false)}>
              返回修改
            </Button>
            <Button onClick={() => setShowIacucWarning(false)}>
              我了解，繼續使用
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
