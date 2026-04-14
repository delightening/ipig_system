/**
 * React Error Boundary 元件
 *
 * 攔截子元件的 JavaScript 錯誤，顯示友善的錯誤畫面。
 *
 * 使用方式：
 *   <ErrorBoundary>
 *     <MyComponent />
 *   </ErrorBoundary>
 *
 *   // 自訂 fallback
 *   <ErrorBoundary fallback={<p>出錯了</p>}>
 *     <MyComponent />
 *   </ErrorBoundary>
 */

import { Component, type ReactNode } from 'react'
import { AlertTriangle, RefreshCw } from 'lucide-react'
import { logger } from '@/lib/logger'

interface ErrorBoundaryProps {
    children: ReactNode
    /** 自訂錯誤 UI */
    fallback?: ReactNode
}

interface ErrorBoundaryState {
    hasError: boolean
    error: Error | null
    /** 自動重新整理倒數秒數，0 表示未啟動或已取消 */
    refreshCountdown: number
    /** 是否為 chunk 載入失敗（部署後 hash 失效） */
    isChunkError: boolean
}

const AUTO_REFRESH_SECONDS = 10
/** sessionStorage key，防止 chunk 錯誤無限重新整理 */
const CHUNK_RELOAD_KEY = 'chunk-error-reload-attempted'

function isChunkLoadError(error: Error): boolean {
    const msg = error?.message ?? ''
    return (
        msg.includes('Failed to fetch dynamically imported module') ||
        msg.includes('Importing a module script failed') ||
        msg.includes('error loading dynamically imported module') ||
        msg.includes('Unable to preload CSS for')
    )
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
    private _countdownInterval: ReturnType<typeof setInterval> | null = null

    constructor(props: ErrorBoundaryProps) {
        super(props)
        this.state = { hasError: false, error: null, refreshCountdown: 0, isChunkError: false }
    }

    static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
        return { hasError: true, error, isChunkError: isChunkLoadError(error) }
    }

    componentDidCatch(error: Error, info: React.ErrorInfo) {
        logger.error('[ErrorBoundary]', error, info.componentStack)

        // 部署後 chunk hash 失效：立即強制重新整理以載入新版本
        // 使用 sessionStorage 防止無限重新整理迴圈
        if (isChunkLoadError(error)) {
            if (!sessionStorage.getItem(CHUNK_RELOAD_KEY)) {
                sessionStorage.setItem(CHUNK_RELOAD_KEY, '1')
                window.location.reload()
            }
            // 已嘗試一次仍失敗 → 顯示錯誤 UI，讓使用者手動處理
        }
    }

    componentDidUpdate(prevProps: ErrorBoundaryProps, prevState: ErrorBoundaryState) {
        // chunk 錯誤由 componentDidCatch 立即處理，不需倒數
        if (this.state.isChunkError) return

        // 剛進入錯誤狀態時，啟動 10 秒倒數並自動重新整理
        if (!prevState.hasError && this.state.hasError) {
            this.setState({ refreshCountdown: AUTO_REFRESH_SECONDS })
            this._countdownInterval = setInterval(() => {
                this.setState((prev) => {
                    const next = prev.refreshCountdown - 1
                    if (next <= 0) {
                        if (this._countdownInterval != null) {
                            clearInterval(this._countdownInterval)
                            this._countdownInterval = null
                        }
                        window.location.reload()
                        return prev
                    }
                    return { ...prev, refreshCountdown: next }
                })
            }, 1000)
        }
    }

    componentWillUnmount() {
        if (this._countdownInterval != null) {
            clearInterval(this._countdownInterval)
            this._countdownInterval = null
        }
    }

    handleRetry = () => {
        if (this._countdownInterval != null) {
            clearInterval(this._countdownInterval)
            this._countdownInterval = null
        }
        // chunk 錯誤重試時清除標記，允許再次嘗試強制重新整理
        if (this.state.isChunkError) {
            sessionStorage.removeItem(CHUNK_RELOAD_KEY)
            window.location.reload()
            return
        }
        this.setState({ hasError: false, error: null, refreshCountdown: 0, isChunkError: false })
    }

    render() {
        if (this.state.hasError) {
            if (this.props.fallback) {
                return this.props.fallback
            }

            const { refreshCountdown, isChunkError } = this.state

            return (
                <div className="flex min-h-[300px] flex-col items-center justify-center gap-4 rounded-lg border bg-background p-8 text-center">
                    <AlertTriangle className="h-12 w-12 text-destructive" />
                    <div className="space-y-1">
                        <h3 className="text-lg font-semibold">
                            {isChunkError ? '版本更新' : '發生錯誤'}
                        </h3>
                        <p className="text-sm text-muted-foreground max-w-md">
                            {isChunkError
                                ? '系統已更新新版本，正在重新整理頁面…'
                                : '頁面發生未預期的錯誤，請重試或聯繫管理者。'}
                        </p>
                        {refreshCountdown > 0 && (
                            <p className="text-sm text-muted-foreground">
                                {refreshCountdown} 秒後自動重新整理…
                            </p>
                        )}
                    </div>
                    <button
                        onClick={this.handleRetry}
                        className="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
                    >
                        <RefreshCw className="h-4 w-4" />
                        {isChunkError ? '立即重新整理' : '立即重試'}
                    </button>
                </div>
            )
        }

        return this.props.children
    }
}
