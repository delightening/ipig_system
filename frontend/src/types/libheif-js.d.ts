// libheif-js 未隨附 TypeScript 型別。此檔為純 ambient module 宣告（無頂層 import/export），
// 僅描述本專案實際使用的 wasm-bundle 高階解碼 API（見 lib/heicDecode.ts）。
// wasm-bundle 變體把 .wasm 內聯進 .js，瀏覽器端 self-contained。

declare module 'libheif-js/wasm-bundle' {
  interface HeifImage {
    get_width(): number
    get_height(): number
    /** 將解碼後的 RGBA 寫入 imageData，完成時以 displayData 回呼（失敗為 null）。 */
    display(imageData: ImageData, cb: (displayData: ImageData | null) => void): void
    /** 釋放底層 WASM 記憶體（若該版本提供）。 */
    free?(): void
  }
  interface HeifDecoder {
    decode(buffer: Uint8Array): HeifImage[]
  }
  interface LibHeif {
    HeifDecoder: new () => HeifDecoder
  }
  const libheif: LibHeif
  export default libheif
}
