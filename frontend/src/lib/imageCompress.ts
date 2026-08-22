// R39: 行動端拍照後的瀏覽器端壓縮 + EXIF 旋轉
//
// 主流程不引第三方庫 — 用 Canvas + createImageBitmap 完成：
//   1. createImageBitmap({ imageOrientation: 'from-image' }) 自動套用 EXIF orientation
//   2. 等比縮放長邊到 maxLongEdge
//   3. canvas.toBlob('image/jpeg', quality) 輸出（同時 strip EXIF — Canvas re-encode 不帶 EXIF）
//
// 預設：長邊 2000px、JPEG quality 0.85。手機原圖 4-12MB → 約 1MB。
//
// HEIC / HEIF 例外：瀏覽器原生 createImageBitmap 解不開，先以 libheif-js（WASM，lazy
// import，見 heicDecode.ts）解碼為 Canvas，直接作為 createImageBitmap 來源走下方同一條
// 壓縮流程，避免「HEIC→JPEG→再解碼」的雙重編解碼。

import { heicToCanvas, isHeic } from './heicDecode'

export interface CompressOptions {
    maxLongEdge?: number
    quality?: number
    /** 強制輸出格式；預設保留 JPEG / PNG，其他轉 JPEG */
    mimeType?: 'image/jpeg' | 'image/png'
}

const DEFAULT_MAX_LONG_EDGE = 2000
const DEFAULT_QUALITY = 0.85

export async function compressImage(
    file: File,
    opts: CompressOptions = {},
): Promise<File> {
    const maxLongEdge = opts.maxLongEdge ?? DEFAULT_MAX_LONG_EDGE
    const quality = opts.quality ?? DEFAULT_QUALITY

    // HEIC / HEIF：先在前端解碼為 Canvas（libheif-js WASM），直接作為 createImageBitmap
    // 來源，避免雙重編解碼。失敗（極舊瀏覽器無 WASM / 損壞檔）時 throw，由上層 toast 提示。
    // 非 HEIC 維持原樣傳 File。
    const source: HTMLCanvasElement | File = isHeic(file) ? await heicToCanvas(file) : file

    // createImageBitmap 套 EXIF orientation；fallback 給不支援的舊瀏覽器。
    // （Canvas 來源無 EXIF，方向已由 libheif 解碼時套用。）
    let bitmap: ImageBitmap
    try {
        bitmap = await createImageBitmap(source, { imageOrientation: 'from-image' })
    } catch {
        // 退回不套 EXIF（少見；現代手機都支援）
        bitmap = await createImageBitmap(source)
    }

    const { width: w0, height: h0 } = bitmap
    const longEdge = Math.max(w0, h0)
    const scale = longEdge > maxLongEdge ? maxLongEdge / longEdge : 1
    const w = Math.round(w0 * scale)
    const h = Math.round(h0 * scale)

    const canvas = document.createElement('canvas')
    canvas.width = w
    canvas.height = h
    const ctx = canvas.getContext('2d')
    if (!ctx) throw new Error('Canvas 2D context 不可用')
    ctx.drawImage(bitmap, 0, 0, w, h)
    bitmap.close()

    // 輸出格式：原為 PNG 保 PNG（透明度），其他一律 JPEG（壓縮率高）。
    // HEIC 走 Canvas 來源，依原始 file 判斷（非 PNG）→ JPEG。
    const outputType: 'image/jpeg' | 'image/png' =
        opts.mimeType ?? (file.type === 'image/png' ? 'image/png' : 'image/jpeg')
    const outputQuality = outputType === 'image/jpeg' ? quality : undefined

    const blob: Blob = await new Promise((resolve, reject) => {
        canvas.toBlob(
            b => (b ? resolve(b) : reject(new Error('Canvas toBlob 失敗'))),
            outputType,
            outputQuality,
        )
    })

    // 換副檔名（如 photo.heic → photo.jpg；保 PNG 副檔名）
    const newExt = outputType === 'image/png' ? 'png' : 'jpg'
    const baseName = file.name.replace(/\.[^.]+$/, '')
    const newName = `${baseName}.${newExt}`

    return new File([blob], newName, { type: outputType, lastModified: Date.now() })
}
