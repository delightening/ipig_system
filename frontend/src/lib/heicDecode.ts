// HEIC / HEIF 前端解碼：iOS「高效率」拍攝且系統未自動轉 JPEG 時，瀏覽器原生
// createImageBitmap 解不開 HEIC。改用 libheif-js（WASM）在前端就地解碼。
//
// 為何用 wasm-bundle 變體：
//   - .wasm 內聯進 .js（self-contained），瀏覽器端不需額外 fetch 二進位，
//     不受 CSP connect-src 限制。
//   - WASM 編譯走 'wasm-unsafe-eval'——現行 prod CSP（nginx security-headers.conf）
//     已放行，無需放寬 CSP，亦不觸及 R58/R31-10 禁 'unsafe-eval' 鐵律。
//   - lazy dynamic import：只有真的遇到 HEIC 才下載 ~1.5MB 解碼器 chunk，
//     一般 JPEG/PNG 上傳零額外成本（Vite 自動 code-split）。

const HEIC_RE = /heic|heif/i

/**
 * 判斷是否為 HEIC / HEIF。部分裝置 / 瀏覽器對 `.heic` 檔回傳的 `file.type` 為空字串，
 * 故同時比對 MIME 與副檔名。
 */
export function isHeic(file: File): boolean {
    return HEIC_RE.test(file.type) || /\.(heic|heif)$/i.test(file.name)
}

/**
 * 將 HEIC / HEIF File 解碼為 HTMLCanvasElement（全解析度）。
 *
 * compressImage 直接以此 Canvas 作為 createImageBitmap 來源，省去「HEIC→JPEG→再解碼」
 * 的雙重編解碼開銷（行動端高解析度照片時對 CPU/記憶體差異顯著）。解碼失敗時 throw。
 */
export async function heicToCanvas(file: File): Promise<HTMLCanvasElement> {
    const { default: libheif } = await import('libheif-js/wasm-bundle')
    const buffer = new Uint8Array(await file.arrayBuffer())

    const decoder = new libheif.HeifDecoder()
    const images = decoder.decode(buffer)
    if (!images || images.length === 0) {
        throw new Error('HEIC 解碼失敗：檔案不含可解析的影像')
    }

    try {
        const image = images[0]
        const width = image.get_width()
        const height = image.get_height()

        const canvas = document.createElement('canvas')
        canvas.width = width
        canvas.height = height
        const ctx = canvas.getContext('2d')
        if (!ctx) throw new Error('Canvas 2D context 不可用')

        const imageData = ctx.createImageData(width, height)
        await new Promise<void>((resolve, reject) => {
            image.display(imageData, displayData => {
                if (!displayData) {
                    reject(new Error('HEIC 解碼失敗：影像處理錯誤'))
                    return
                }
                resolve()
            })
        })
        ctx.putImageData(imageData, 0, 0)
        return canvas
    } finally {
        // 釋放所有解碼影像（主圖 + 可能的縮圖 / 深度圖等輔助影像）的 WASM 記憶體，
        // 避免大圖累積洩漏；該版本未提供 free 時為 no-op。
        for (const img of images) {
            img.free?.()
        }
    }
}

/**
 * 將 HEIC / HEIF File 重新編碼為 JPEG File（全解析度）。供需要 File 物件直接上傳的
 * 路徑使用（如 FileUpload 元件）；compressImage 改走 heicToCanvas 以免多一次編碼。
 */
export async function heicToJpegFile(file: File, quality = 0.92): Promise<File> {
    const canvas = await heicToCanvas(file)
    const blob: Blob = await new Promise((resolve, reject) => {
        canvas.toBlob(
            b => (b ? resolve(b) : reject(new Error('HEIC→JPEG 轉檔失敗'))),
            'image/jpeg',
            quality,
        )
    })

    const newName = `${file.name.replace(/\.[^.]+$/, '')}.jpg`
    return new File([blob], newName, { type: 'image/jpeg', lastModified: file.lastModified })
}
