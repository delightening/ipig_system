// 手寫簽名 Canvas 元件
// 封裝 signature_pad 函式庫，支援觸控/滑鼠繪製、清除、復原
// 匯出 SVG + 原始筆跡點 JSON

import { useRef, useEffect, useState, useCallback } from 'react'
import SignaturePad from 'signature_pad'
import { Button } from '@/components/ui/button'
import { Eraser, Undo2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { sanitizeSvg } from '@/lib/sanitize'

// 簽名資料型別
export interface SignatureData {
    svg: string          // SVG 向量簽名
    strokeData: object[] // 原始筆跡點（座標、壓力、時間）
    timestamp: string    // 簽名時間 ISO 8601
}

interface HandwrittenSignaturePadProps {
    /** 簽名變更回呼 */
    onSignatureChange?: (data: SignatureData | null) => void
    /** Canvas 寬度（預設自動填滿） */
    width?: number
    /** Canvas 高度 */
    height?: number
    /** 是否停用 */
    disabled?: boolean
    /** 已有簽名預覽（SVG 字串） */
    previewSvg?: string
    /** 自訂 className */
    className?: string
}

export function HandwrittenSignaturePad({
    onSignatureChange,
    width,
    height = 200,
    disabled = false,
    previewSvg,
    className = '',
}: HandwrittenSignaturePadProps) {
    const { t } = useTranslation()
    const canvasRef = useRef<HTMLCanvasElement>(null)
    const containerRef = useRef<HTMLDivElement>(null)
    const wrapperRef = useRef<HTMLDivElement>(null)
    const padRef = useRef<SignaturePad | null>(null)
    const [isEmpty, setIsEmpty] = useState(true)
    const [showPreview, setShowPreview] = useState(!!previewSvg)

    // 初始化 SignaturePad
    useEffect(() => {
        if (!canvasRef.current || showPreview) return

        const canvas = canvasRef.current
        const pad = new SignaturePad(canvas, {
            backgroundColor: 'rgba(255, 255, 255, 0)',
            penColor: '#1a1a2e',
            minWidth: 1.5,
            maxWidth: 3,
            velocityFilterWeight: 0.7,
        })

        // 監聽繪製事件
        pad.addEventListener('endStroke', () => {
            setIsEmpty(pad.isEmpty())
            emitChange(pad)
        })

        if (disabled) {
            pad.off()
        }

        padRef.current = pad

        return () => {
            pad.off()
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps -- init only, refs/emitChange stable
    }, [disabled, showPreview])

    // 響應式調整 Canvas 大小
    useEffect(() => {
        if (!wrapperRef.current || !canvasRef.current || showPreview) return

        const resizeCanvas = () => {
            const wrapper = wrapperRef.current
            const canvas = canvasRef.current
            if (!wrapper || !canvas) return

            const ratio = Math.max(window.devicePixelRatio || 1, 1)
            const w = width || wrapper.clientWidth
            if (w <= 0) return

            canvas.width = w * ratio
            canvas.height = height * ratio
            canvas.style.width = `${w}px`
            canvas.style.height = `${height}px`

            const ctx = canvas.getContext('2d')
            if (ctx) {
                ctx.scale(ratio, ratio)
            }

            if (padRef.current && !padRef.current.isEmpty()) {
                const data = padRef.current.toData()
                padRef.current.clear()
                padRef.current.fromData(data)
            }
        }

        resizeCanvas()

        const observer = new ResizeObserver(resizeCanvas)
        observer.observe(wrapperRef.current)

        return () => observer.disconnect()
    }, [width, height, showPreview])

    // 匯出簽名資料
    const emitChange = useCallback((pad: SignaturePad) => {
        if (!onSignatureChange) return
        if (pad.isEmpty()) {
            onSignatureChange(null)
            return
        }

        const svgData = pad.toSVG()
        const strokeData = pad.toData()
        onSignatureChange({
            svg: svgData,
            strokeData: strokeData as unknown as object[],
            timestamp: new Date().toISOString(),
        })
    }, [onSignatureChange])

    // 清除簽名
    const handleClear = useCallback(() => {
        if (padRef.current) {
            padRef.current.clear()
            setIsEmpty(true)
            onSignatureChange?.(null)
        }
    }, [onSignatureChange])

    // 復原最後一筆
    const handleUndo = useCallback(() => {
        if (padRef.current) {
            const data = padRef.current.toData()
            if (data.length > 0) {
                data.pop()
                padRef.current.fromData(data)
                setIsEmpty(padRef.current.isEmpty())
                emitChange(padRef.current)
            }
        }
    }, [emitChange])

    // 如果有預覽模式（已簽名），顯示靜態 SVG
    if (showPreview && previewSvg) {
        return (
            <div className={`signature-preview ${className}`}>
                <div
                    className="signature-preview-image"
                    style={{ height: `${height}px` }}
                    dangerouslySetInnerHTML={{ __html: sanitizeSvg(previewSvg) }}
                />
                {!disabled && (
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => setShowPreview(false)}
                        className="mt-2"
                    >
                        {t('signature.resignBtn', '重新簽名')}
                    </Button>
                )}
            </div>
        )
    }

    return (
        <div ref={containerRef} className={`signature-pad-container ${className}`}>
            {/* Canvas 繪圖區 */}
            <div ref={wrapperRef} className="signature-canvas-wrapper" style={{ height: `${height}px` }}>
                <canvas
                    ref={canvasRef}
                    className={`signature-canvas ${disabled ? 'signature-canvas-disabled' : ''}`}
                />
                {/* 簽名線提示 */}
                <div className="signature-line" />
                {/* 空白狀態提示 */}
                {isEmpty && !disabled && (
                    <div className="signature-placeholder">
                        {t('signature.signHere', '請在此處簽名')}
                    </div>
                )}
            </div>

            {/* 控制按鈕 */}
            {!disabled && (
                <div className="signature-controls">
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={handleUndo}
                        disabled={isEmpty}
                    >
                        <Undo2 className="w-4 h-4 mr-1" />
                        {t('signature.undo', '復原')}
                    </Button>
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={handleClear}
                        disabled={isEmpty}
                    >
                        <Eraser className="w-4 h-4 mr-1" />
                        {t('signature.clear', '清除')}
                    </Button>
                </div>
            )}
        </div>
    )
}

export default HandwrittenSignaturePad
