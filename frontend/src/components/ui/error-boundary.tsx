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

interface ErrorBoundaryProps {
    children: ReactNode
    /** 自訂錯誤 UI */
    fallback?: ReactNode
}

interface ErrorBoundaryState {
    hasError: boolean
    error: Error | null
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
    constructor(props: ErrorBoundaryProps) {
        super(props)
        this.state = { hasError: false, error: null }
    }

    static getDerivedStateFromError(error: Error): ErrorBoundaryState {
        return { hasError: true, error }
    }

    componentDidCatch(error: Error, info: React.ErrorInfo) {
        console.error('[ErrorBoundary]', error, info.componentStack)
    }

    handleRetry = () => {
        this.setState({ hasError: false, error: null })
    }

    render() {
        if (this.state.hasError) {
            if (this.props.fallback) {
                return this.props.fallback
            }

            return (
                <div className="flex min-h-[300px] flex-col items-center justify-center gap-4 rounded-lg border bg-background p-8 text-center">
                    <AlertTriangle className="h-12 w-12 text-destructive" />
                    <div className="space-y-1">
                        <h3 className="text-lg font-semibold">發生錯誤</h3>
                        <p className="text-sm text-muted-foreground max-w-md">
                            {this.state.error?.message || '頁面發生未預期的錯誤，請重試或聯繫管理者。'}
                        </p>
                    </div>
                    <button
                        onClick={this.handleRetry}
                        className="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
                    >
                        <RefreshCw className="h-4 w-4" />
                        重試
                    </button>
                </div>
            )
        }

        return this.props.children
    }
}
