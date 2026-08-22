import { render, screen, fireEvent } from '@testing-library/react'
import i18n from '@/lib/i18n'
import { PdfExportButtons } from '../PdfExportButtons'

// Force language to zh-TW for deterministic strings (jsdom navigator default is en-US)
beforeAll(async () => {
    await i18n.changeLanguage('zh-TW')
})

describe('PdfExportButtons', () => {
    const baseUrl = '/api/v1/protocols/abc-123'

    let originalLocation: Location
    let assignedHref: string | null

    beforeEach(() => {
        assignedHref = null
        originalLocation = window.location
        Object.defineProperty(window, 'location', {
            configurable: true,
            value: {
                ...originalLocation,
                set href(value: string) {
                    assignedHref = value
                },
                get href() {
                    return assignedHref ?? originalLocation.href
                },
            },
        })
    })

    afterEach(() => {
        Object.defineProperty(window, 'location', {
            configurable: true,
            value: originalLocation,
        })
    })

    it('renders all three buttons by default', () => {
        render(<PdfExportButtons baseUrl={baseUrl} />)
        expect(screen.getByRole('button', { name: /預覽 PDF/ })).toBeInTheDocument()
        expect(screen.getByRole('button', { name: /下載 docx/ })).toBeInTheDocument()
        expect(screen.getByRole('button', { name: /下載 PDF/ })).toBeInTheDocument()
    })

    it('hides preview button when showPreview=false', () => {
        render(<PdfExportButtons baseUrl={baseUrl} showPreview={false} />)
        expect(screen.queryByRole('button', { name: /預覽 PDF/ })).not.toBeInTheDocument()
    })

    it('hides docx button when showDownloadDocx=false', () => {
        render(<PdfExportButtons baseUrl={baseUrl} showDownloadDocx={false} />)
        expect(screen.queryByRole('button', { name: /下載 docx/ })).not.toBeInTheDocument()
    })

    it('hides pdf button when showDownloadPdf=false', () => {
        render(<PdfExportButtons baseUrl={baseUrl} showDownloadPdf={false} />)
        expect(screen.queryByRole('button', { name: /下載 PDF/ })).not.toBeInTheDocument()
    })

    it('navigates to docx export endpoint on docx click', () => {
        render(<PdfExportButtons baseUrl={baseUrl} />)
        fireEvent.click(screen.getByRole('button', { name: /下載 docx/ }))
        expect(assignedHref).toBe(`${baseUrl}/export-docx-v3`)
    })

    it('navigates to pdf export endpoint on pdf click', () => {
        render(<PdfExportButtons baseUrl={baseUrl} />)
        fireEvent.click(screen.getByRole('button', { name: /下載 PDF/ }))
        expect(assignedHref).toBe(`${baseUrl}/export-pdf-v3`)
    })

    it('opens preview dialog with iframe pointing at preview endpoint', () => {
        render(<PdfExportButtons baseUrl={baseUrl} />)
        fireEvent.click(screen.getByRole('button', { name: /預覽 PDF/ }))
        const iframe = document.querySelector('iframe')
        expect(iframe).not.toBeNull()
        expect(iframe?.getAttribute('src')).toBe(`${baseUrl}/preview-pdf-v3`)
    })

    it('uses custom previewTitle when provided', () => {
        const customTitle = '計畫書 PDF 預覽'
        render(<PdfExportButtons baseUrl={baseUrl} previewTitle={customTitle} />)
        fireEvent.click(screen.getByRole('button', { name: /預覽 PDF/ }))
        expect(screen.getByText(customTitle)).toBeInTheDocument()
    })

    it('renders nothing + console.error when baseUrl is empty (safeParse fallback)', () => {
        // 改用 safeParse — 生產環境 invalid props 不該整個元件樹崩潰
        const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
        const { container } = render(<PdfExportButtons baseUrl="" />)
        expect(container.firstChild).toBeNull()
        expect(spy).toHaveBeenCalledWith(
            '[PdfExportButtons] Invalid props, rendering nothing:',
            expect.any(Object),
        )
        spy.mockRestore()
    })

    it('normalizes baseUrl trailing slash (avoids double slash in URL)', () => {
        // CodeRabbit PR #316 review — baseUrl `/api/v1/` 結尾會造成 //export-...
        render(<PdfExportButtons baseUrl="/api/v1/protocols/abc/" showPreview={false} />)
        const docxBtn = screen.getByRole('button', { name: /下載 docx/i })
        // jsdom navigation API 用 spy，只驗 click 不報錯（normalize 後路徑無 //）
        const locationSpy = vi.spyOn(window.location, 'href', 'set')
        fireEvent.click(docxBtn)
        expect(locationSpy).toHaveBeenCalledWith('/api/v1/protocols/abc/export-docx-v3')
        locationSpy.mockRestore()
    })
})
