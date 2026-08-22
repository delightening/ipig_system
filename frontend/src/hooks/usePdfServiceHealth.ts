import { useQuery } from '@tanstack/react-query'

import api from '@/lib/api/client'

/**
 * GLP 匯出前置 health check：探測 print-pdf 服務是否上線。
 *
 * 背景：GLP 文件（AUP 計畫書 / 審查回覆 / 審核結果 / 巡場報告 / 欄位狀態表）的 PDF
 * 由 print-pdf（WeasyPrint）渲染。服務未上線時 backend 回 503 / glp_ready:false。
 * 為避免使用者按下匯出才看到錯誤，本 hook 在進入匯出頁面 / hover 匯出按鈕時先 ping
 * `/api/pdf-service-health`，未就緒則 disable 按鈕 + tooltip 提示。
 *
 * Backend 偵測到服務未上線會 rate-limited 觸發 email 通知 admin（30 分鐘 1 封）。
 *
 * 結果 cache 30s — 避免每次 hover / 重渲染都打 API。
 *
 * 用法：
 *   const { glpReady, isLoading } = usePdfServiceHealth()
 *   <Button disabled={!glpReady || isLoading}>匯出 PDF</Button>
 *   {!glpReady && <Tooltip>PDF 服務未上線，已通知管理員</Tooltip>}
 */
type PdfServiceHealthResp = {
  glp_ready?: boolean
  service?: string
  engine?: string
  error?: string
}

export function usePdfServiceHealth(options: { enabled?: boolean } = {}) {
  const { enabled = true } = options
  const query = useQuery<PdfServiceHealthResp>({
    queryKey: ['pdf-service-health'],
    queryFn: async () => {
      try {
        // backend route 是 /api/pdf-service-health（與 /metrics 同階，不歸 /v1 versioning），
        // 但 api instance 預設 baseURL=/api/v1，因此用 per-request baseURL='/api' 覆寫。
        const res = await api.get<PdfServiceHealthResp>('/pdf-service-health', {
          baseURL: '/api',
          // 不要把 503 當成「全域錯誤」處理 — 服務 down 是預期可能情況
          _silentError: true,
        } as Parameters<typeof api.get>[1])
        return res.data
      } catch (err: unknown) {
        // axios 把 503 也丟 reject。reuse response body
        if (err && typeof err === 'object' && 'response' in err) {
          const resp = (err as { response?: { data?: unknown } }).response
          // Gemini #531：僅接受物件型 JSON body；HTML 錯誤頁（502/nginx 等非物件）視為 unreachable
          if (resp?.data && typeof resp.data === 'object') {
            return resp.data as PdfServiceHealthResp
          }
        }
        return { glp_ready: false, error: 'unreachable' }
      }
    },
    staleTime: 30_000,
    refetchOnWindowFocus: false,
    refetchOnMount: false,
    retry: 1,
    enabled,
  })

  return {
    glpReady: query.data?.glp_ready === true,
    error: query.data?.error,
    isLoading: query.isLoading,
    refetch: query.refetch,
  }
}
