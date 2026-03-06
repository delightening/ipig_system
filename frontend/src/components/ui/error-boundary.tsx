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
}

const AUTO_REFRESH_SECONDS = 10

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
    private _countdownInterval: ReturnType<typeof setInterval> | null = null

    constructor(props: ErrorBoundaryProps) {
        super(props)
        this.state = { hasError: false, error: null, refreshCountdown: 0 }
    }

    static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
        return { hasError: true, error }
    }

    componentDidCatch(error: Error, info: React.ErrorInfo) {
        logger.error('[ErrorBoundary]', error, info.componentStack)
    }

    componentDidUpdate(prevProps: ErrorBoundaryProps, prevState: ErrorBoundaryState) {
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
        this.setState({ hasError: false, error: null, refreshCountdown: 0 })
    }

    render() {
        if (this.state.hasError) {
            if (this.props.fallback) {
                return this.props.fallback
            }

            const { refreshCountdown } = this.state

            return (
                <div className="flex min-h-[300px] flex-col items-center justify-center gap-4 rounded-lg border bg-background p-8 text-center">
                    <AlertTriangle className="h-12 w-12 text-destructive" />
                    <div className="space-y-1">
                        <h3 className="text-lg font-semibold">發生錯誤</h3>
                        <p className="text-sm text-muted-foreground max-w-md">
                            {this.state.error?.message || '頁面發生未預期的錯誤，請重試或聯繫管理者。'}
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
                        立即重試
                    </button>
                </div>
            )
        }

        return this.props.children
    }
}
