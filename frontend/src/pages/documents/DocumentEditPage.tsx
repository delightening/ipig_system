import React from 'react'
import { useSearchParams } from 'react-router-dom'
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
  } = useDocumentForm({ defaultType })

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
                    {Object.entries(DOC_TYPE_NAMES).map(([key, name]) => (
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
                  <Select
                    value={formData.warehouse_from_id}
                    onValueChange={(v) => updateField('warehouse_from_id', v)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="選擇來源倉庫" />
                    </SelectTrigger>
                    <SelectContent>
                      {warehouses?.map((wh) => (
                        <SelectItem key={wh.id} value={wh.id}>
                          {wh.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label>目標倉庫 *</Label>
                  <Select
                    value={formData.warehouse_to_id}
                    onValueChange={(v) => updateField('warehouse_to_id', v)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="選擇目標倉庫" />
                    </SelectTrigger>
                    <SelectContent>
                      {warehouses?.map((wh) => (
                        <SelectItem key={wh.id} value={wh.id}>
                          {wh.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
            ) : (
              <div className="space-y-2">
                <Label>倉庫 *</Label>
                <Select
                  value={formData.warehouse_id}
                  onValueChange={(v) => updateField('warehouse_id', v)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="選擇倉庫" />
                  </SelectTrigger>
                  <SelectContent>
                    {warehouses?.map((wh) => (
                      <SelectItem key={wh.id} value={wh.id}>
                        {wh.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}

            {needsPartner && (
              <div className="space-y-2">
                {formData.doc_type === 'SO' || formData.doc_type === 'DO' ? (
                  <>
                    <Label>IACUC No. *</Label>
                    <Select
                      value={
                        formData.partner_id
                          ? partners?.find((p) => p.id === formData.partner_id)
                              ?.code ?? ''
                          : ''
                      }
                      onValueChange={handleIacucNoSelect}
                      disabled={createOrFindCustomerMutation.isPending}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="選擇IACUC No." />
                      </SelectTrigger>
                      <SelectContent>
                        {activeProtocols?.map((protocol) => (
                          <SelectItem
                            key={protocol.iacuc_no}
                            value={protocol.iacuc_no || ''}
                          >
                            {protocol.iacuc_no} - {protocol.title}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    {createOrFindCustomerMutation.isPending && (
                      <p className="text-xs text-muted-foreground flex items-center gap-1">
                        <Loader2 className="h-3 w-3 animate-spin" />
                        正在創建客戶...
                      </p>
                    )}
                  </>
                ) : (
                  <>
                    <Label>
                      {partnerType === 'supplier' ? '供應商' : '客戶'} *
                    </Label>
                    <Select
                      value={formData.partner_id}
                      onValueChange={(v) => updateField('partner_id', v)}
                    >
                      <SelectTrigger>
                        <SelectValue
                          placeholder={`選擇${partnerType === 'supplier' ? '供應商' : '客戶'}`}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        {filteredPartners?.map((partner) => (
                          <SelectItem key={partner.id} value={partner.id}>
                            {partner.name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </>
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
    </div>
  )
}
