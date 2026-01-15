import { Label } from '@/components/ui/label'
import { formatDate } from '@/lib/utils'
import { FileText, Download } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useRef } from 'react'
import jsPDF from 'jspdf'

interface ProtocolContentViewProps {
  workingContent: any
  protocolTitle: string
  startDate?: string
  endDate?: string
  onExportPDF?: () => void
}

export function ProtocolContentView({ workingContent, protocolTitle, startDate, endDate, onExportPDF }: ProtocolContentViewProps) {
  const contentRef = useRef<HTMLDivElement>(null)

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
  const guidelines = workingContent.guidelines || {}
  const surgery = workingContent.surgery || {}
  const animals = workingContent.animals || {}
  const personnel = workingContent.personnel || []
  const attachments = workingContent.attachments || []
  const signature = workingContent.signature || []

  const handleExportPDF = async () => {
    if (!contentRef.current) return

    try {
      // Show loading state
      if (onExportPDF) onExportPDF()

      // Create a clone of the content for PDF generation
      const clone = contentRef.current.cloneNode(true) as HTMLElement
      clone.style.position = 'absolute'
      clone.style.left = '-9999px'
      clone.style.width = '210mm' // A4 width
      document.body.appendChild(clone)

      // Wait for images to load
      await new Promise(resolve => setTimeout(resolve, 500))

      // Generate PDF with table of contents
      const pdf = new jsPDF('p', 'mm', 'a4')
      const pageWidth = pdf.internal.pageSize.getWidth()
      const pageHeight = pdf.internal.pageSize.getHeight()
      const margin = 20
      let yPos = margin
      const lineHeight = 7
      const sectionSpacing = 10

      // Helper function to add a new page if needed
      const checkPageBreak = (requiredHeight: number) => {
        if (yPos + requiredHeight > pageHeight - margin) {
          // Only add new page if we have content
          if (yPos > margin + lineHeight) {
            pdf.addPage()
            yPos = margin
            return true
          }
        }
        return false
      }

      // Helper function to add text with word wrap
      const addText = (text: string, fontSize: number, isBold: boolean = false, x: number = margin) => {
        pdf.setFontSize(fontSize)
        pdf.setFont('helvetica', isBold ? 'bold' : 'normal')
        
        const maxWidth = pageWidth - 2 * margin
        const lines = pdf.splitTextToSize(text, maxWidth)
        
        for (const line of lines) {
          checkPageBreak(lineHeight)
          pdf.text(line, x, yPos)
          yPos += lineHeight
        }
      }

      // Table of Contents
      pdf.setFontSize(18)
      pdf.setFont('helvetica', 'bold')
      pdf.text('目錄', pageWidth / 2, yPos, { align: 'center' })
      yPos += lineHeight * 2

      const tocSections = [
        { title: '1. 研究資料', hasContent: true },
        { title: '2. 研究目的', hasContent: purpose.significance || purpose.replacement || purpose.reduction },
        { title: '3. 試驗物質與對照物質', hasContent: items.use_test_item === true },
        { title: '4. 研究設計與方法', hasContent: design.procedures || design.anesthesia || design.pain },
        { title: '5. 相關規範及參考文獻', hasContent: guidelines.content || (guidelines.references && guidelines.references.length > 0) },
        { title: '6. 手術計畫書', hasContent: surgery.surgery_type || surgery.surgery_description },
        { title: '7. 實驗動物資料', hasContent: animals.animals && animals.animals.length > 0 },
        { title: '8. 試驗人員資料', hasContent: personnel.length > 0 },
        { title: '9. 附件', hasContent: attachments.length > 0 },
        { title: '10. 電子簽名', hasContent: signature.length > 0 },
      ].filter(section => section.hasContent)

      // Store page numbers for TOC - we'll update these as we generate content
      const sectionPageNumbers: { [key: string]: number } = {}
      const tocYPositions: { [key: string]: number } = {}

      // Add TOC entries and store their Y positions
      pdf.setFontSize(12)
      pdf.setFont('helvetica', 'normal')
      for (const section of tocSections) {
        checkPageBreak(lineHeight * 2)
        pdf.text(section.title, margin, yPos)
        tocYPositions[section.title] = yPos
        // Placeholder for page number - will be updated later
        pdf.text('...', pageWidth - margin - 10, yPos, { align: 'right' })
        yPos += lineHeight * 1.5
      }

      // Add page break after TOC
      pdf.addPage()
      yPos = margin

      // Helper to get current page number
      const getCurrentPage = () => pdf.getCurrentPageInfo().pageNumber

      // Section 1: 研究資料
      if (true) {
        sectionPageNumbers['1. 研究資料'] = getCurrentPage()
        checkPageBreak(lineHeight * 3)
        addText('1. 研究資料', 16, true)
        yPos += sectionSpacing

        if (basic.is_glp !== undefined) {
          addText(`GLP 屬性: ${basic.is_glp ? '符合 GLP 規範' : '不符合 GLP 規範'}`, 11)
        }
        if (protocolTitle) {
          addText(`研究名稱: ${protocolTitle}`, 11)
        }
        if (basic.project_type) {
          addText(`計畫類型: ${basic.project_type}`, 11)
        }
        if (basic.project_category) {
          addText(`計畫種類: ${basic.project_category}${basic.project_category_other ? ` (${basic.project_category_other})` : ''}`, 11)
        }
        if ((startDate || basic.start_date) && (endDate || basic.end_date)) {
          addText(`預計試驗時程: ${formatDate(startDate || basic.start_date)} ~ ${formatDate(endDate || basic.end_date)}`, 11)
        }

        if (basic.pi) {
          yPos += lineHeight
          addText('計畫主持人:', 12, true)
          if (basic.pi.name) addText(`  姓名: ${basic.pi.name}`, 11)
          if (basic.pi.phone) addText(`  電話: ${basic.pi.phone}`, 11)
          if (basic.pi.email) addText(`  Email: ${basic.pi.email}`, 11)
          if (basic.pi.address) addText(`  地址: ${basic.pi.address}`, 11)
        }

        if (basic.sponsor) {
          yPos += lineHeight
          addText('委託單位:', 12, true)
          if (basic.sponsor.name) addText(`  單位名稱: ${basic.sponsor.name}`, 11)
          if (basic.sponsor.contact_person) addText(`  聯絡人: ${basic.sponsor.contact_person}`, 11)
          if (basic.sponsor.contact_phone) addText(`  聯絡電話: ${basic.sponsor.contact_phone}`, 11)
          if (basic.sponsor.contact_email) addText(`  聯絡 Email: ${basic.sponsor.contact_email}`, 11)
        }

        if (basic.facility) {
          yPos += lineHeight
          addText('試驗機構與設施:', 12, true)
          if (basic.facility.title) addText(`  機構名稱: ${basic.facility.title}`, 11)
          if (basic.housing_location) addText(`  位置: ${basic.housing_location}`, 11)
        }

        yPos += sectionSpacing
      }

      // Section 2: 研究目的
      if (purpose.significance || purpose.replacement || purpose.reduction) {
        sectionPageNumbers['2. 研究目的'] = getCurrentPage()
        checkPageBreak(lineHeight * 3)
        addText('2. 研究目的', 16, true)
        yPos += sectionSpacing

        if (purpose.significance) {
          addText('2.1 研究之目的及重要性', 14, true)
          addText(purpose.significance, 11)
          yPos += lineHeight
        }

        if (purpose.replacement) {
          addText('2.2 請以動物試驗應用3Rs之替代原則，說明本動物試驗之合理性', 14, true)
          if (purpose.replacement.rationale) {
            addText('2.2.1 請說明活體動物試驗之必要性，以及選擇此動物種別的原因', 12, true)
            addText(purpose.replacement.rationale, 11)
            yPos += lineHeight
          }
          if (purpose.replacement.alt_search) {
            addText('2.2.2 請於下列網站搜尋非動物性替代方案', 12, true)
            if (purpose.replacement.alt_search.platforms && purpose.replacement.alt_search.platforms.length > 0) {
              purpose.replacement.alt_search.platforms.forEach((p: string) => {
                addText(`  • ${p}`, 11)
              })
            }
            if (purpose.replacement.alt_search.keywords) {
              addText(`搜尋關鍵字: ${purpose.replacement.alt_search.keywords}`, 11)
            }
            if (purpose.replacement.alt_search.conclusion) {
              addText('搜尋結果與結論:', 11, true)
              addText(purpose.replacement.alt_search.conclusion, 11)
            }
            yPos += lineHeight
          }
          if (purpose.duplicate) {
            addText(`2.2.3 是否為重複他人試驗: ${purpose.duplicate.experiment ? '是' : '否'}`, 12, true)
            if (purpose.duplicate.experiment && purpose.duplicate.justification) {
              addText('請說明重複進行之科學理由:', 11, true)
              addText(purpose.duplicate.justification, 11)
            }
            yPos += lineHeight
          }
        }

        if (purpose.reduction && purpose.reduction.design) {
          addText('2.3 請以實驗動物應用3Rs之減量原則，說明動物試驗設計', 14, true)
          addText(purpose.reduction.design, 11)
          yPos += lineHeight
        }

        yPos += sectionSpacing
      }

      // Section 3: 試驗物質與對照物質
      if (items.use_test_item === true) {
        sectionPageNumbers['3. 試驗物質與對照物質'] = getCurrentPage()
        checkPageBreak(lineHeight * 3)
        addText('3. 試驗物質與對照物質', 16, true)
        yPos += sectionSpacing

        if (items.test_items && items.test_items.length > 0) {
          addText('試驗物質', 14, true)
          items.test_items.forEach((item: any, index: number) => {
            addText(`試驗物質 #${index + 1}`, 12, true)
            if (item.name) addText(`  物質名稱: ${item.name}`, 11)
            if (item.form) addText(`  劑型: ${item.form}`, 11)
            if (item.purpose) addText(`  用途: ${item.purpose}`, 11)
            if (item.storage_conditions) addText(`  保存環境: ${item.storage_conditions}`, 11)
            addText(`  是否為無菌製備: ${item.is_sterile ? '是' : '否'}`, 11)
            if (!item.is_sterile && item.non_sterile_justification) {
              addText(`  說明: ${item.non_sterile_justification}`, 11)
            }
            yPos += lineHeight
          })
        }

        if (items.control_items && items.control_items.length > 0) {
          addText('對照物質', 14, true)
          items.control_items.forEach((item: any, index: number) => {
            addText(`對照物質 #${index + 1}`, 12, true)
            if (item.name) addText(`  對照名稱: ${item.name}`, 11)
            if (item.purpose) addText(`  目的: ${item.purpose}`, 11)
            if (item.storage_conditions) addText(`  保存環境: ${item.storage_conditions}`, 11)
            addText(`  是否為無菌製備: ${item.is_sterile ? '是' : '否'}`, 11)
            if (!item.is_sterile && item.non_sterile_justification) {
              addText(`  說明: ${item.non_sterile_justification}`, 11)
            }
            yPos += lineHeight
          })
        }

        yPos += sectionSpacing
      }

      // Section 4: 研究設計與方法
      if (design.procedures || design.anesthesia || design.pain) {
        sectionPageNumbers['4. 研究設計與方法'] = getCurrentPage()
        checkPageBreak(lineHeight * 3)
        addText('4. 研究設計與方法', 16, true)
        yPos += sectionSpacing

        if (design.procedures) {
          addText('動物試驗流程描述', 14, true)
          addText(design.procedures, 11)
          yPos += lineHeight
        }

        if (design.anesthesia && design.anesthesia.is_under_anesthesia !== null) {
          addText(`是否於麻醉下進行試驗: ${design.anesthesia.is_under_anesthesia ? '是' : '否'}`, 14, true)
          if (design.anesthesia.anesthesia_type) {
            addText(`麻醉類型: ${design.anesthesia.anesthesia_type}`, 11)
          }
          yPos += lineHeight
        }

        if (design.pain && design.pain.category) {
          addText(`疼痛類別: ${design.pain.category}`, 14, true)
          if (design.pain.management_plan) {
            addText('疼痛管理計畫:', 12, true)
            addText(design.pain.management_plan, 11)
          }
          yPos += lineHeight
        }

        if (design.endpoints) {
          if (design.endpoints.experimental_endpoint) {
            addText('實驗終點:', 14, true)
            addText(design.endpoints.experimental_endpoint, 11)
            yPos += lineHeight
          }
          if (design.endpoints.humane_endpoint) {
            addText('人道終點:', 14, true)
            addText(design.endpoints.humane_endpoint, 11)
            yPos += lineHeight
          }
        }

        yPos += sectionSpacing
      }

      // Section 5: 相關規範及參考文獻
      if (guidelines.content || (guidelines.references && guidelines.references.length > 0)) {
        sectionPageNumbers['5. 相關規範及參考文獻'] = getCurrentPage()
        checkPageBreak(lineHeight * 3)
        addText('5. 相關規範及參考文獻', 16, true)
        yPos += sectionSpacing

        if (guidelines.content) {
          addText('相關規範說明', 14, true)
          addText(guidelines.content, 11)
          yPos += lineHeight
        }

        if (guidelines.references && guidelines.references.length > 0) {
          addText('參考文獻', 14, true)
          guidelines.references.forEach((ref: any, index: number) => {
            if (ref.citation) {
              addText(`${index + 1}. ${ref.citation}`, 11)
            }
            if (ref.url) {
              addText(`   URL: ${ref.url}`, 10)
            }
            yPos += lineHeight
          })
        }

        yPos += sectionSpacing
      }

      // Section 6: 手術計畫書
      if (surgery.surgery_type || surgery.surgery_description) {
        sectionPageNumbers['6. 手術計畫書'] = getCurrentPage()
        checkPageBreak(lineHeight * 3)
        addText('6. 手術計畫書', 16, true)
        yPos += sectionSpacing

        if (surgery.surgery_type) {
          addText(`手術類型: ${surgery.surgery_type}`, 14, true)
        }
        if (surgery.preop_preparation) {
          addText('術前準備:', 14, true)
          addText(surgery.preop_preparation, 11)
          yPos += lineHeight
        }
        if (surgery.surgery_description) {
          addText('手術描述:', 14, true)
          addText(surgery.surgery_description, 11)
          yPos += lineHeight
        }
        if (surgery.monitoring) {
          addText('監控方式:', 14, true)
          addText(surgery.monitoring, 11)
          yPos += lineHeight
        }
        if (surgery.postop_care) {
          addText('術後照護:', 14, true)
          addText(surgery.postop_care, 11)
          yPos += lineHeight
        }

        yPos += sectionSpacing
      }

      // Section 7: 實驗動物資料
      if (animals.animals && animals.animals.length > 0) {
        sectionPageNumbers['7. 實驗動物資料'] = getCurrentPage()
        checkPageBreak(lineHeight * 3)
        addText('7. 實驗動物資料', 16, true)
        yPos += sectionSpacing

        animals.animals.forEach((animal: any, index: number) => {
          addText(`動物群組 #${index + 1}`, 14, true)
          if (animal.species) addText(`  物種: ${animal.species}${animal.species_other ? ` (${animal.species_other})` : ''}`, 11)
          if (animal.strain) addText(`  品系: ${animal.strain}${animal.strain_other ? ` (${animal.strain_other})` : ''}`, 11)
          if (animal.sex) addText(`  性別: ${animal.sex}`, 11)
          if (animal.number) addText(`  數量: ${animal.number}`, 11)
          if (!animal.age_unlimited && (animal.age_min || animal.age_max)) {
            addText(`  月齡範圍: ${animal.age_min || '不限'} ~ ${animal.age_max || '不限'}`, 11)
          } else if (animal.age_unlimited) {
            addText('  月齡範圍: 不限', 11)
          }
          if (!animal.weight_unlimited && (animal.weight_min || animal.weight_max)) {
            addText(`  體重範圍: ${animal.weight_min || '不限'}kg ~ ${animal.weight_max || '不限'}kg`, 11)
          } else if (animal.weight_unlimited) {
            addText('  體重範圍: 不限', 11)
          }
          if (animal.housing_location) addText(`  飼養位置: ${animal.housing_location}`, 11)
          yPos += lineHeight
        })

        if (animals.total_animals) {
          addText(`總動物數: ${animals.total_animals}`, 12, true)
        }

        yPos += sectionSpacing
      }

      // Section 8: 試驗人員資料
      if (personnel.length > 0) {
        sectionPageNumbers['8. 試驗人員資料'] = getCurrentPage()
        checkPageBreak(lineHeight * 3)
        addText('8. 試驗人員資料', 16, true)
        yPos += sectionSpacing

        personnel.forEach((person: any, index: number) => {
          addText(`人員 #${index + 1}`, 14, true)
          if (person.name) addText(`  姓名: ${person.name}`, 11)
          if (person.position) addText(`  職位: ${person.position}`, 11)
          if (person.years_experience) addText(`  參與動物試驗年數: ${person.years_experience} 年`, 11)
          if (person.roles && person.roles.length > 0) {
            addText('  工作內容:', 11, true)
            person.roles.forEach((role: string) => {
              addText(`    • ${role}`, 10)
            })
          }
          if (person.trainings && person.trainings.length > 0) {
            addText('  訓練/資格:', 11, true)
            person.trainings.forEach((training: string) => {
              addText(`    • ${training}`, 10)
            })
          }
          yPos += lineHeight * 2
        })

        yPos += sectionSpacing
      }

      // Section 9: 附件
      if (attachments.length > 0) {
        sectionPageNumbers['9. 附件'] = getCurrentPage()
        checkPageBreak(lineHeight * 3)
        addText('9. 附件', 16, true)
        yPos += sectionSpacing

        attachments.forEach((attachment: any, index: number) => {
          if (attachment.file_name) {
            addText(`${index + 1}. ${attachment.file_name}`, 11)
          }
          yPos += lineHeight
        })

        yPos += sectionSpacing
        currentPage = pdf.getCurrentPageInfo().pageNumber
      }

      // Section 10: 電子簽名
      if (signature.length > 0) {
        sectionPageNumbers['10. 電子簽名'] = getCurrentPage()
        checkPageBreak(lineHeight * 3)
        addText('10. 電子簽名', 16, true)
        yPos += sectionSpacing

        signature.forEach((sig: any, index: number) => {
          if (sig.file_name) {
            addText(`${index + 1}. ${sig.file_name}`, 11)
          }
          yPos += lineHeight
        })
      }

      // Update TOC with page numbers
      pdf.setPage(1)
      pdf.setFontSize(12)
      pdf.setFont('helvetica', 'normal')
      
      for (const section of tocSections) {
        const pageNum = sectionPageNumbers[section.title] || 1
        const tocY = tocYPositions[section.title]
        if (tocY) {
          // Clear the placeholder dots by drawing white rectangle
          pdf.setDrawColor(255, 255, 255)
          pdf.setFillColor(255, 255, 255)
          pdf.rect(pageWidth - margin - 15, tocY - 3, 15, lineHeight, 'F')
          // Add the page number
          pdf.text(`${pageNum}`, pageWidth - margin - 5, tocY, { align: 'right' })
        }
      }

      // Remove clone
      document.body.removeChild(clone)

      // Save PDF
      pdf.save(`${protocolTitle || 'AUP計畫書'}_${new Date().toISOString().split('T')[0]}.pdf`)
    } catch (error) {
      console.error('PDF export error:', error)
      alert('PDF 匯出失敗，請重試')
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-end">
        <Button onClick={handleExportPDF} variant="outline">
          <Download className="mr-2 h-4 w-4" />
          匯出 PDF
        </Button>
      </div>
      
      <div ref={contentRef} className="protocol-pdf-view bg-white p-8 shadow-lg max-w-4xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8 border-b pb-4">
          <h1 className="text-3xl font-bold mb-2">AUP 動物試驗計畫書</h1>
          <p className="text-lg text-muted-foreground">{protocolTitle}</p>
        </div>

        {/* 1. 研究資料 */}
        <section className="mb-8 section-1" data-section="1. 研究資料">
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
        {(purpose.significance || purpose.replacement || purpose.reduction) && (
          <section className="mb-8 border-t pt-6 section-2" data-section="2. 研究目的">
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
        )}

        {/* 3. 試驗物質與對照物質 */}
        {items.use_test_item === true && (
          <section className="mb-8 border-t pt-6 section-3" data-section="3. 試驗物質與對照物質">
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
        {(design.procedures || design.anesthesia || design.pain || design.endpoints) && (
          <section className="mb-8 border-t pt-6 section-4" data-section="4. 研究設計與方法">
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">4. 研究設計與方法</h2>
            
            {design.procedures && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">動物試驗流程描述</h3>
                <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{design.procedures}</p>
              </div>
            )}

            {design.anesthesia && design.anesthesia.is_under_anesthesia !== null && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">是否於麻醉下進行試驗</h3>
                <p className="text-sm">{design.anesthesia.is_under_anesthesia ? '是' : '否'}</p>
                {design.anesthesia.anesthesia_type && (
                  <p className="text-sm mt-2">麻醉類型: {design.anesthesia.anesthesia_type}</p>
                )}
              </div>
            )}

            {design.pain && design.pain.category && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">疼痛類別</h3>
                <p className="text-sm">{design.pain.category}</p>
                {design.pain.management_plan && (
                  <div className="mt-2">
                    <p className="text-sm font-medium mb-1">疼痛管理計畫：</p>
                    <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{design.pain.management_plan}</p>
                  </div>
                )}
              </div>
            )}

            {design.endpoints && (
              <div className="mb-4">
                {design.endpoints.experimental_endpoint && (
                  <div className="mb-3">
                    <h3 className="text-lg font-semibold mb-2">實驗終點</h3>
                    <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{design.endpoints.experimental_endpoint}</p>
                  </div>
                )}
                {design.endpoints.humane_endpoint && (
                  <div>
                    <h3 className="text-lg font-semibold mb-2">人道終點</h3>
                    <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{design.endpoints.humane_endpoint}</p>
                  </div>
                )}
              </div>
            )}
          </section>
        )}

        {/* 5. 相關規範及參考文獻 */}
        {(guidelines.content || (guidelines.references && guidelines.references.length > 0)) && (
          <section className="mb-8 border-t pt-6 section-5" data-section="5. 相關規範及參考文獻">
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">5. 相關規範及參考文獻</h2>
            
            {guidelines.content && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">相關規範說明</h3>
                <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{guidelines.content}</p>
              </div>
            )}

            {guidelines.references && guidelines.references.length > 0 && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-3">參考文獻</h3>
                <ol className="list-decimal list-inside space-y-2">
                  {guidelines.references.map((ref: any, index: number) => (
                    <li key={index} className="text-sm">
                      {ref.citation || '-'}
                      {ref.url && (
                        <span className="text-blue-600 ml-2">
                          <a href={ref.url} target="_blank" rel="noopener noreferrer">{ref.url}</a>
                        </span>
                      )}
                    </li>
                  ))}
                </ol>
              </div>
            )}
          </section>
        )}

        {/* 6. 手術計畫書 */}
        {(surgery.surgery_type || surgery.surgery_description) && (
          <section className="mb-8 border-t pt-6 section-6" data-section="6. 手術計畫書">
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">6. 手術計畫書</h2>
            
            {surgery.surgery_type && (
              <div className="mb-4">
                <Label className="text-sm font-semibold">手術類型</Label>
                <p className="mt-1">{surgery.surgery_type}</p>
              </div>
            )}

            {surgery.preop_preparation && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">術前準備</h3>
                <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.preop_preparation}</p>
              </div>
            )}

            {surgery.surgery_description && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">手術描述</h3>
                <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.surgery_description}</p>
              </div>
            )}

            {surgery.monitoring && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">監控方式</h3>
                <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.monitoring}</p>
              </div>
            )}

            {surgery.postop_care && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">術後照護</h3>
                <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.postop_care}</p>
              </div>
            )}

            {surgery.drugs && surgery.drugs.length > 0 && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-3">用藥計畫</h3>
                <div className="space-y-2">
                  {surgery.drugs.map((drug: any, index: number) => (
                    <div key={index} className="p-3 border rounded bg-slate-50">
                      <p className="text-sm font-medium">{drug.drug_name || '-'}</p>
                      <p className="text-xs text-muted-foreground">
                        劑量: {drug.dose || '-'} | 途徑: {drug.route || '-'} | 頻率: {drug.frequency || '-'} | 目的: {drug.purpose || '-'}
                      </p>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </section>
        )}

        {/* 7. 實驗動物資料 */}
        {animals.animals && animals.animals.length > 0 && (
          <section className="mb-8 border-t pt-6 section-7" data-section="7. 實驗動物資料">
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">7. 實驗動物資料</h2>
            
            <div className="space-y-4">
              {animals.animals.map((animal: any, index: number) => (
                <div key={index} className="p-4 border rounded bg-slate-50">
                  <h3 className="text-lg font-semibold mb-3">動物群組 #{index + 1}</h3>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <Label className="font-semibold">物種</Label>
                      <p className="mt-1">{animal.species || '-'}{animal.species_other ? ` (${animal.species_other})` : ''}</p>
                    </div>
                    {animal.strain && (
                      <div>
                        <Label className="font-semibold">品系</Label>
                        <p className="mt-1">{animal.strain}{animal.strain_other ? ` (${animal.strain_other})` : ''}</p>
                      </div>
                    )}
                    <div>
                      <Label className="font-semibold">性別</Label>
                      <p className="mt-1">{animal.sex || '-'}</p>
                    </div>
                    <div>
                      <Label className="font-semibold">數量</Label>
                      <p className="mt-1">{animal.number || '-'}</p>
                    </div>
                    <div>
                      <Label className="font-semibold">月齡範圍</Label>
                      <p className="mt-1">
                        {animal.age_unlimited ? '不限' : `${animal.age_min || '不限'} ~ ${animal.age_max || '不限'}`}
                      </p>
                    </div>
                    <div>
                      <Label className="font-semibold">體重範圍</Label>
                      <p className="mt-1">
                        {animal.weight_unlimited ? '不限' : `${animal.weight_min || '不限'}kg ~ ${animal.weight_max || '不限'}kg`}
                      </p>
                    </div>
                    {animal.housing_location && (
                      <div className="col-span-2">
                        <Label className="font-semibold">飼養位置</Label>
                        <p className="mt-1">{animal.housing_location}</p>
                      </div>
                    )}
                  </div>
                </div>
              ))}

              {animals.total_animals && (
                <div className="mt-4 p-4 bg-blue-50 rounded">
                  <Label className="text-lg font-semibold">總動物數: {animals.total_animals}</Label>
                </div>
              )}
            </div>
          </section>
        )}

        {/* 8. 試驗人員資料 */}
        {personnel.length > 0 && (
          <section className="mb-8 border-t pt-6 section-8" data-section="8. 試驗人員資料">
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">8. 試驗人員資料</h2>
            
            <div className="space-y-4">
              {personnel.map((person: any, index: number) => (
                <div key={index} className="p-4 border rounded bg-slate-50">
                  <h3 className="text-lg font-semibold mb-3">人員 #{index + 1}</h3>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <Label className="font-semibold">姓名</Label>
                      <p className="mt-1">{person.name || '-'}</p>
                    </div>
                    <div>
                      <Label className="font-semibold">職位</Label>
                      <p className="mt-1">{person.position || '-'}</p>
                    </div>
                    <div>
                      <Label className="font-semibold">參與動物試驗年數</Label>
                      <p className="mt-1">{person.years_experience || '-'} 年</p>
                    </div>
                    {person.roles && person.roles.length > 0 && (
                      <div className="col-span-2">
                        <Label className="font-semibold">工作內容</Label>
                        <div className="mt-1 flex flex-wrap gap-2">
                          {person.roles.map((role: string, roleIndex: number) => (
                            <span key={roleIndex} className="px-2 py-1 bg-blue-100 rounded text-xs">{role}</span>
                          ))}
                        </div>
                      </div>
                    )}
                    {person.trainings && person.trainings.length > 0 && (
                      <div className="col-span-2">
                        <Label className="font-semibold">訓練/資格</Label>
                        <div className="mt-1 flex flex-wrap gap-2">
                          {person.trainings.map((training: string, trainingIndex: number) => (
                            <span key={trainingIndex} className="px-2 py-1 bg-green-100 rounded text-xs">{training}</span>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </section>
        )}

        {/* 9. 附件 */}
        {attachments.length > 0 && (
          <section className="mb-8 border-t pt-6 section-9" data-section="9. 附件">
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">9. 附件</h2>
            
            <div className="space-y-2">
              {attachments.map((attachment: any, index: number) => (
                <div key={index} className="p-3 border rounded">
                  <p className="text-sm">{index + 1}. {attachment.file_name || '-'}</p>
                </div>
              ))}
            </div>
          </section>
        )}

        {/* 10. 電子簽名 */}
        {signature.length > 0 && (
          <section className="mb-8 border-t pt-6 section-10" data-section="10. 電子簽名">
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">10. 電子簽名</h2>
            
            <div className="space-y-4">
              {signature.map((sig: any, index: number) => (
                <div key={index} className="p-4 border rounded">
                  {sig.preview_url ? (
                    <img src={sig.preview_url} alt={sig.file_name || `簽名 ${index + 1}`} className="max-w-xs max-h-32 object-contain" />
                  ) : (
                    <p className="text-sm">{index + 1}. {sig.file_name || '-'}</p>
                  )}
                </div>
              ))}
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
            .protocol-pdf-view section:not(:last-child) {
              page-break-after: auto;
            }
          }
        `}</style>
      </div>
    </div>
  )
}
