import React from 'react'
import ReactDOM from 'react-dom/client'
import { MutationCache, QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter } from 'react-router-dom'
import { ErrorBoundary } from '@/components/ui/error-boundary'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import App from './App'
import './index.css'
import './lib/i18n' // Initialize i18n

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
      refetchOnWindowFocus: true, // Refetch when window regains focus to get latest data
      refetchOnMount: true, // Always refetch on mount to ensure fresh data
      staleTime: 30 * 1000, // 30 seconds - data is considered stale quickly for animal data
      gcTime: 5 * 60 * 1000, // Keep unused data in cache for 5 minutes
    },
  },
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
