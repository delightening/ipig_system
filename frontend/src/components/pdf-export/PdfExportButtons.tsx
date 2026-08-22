/**
 * PdfExportButtons — 通用 PDF 匯出按鈕組
 *
 * 提供「預覽 PDF / 下載 docx / 下載 PDF」三顆按鈕，呼叫對應 backend endpoint：
 *   - preview: GET {baseUrl}/preview-pdf-v3 （iframe 顯示於 Dialog）
 *   - docx:    GET {baseUrl}/export-docx-v3 （瀏覽器下載）
 *   - pdf:     GET {baseUrl}/export-pdf-v3  （瀏覽器下載）
 *
 * 對應 R32-A5 任務：純展示元件，不含業務邏輯。實際 wire 到 detail page
 * 由後續 PR (R32-A5b) 負責，待 backend handler 確認 endpoint 結構後進行。
 */

import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Download, FileText, Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'

export interface PdfExportButtonsProps {
    baseUrl: string
    showPreview?: boolean
    showDownloadDocx?: boolean
    showDownloadPdf?: boolean
    previewTitle?: string
}

/** R58: 改原生 runtime guard 取代 Zod safeParse（避開 Function 探測 CSP 問題）。 */
function validatePdfExportButtonsProps(props: unknown): props is PdfExportButtonsProps {
    if (!props || typeof props !== 'object') return false
    const p = props as Record<string, unknown>
    if (typeof p.baseUrl !== 'string' || p.baseUrl.length < 1) return false
    if (p.showPreview !== undefined && typeof p.showPreview !== 'boolean') return false
    if (p.showDownloadDocx !== undefined && typeof p.showDownloadDocx !== 'boolean') return false
    if (p.showDownloadPdf !== undefined && typeof p.showDownloadPdf !== 'boolean') return false
    if (p.previewTitle !== undefined && typeof p.previewTitle !== 'string') return false
    return true
}

const PREVIEW_PATH = '/preview-pdf-v3'
const DOCX_PATH = '/export-docx-v3'
const PDF_PATH = '/export-pdf-v3'

/** 正規化 baseUrl：去尾部斜線避免拼接出 `//` 雙斜線（CodeRabbit PR #316 review）。*/
function normalizeBaseUrl(url: string): string {
    return url.replace(/\/+$/, '')
}

function triggerDownload(url: string): void {
    // HttpOnly cookie 由 axios `withCredentials: true` 設定後同源 navigation
    // 自動帶入；此 API 不需 Authorization header（auth 走 cookie，非 Bearer token）
    window.location.href = url
}

interface ButtonRowProps {
    baseUrl: string
    showPreview: boolean
    showDownloadDocx: boolean
    showDownloadPdf: boolean
    onPreview: () => void
}

function ButtonRow({
    baseUrl,
    showPreview,
    showDownloadDocx,
    showDownloadPdf,
    onPreview,
}: ButtonRowProps) {
    const { t } = useTranslation()
    return (
        <div className="flex flex-wrap items-center gap-2">
            {showPreview && (
                <Button type="button" variant="outline" onClick={onPreview}>
                    <FileText className="mr-2 h-4 w-4" />
                    {t('common.pdfExport.previewPdf')}
                </Button>
            )}
            {showDownloadDocx && (
                <Button
                    type="button"
                    variant="outline"
                    onClick={() => triggerDownload(baseUrl + DOCX_PATH)}
                >
                    <Download className="mr-2 h-4 w-4" />
                    {t('common.pdfExport.downloadDocx')}
                </Button>
            )}
            {showDownloadPdf && (
                <Button
                    type="button"
                    onClick={() => triggerDownload(baseUrl + PDF_PATH)}
                >
                    <Download className="mr-2 h-4 w-4" />
                    {t('common.pdfExport.downloadPdf')}
                </Button>
            )}
        </div>
    )
}

interface PreviewDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    src: string
    title: string
    loaded: boolean
    onLoaded: () => void
}

function PreviewDialog({
    open,
    onOpenChange,
    src,
    title,
    loaded,
    onLoaded,
}: PreviewDialogProps) {
    const { t } = useTranslation()
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent size="xl" className="sm:h-[85vh] flex flex-col">
                <DialogHeader>
                    <DialogTitle>{title}</DialogTitle>
                </DialogHeader>
                <div className="relative flex-1 min-h-[60vh] border rounded-md overflow-hidden bg-muted">
                    {!loaded && (
                        <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-background/80">
                            <Loader2 className="h-8 w-8 animate-spin text-primary" />
                            <p className="text-sm text-muted-foreground animate-pulse">
                                {t('common.pdfExport.generating')}
                            </p>
                        </div>
                    )}
                    {open && (
                        <iframe
                            title={title}
                            src={src}
                            className="w-full h-full"
                            onLoad={onLoaded}
                        />
                    )}
                </div>
            </DialogContent>
        </Dialog>
    )
}

export function PdfExportButtons(props: PdfExportButtonsProps) {
    // Hooks 必須在 early return 之前 — React rules-of-hooks 要求每次 render
    // 都呼叫相同數量的 hooks（CI lint react-hooks/rules-of-hooks 抓到）。
    const { t } = useTranslation()
    const [previewOpen, setPreviewOpen] = useState(false)
    const [iframeLoaded, setIframeLoaded] = useState(false)

    // 原生 runtime guard — 生產環境 invalid props 不該整個元件樹崩潰
    // （CodeRabbit PR #316 review）。dev / test 仍透過 console.error 主動發現問題。
    if (!validatePdfExportButtonsProps(props)) {
        console.error('[PdfExportButtons] Invalid props, rendering nothing:', props)
        return null
    }
    const {
        baseUrl: rawBaseUrl,
        showPreview = true,
        showDownloadDocx = true,
        showDownloadPdf = true,
        previewTitle,
    } = props
    const baseUrl = normalizeBaseUrl(rawBaseUrl)

    const title = previewTitle ?? t('common.pdfExport.previewTitle')

    const handleOpenPreview = () => {
        setIframeLoaded(false)
        setPreviewOpen(true)
    }

    return (
        <>
            <ButtonRow
                baseUrl={baseUrl}
                showPreview={showPreview}
                showDownloadDocx={showDownloadDocx}
                showDownloadPdf={showDownloadPdf}
                onPreview={handleOpenPreview}
            />
            <PreviewDialog
                open={previewOpen}
                onOpenChange={setPreviewOpen}
                src={baseUrl + PREVIEW_PATH}
                title={title}
                loaded={iframeLoaded}
                onLoaded={() => setIframeLoaded(true)}
            />
        </>
    )
}
