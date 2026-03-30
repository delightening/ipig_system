import { Button } from '@/components/ui/button'
import { Loader2, Sparkles } from 'lucide-react'

import type { SkuPreviewTableProps } from './importTypes'

export function SkuPreviewTable({
  previewRows,
  skuOverrides,
  setSkuOverrides,
  rowCategoryCode,
  setRowCategoryCode,
  rowSubcategoryCode,
  setRowSubcategoryCode,
  skuCategories,
  subcategoriesByCategory,
  generateSkuIsPending,
  importIsPending,
  onGenerateSku,
  onConfirmImport,
  onBack,
  setCategorySubcategoryOverrides,
}: SkuPreviewTableProps) {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm font-medium text-foreground">
          請為以下商品設定 SKU（留空則由系統自動產生），共 {previewRows.length} 筆
        </p>
        <Button variant="ghost" size="sm" onClick={onBack}>
          返回
        </Button>
      </div>
      <p className="text-xs text-muted-foreground">
        先選擇品類與子類（與「新增產品」相同分層），再按「產生 SKU」由系統產出編碼；亦可手動輸入或留空由匯入時自動產生。
      </p>
      <div className="max-h-80 overflow-auto rounded-lg border border-border">
        <table className="w-full text-sm">
          <thead className="bg-muted sticky top-0">
            <tr>
              <th className="px-2 py-2 text-left font-medium w-10">列</th>
              <th className="px-2 py-2 text-left font-medium min-w-[80px]">名稱</th>
              <th className="px-2 py-2 text-left font-medium min-w-[60px]">規格</th>
              <th className="px-2 py-2 text-left font-medium w-12">單位</th>
              <th className="px-2 py-2 text-left font-medium w-16">安全庫存</th>
              <th className="px-2 py-2 text-left font-medium min-w-[100px]">品類</th>
              <th className="px-2 py-2 text-left font-medium min-w-[100px]">子類</th>
              <th className="px-2 py-2 text-left font-medium w-20">動作</th>
              <th className="px-2 py-2 text-left font-medium min-w-[110px]">SKU 編碼</th>
            </tr>
          </thead>
          <tbody>
            {previewRows.map((r) => {
              const catCode = rowCategoryCode[r.row] ?? ''
              const subCode = rowSubcategoryCode[r.row] ?? ''
              const subList = catCode ? subcategoriesByCategory[catCode] ?? [] : []
              const hasSubcategories = subList.length > 0
              const canGenerate = !!catCode && (hasSubcategories ? !!subCode : true)
              return (
                <tr key={r.row} className="border-t border-border hover:bg-muted">
                  <td className="px-2 py-1.5">{r.row}</td>
                  <td className="px-2 py-1.5">{r.name}</td>
                  <td className="px-2 py-1.5 text-muted-foreground">{r.spec ?? '-'}</td>
                  <td className="px-2 py-1.5">{r.base_uom}</td>
                  <td className="px-2 py-1.5">{r.safety_stock ?? '-'}</td>
                  <td className="px-2 py-1.5">
                    <select
                      aria-label={`第 ${r.row} 列品類`}
                      value={catCode}
                      onChange={(e) => {
                        const v = e.target.value
                        setRowCategoryCode((prev) => ({ ...prev, [r.row]: v }))
                        setRowSubcategoryCode((prev) => ({ ...prev, [r.row]: '' }))
                      }}
                      className="w-full min-w-[90px] rounded border border-border px-1.5 py-1 text-sm focus:border-status-info-solid focus:outline-none focus:ring-1 focus:ring-primary"
                    >
                      <option value="">—</option>
                      {skuCategories.map((c) => (
                        <option key={c.code} value={c.code}>
                          {c.name}
                        </option>
                      ))}
                    </select>
                  </td>
                  <td className="px-2 py-1.5">
                    {!catCode ? (
                      <span className="text-muted-foreground">—</span>
                    ) : hasSubcategories ? (
                      <select
                        aria-label={`第 ${r.row} 列子類`}
                        value={subCode}
                        onChange={(e) =>
                          setRowSubcategoryCode((prev) => ({
                            ...prev,
                            [r.row]: e.target.value,
                          }))
                        }
                        className="w-full min-w-[90px] rounded border border-border px-1.5 py-1 text-sm focus:border-status-info-solid focus:outline-none focus:ring-1 focus:ring-primary"
                      >
                        <option value="">—</option>
                        {subList.map((s) => (
                          <option key={s.code} value={s.code}>
                            {s.name}
                          </option>
                        ))}
                      </select>
                    ) : (
                      <span className="text-muted-foreground text-xs">同品類</span>
                    )}
                  </td>
                  <td className="px-2 py-1.5">
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      className="h-7 text-xs"
                      disabled={!canGenerate || generateSkuIsPending}
                      onClick={() => {
                        const sub = hasSubcategories ? subCode : catCode
                        onGenerateSku(r.row, catCode, sub, (sku) => {
                          setSkuOverrides((prev) => ({ ...prev, [r.row]: sku }))
                          setCategorySubcategoryOverrides((prev) => ({
                            ...prev,
                            [r.row]: {
                              category_code: catCode,
                              subcategory_code: sub,
                            },
                          }))
                        })
                      }}
                    >
                      {generateSkuIsPending ? (
                        <Loader2 className="h-3.5 w-3.5 animate-spin" />
                      ) : (
                        <>
                          <Sparkles className="h-3.5 w-3.5 mr-0.5" />
                          產生 SKU
                        </>
                      )}
                    </Button>
                  </td>
                  <td className="px-2 py-1.5">
                    <input
                      type="text"
                      value={skuOverrides[r.row] ?? ''}
                      onChange={(e) =>
                        setSkuOverrides((prev) => ({ ...prev, [r.row]: e.target.value }))
                      }
                      placeholder="留空自動產生"
                      className="w-full min-w-[100px] rounded border border-border px-2 py-1 text-sm font-mono focus:border-status-info-solid focus:outline-none focus:ring-1 focus:ring-primary"
                    />
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
      <div className="flex gap-2">
        <Button
          size="sm"
          onClick={onConfirmImport}
          disabled={importIsPending}
          className="bg-purple-600 hover:bg-purple-700"
        >
          {importIsPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
          確認匯入
        </Button>
        <Button variant="outline" size="sm" onClick={onBack}>
          返回
        </Button>
      </div>
    </div>
  )
}
