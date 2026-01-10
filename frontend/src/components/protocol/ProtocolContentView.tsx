import { Label } from '@/components/ui/label'
import { formatDate } from '@/lib/utils'
import { FileText } from 'lucide-react'

interface ProtocolContentViewProps {
  workingContent: any
  protocolTitle: string
  startDate?: string
  endDate?: string
}

export function ProtocolContentView({ workingContent, protocolTitle, startDate, endDate }: ProtocolContentViewProps) {
  if (!workingContent) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        <FileText className="h-12 w-12 mx-auto mb-2" />
        <p>尚未填寫計畫內容</p>
      </div>
    )
  }

  const basic = workingContent.basic || {}
  const purpose = workingContent.purpose || {}
  const items = workingContent.items || {}
  const design = workingContent.design || {}

  return (
    <div className="protocol-pdf-view bg-white p-8 shadow-lg max-w-4xl mx-auto">
      {/* Header */}
      <div className="text-center mb-8 border-b pb-4">
        <h1 className="text-3xl font-bold mb-2">AUP 動物試驗計畫書</h1>
        <p className="text-lg text-muted-foreground">{protocolTitle}</p>
      </div>

      {/* 1. 研究資料 */}
      <section className="mb-8">
        <h2 className="text-2xl font-bold mb-4 border-b pb-2">1. 研究資料</h2>
        
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label className="text-sm font-semibold">GLP 屬性</Label>
              <p className="mt-1">{basic.is_glp ? '符合 GLP 規範' : '不符合 GLP 規範'}</p>
            </div>
            <div>
              <Label className="text-sm font-semibold">研究名稱</Label>
              <p className="mt-1">{protocolTitle || '-'}</p>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label className="text-sm font-semibold">計畫類型</Label>
              <p className="mt-1">{basic.project_type || '-'}</p>
            </div>
            <div>
              <Label className="text-sm font-semibold">計畫種類</Label>
              <p className="mt-1">
                {basic.project_category || '-'}
                {basic.project_category_other && ` (${basic.project_category_other})`}
              </p>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label className="text-sm font-semibold">預計試驗時程</Label>
              <p className="mt-1">
                {(startDate || basic.start_date) && (endDate || basic.end_date)
                  ? `${formatDate(startDate || basic.start_date)} ~ ${formatDate(endDate || basic.end_date)}`
                  : '-'}
              </p>
            </div>
          </div>

          {/* PI 資訊 */}
          {basic.pi && (
            <div className="border-t pt-4 mt-4">
              <h3 className="text-lg font-semibold mb-3">計畫主持人</h3>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label className="text-sm font-semibold">姓名</Label>
                  <p className="mt-1">{basic.pi.name || '-'}</p>
                </div>
                <div>
                  <Label className="text-sm font-semibold">電話</Label>
                  <p className="mt-1">{basic.pi.phone || '-'}</p>
                </div>
                <div>
                  <Label className="text-sm font-semibold">Email</Label>
                  <p className="mt-1">{basic.pi.email || '-'}</p>
                </div>
                <div>
                  <Label className="text-sm font-semibold">地址</Label>
                  <p className="mt-1">{basic.pi.address || '-'}</p>
                </div>
              </div>
            </div>
          )}

          {/* Sponsor 資訊 */}
          {basic.sponsor && (
            <div className="border-t pt-4 mt-4">
              <h3 className="text-lg font-semibold mb-3">委託單位</h3>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label className="text-sm font-semibold">單位名稱</Label>
                  <p className="mt-1">{basic.sponsor.name || '-'}</p>
                </div>
                <div>
                  <Label className="text-sm font-semibold">聯絡人</Label>
                  <p className="mt-1">{basic.sponsor.contact_person || '-'}</p>
                </div>
                <div>
                  <Label className="text-sm font-semibold">聯絡電話</Label>
                  <p className="mt-1">{basic.sponsor.contact_phone || '-'}</p>
                </div>
                <div>
                  <Label className="text-sm font-semibold">聯絡 Email</Label>
                  <p className="mt-1">{basic.sponsor.contact_email || '-'}</p>
                </div>
              </div>
            </div>
          )}

          {/* 試驗機構與設施 */}
          <div className="border-t pt-4 mt-4">
            <h3 className="text-lg font-semibold mb-3">試驗機構與設施</h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="text-sm font-semibold">機構名稱</Label>
                <p className="mt-1">{basic.facility?.title || '-'}</p>
              </div>
              <div>
                <Label className="text-sm font-semibold">位置</Label>
                <p className="mt-1">{basic.housing_location || '-'}</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* 2. 研究目的 */}
      <section className="mb-8 border-t pt-6">
        <h2 className="text-2xl font-bold mb-4 border-b pb-2">2. 研究目的</h2>
        
        {/* 2.1 研究之目的及重要性 */}
        {purpose.significance && (
          <div className="mb-4">
            <h3 className="text-lg font-semibold mb-2">2.1 研究之目的及重要性</h3>
            <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.significance}</p>
          </div>
        )}

        {/* 2.2 替代原則 */}
        {purpose.replacement && (
          <div className="mb-4">
            <h3 className="text-lg font-semibold mb-2">2.2 請以動物試驗應用3Rs之替代原則，說明本動物試驗之合理性</h3>
            
            {purpose.replacement.rationale && (
              <div className="mb-3">
                <h4 className="text-base font-medium mb-1">2.2.1 請說明活體動物試驗之必要性，以及選擇此動物種別的原因</h4>
                <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.replacement.rationale}</p>
              </div>
            )}

            {purpose.replacement.alt_search && (
              <div className="mb-3">
                <h4 className="text-base font-medium mb-1">2.2.2 請於下列網站搜尋非動物性替代方案</h4>
                {purpose.replacement.alt_search.platforms && purpose.replacement.alt_search.platforms.length > 0 && (
                  <ul className="list-disc list-inside text-sm mb-2">
                    {purpose.replacement.alt_search.platforms.map((p: string, idx: number) => (
                      <li key={idx}>{p}</li>
                    ))}
                  </ul>
                )}
                {purpose.replacement.alt_search.keywords && (
                  <p className="text-sm mb-2"><strong>搜尋關鍵字：</strong>{purpose.replacement.alt_search.keywords}</p>
                )}
                {purpose.replacement.alt_search.conclusion && (
                  <div>
                    <p className="text-sm font-medium mb-1">搜尋結果與結論：</p>
                    <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.replacement.alt_search.conclusion}</p>
                  </div>
                )}
              </div>
            )}

            {purpose.duplicate && (
              <div className="mb-3">
                <h4 className="text-base font-medium mb-1">2.2.3 是否為重複他人試驗</h4>
                <p className="text-sm mb-2">{purpose.duplicate.experiment ? '是' : '否'}</p>
                {purpose.duplicate.experiment && purpose.duplicate.justification && (
                  <div>
                    <p className="text-sm font-medium mb-1">請說明重複進行之科學理由：</p>
                    <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.duplicate.justification}</p>
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        {/* 2.3 減量原則 */}
        {purpose.reduction && purpose.reduction.design && (
          <div className="mb-4">
            <h3 className="text-lg font-semibold mb-2">2.3 請以實驗動物應用3Rs之減量原則，說明動物試驗設計，包括動物分組方法、訂定使用動物數量之理由等</h3>
            <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.reduction.design}</p>
          </div>
        )}
      </section>

      {/* 3. 試驗物質與對照物質 */}
      {items.use_test_item === true && (
        <section className="mb-8 border-t pt-6">
          <h2 className="text-2xl font-bold mb-4 border-b pb-2">3. 試驗物質與對照物質</h2>
          
          {/* 試驗物質 */}
          {items.test_items && items.test_items.length > 0 && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-3">試驗物質</h3>
              {items.test_items.map((item: any, index: number) => (
                <div key={index} className="mb-4 p-4 border rounded bg-slate-50">
                  <h4 className="font-medium mb-2">試驗物質 #{index + 1}</h4>
                  <div className="grid grid-cols-2 gap-3 text-sm">
                    <div>
                      <Label className="font-semibold">物質名稱：</Label>
                      <span>{item.name || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">劑型：</Label>
                      <span>{item.form || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">用途：</Label>
                      <span>{item.purpose || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">保存環境：</Label>
                      <span>{item.storage_conditions || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">本物質是否為無菌製備：</Label>
                      <span>{item.is_sterile ? '是' : '否'}</span>
                    </div>
                    {!item.is_sterile && item.non_sterile_justification && (
                      <div className="col-span-2">
                        <Label className="font-semibold">說明：</Label>
                        <p className="mt-1 text-sm whitespace-pre-wrap">{item.non_sterile_justification}</p>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* 對照物質 */}
          {items.control_items && items.control_items.length > 0 && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-3">對照物質</h3>
              {items.control_items.map((item: any, index: number) => (
                <div key={index} className="mb-4 p-4 border rounded bg-slate-50">
                  <h4 className="font-medium mb-2">對照物質 #{index + 1}</h4>
                  <div className="grid grid-cols-2 gap-3 text-sm">
                    <div>
                      <Label className="font-semibold">對照名稱：</Label>
                      <span>{item.name || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">目的：</Label>
                      <span>{item.purpose || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">保存環境：</Label>
                      <span>{item.storage_conditions || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">本物質是否為無菌製備：</Label>
                      <span>{item.is_sterile ? '是' : '否'}</span>
                    </div>
                    {!item.is_sterile && item.non_sterile_justification && (
                      <div className="col-span-2">
                        <Label className="font-semibold">說明：</Label>
                        <p className="mt-1 text-sm whitespace-pre-wrap">{item.non_sterile_justification}</p>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>
      )}

      {/* 4. 研究設計與方法 */}
      {design.procedures && (
        <section className="mb-8 border-t pt-6">
          <h2 className="text-2xl font-bold mb-4 border-b pb-2">4. 研究設計與方法</h2>
          <div className="mb-4">
            <h3 className="text-lg font-semibold mb-2">動物試驗流程描述</h3>
            <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{design.procedures}</p>
          </div>
        </section>
      )}

      {/* Print Styles */}
      <style>{`
        @media print {
          .protocol-pdf-view {
            box-shadow: none;
            padding: 20px;
          }
          .protocol-pdf-view section {
            page-break-inside: avoid;
          }
          .protocol-pdf-view h2 {
            page-break-after: avoid;
          }
        }
      `}</style>
    </div>
  )
}