import { useEffect, useRef } from 'react'
import { useSearchParams } from 'react-router-dom'

import { GOOGLE_SEARCH_PARAMS, parseGoogleRedirect, type GoogleRedirectResult } from '../googleLogin'

/**
 * R121：讀取 Google 登入導回登入頁的結果，交給 `onResult` 處理**一次**，並立即把網址清乾淨：
 * - 2FA temp token 不留在網址列與瀏覽紀錄（先移除 fragment，再以 replace 清 query）；
 * - 重新整理頁面不會重複觸發（例如重複跳錯誤訊息）。
 *
 * `onResult` 透過 ref 取最新版本，呼叫端不必為了 deps 穩定而包 useCallback。
 */
export function useGoogleLoginRedirect(
  onResult: (result: GoogleRedirectResult) => void | Promise<void>,
) {
  const [searchParams, setSearchParams] = useSearchParams()
  const onResultRef = useRef(onResult)
  const handledRef = useRef(false)

  useEffect(() => {
    onResultRef.current = onResult
  }, [onResult])

  useEffect(() => {
    if (handledRef.current) return
    const result = parseGoogleRedirect(searchParams, window.location.hash)
    if (!result) return
    handledRef.current = true

    if (window.location.hash) {
      window.history.replaceState(
        window.history.state,
        '',
        `${window.location.pathname}${window.location.search}`,
      )
    }
    const next = new URLSearchParams(searchParams)
    GOOGLE_SEARCH_PARAMS.forEach((key) => next.delete(key))
    setSearchParams(next, { replace: true })

    void onResultRef.current(result)
  }, [searchParams, setSearchParams])
}
