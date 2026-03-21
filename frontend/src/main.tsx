import React from 'react'
import ReactDOM from 'react-dom/client'
import { focusManager, MutationCache, QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter } from 'react-router-dom'
import { ErrorBoundary } from '@/components/ui/error-boundary'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import App from './App'
import './index.css'
import './lib/i18n' // Initialize i18n
import { reportWebVitals } from './lib/webVitals'

const queryClient = new QueryClient({
  /**
   * MutationCache 全域錯誤處理
   *
   * 無論個別 mutation 是否自行處理 onError，此 callback 都會被觸發。
   * 僅在 mutation 未定義 onError 時顯示 toast，避免重複提示。
   */
  mutationCache: new MutationCache({
    onError: (error, _variables, _context, mutation) => {
      // 如果元件已自行定義 onError，由元件自行處理
      if (mutation.options.onError) return

      toast({
        title: '操作失敗',
        description: getApiErrorMessage(error),
        variant: 'destructive',
      })
    },
  }),
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
      refetchOnMount: true,
      staleTime: 2 * 60 * 1000,
      gcTime: 5 * 60 * 1000,
    },
  },
})

// Throttle focus events: at most one focus refetch every 10 seconds
let lastFocusTime = 0
focusManager.setEventListener((handleFocus) => {
  const onFocus = () => {
    const now = Date.now()
    if (now - lastFocusTime > 10_000) {
      lastFocusTime = now
      handleFocus()
    }
  }
  window.addEventListener('visibilitychange', () => {
    if (!document.hidden) onFocus()
  })
  window.addEventListener('focus', onFocus)
  return () => {
    window.removeEventListener('visibilitychange', onFocus)
    window.removeEventListener('focus', onFocus)
  }
})

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <ErrorBoundary>
          <App />
        </ErrorBoundary>
      </BrowserRouter>
    </QueryClientProvider>
  </React.StrictMode>,
)

reportWebVitals()
