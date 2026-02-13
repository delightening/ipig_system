/**
 * 全頁 Loading Overlay 元件
 *
 * 呈現全畫面或容器內的載入中遮罩，搭配旋轉圖示與可選文字。
 *
 * 使用方式：
 *   <LoadingOverlay />
 *   <LoadingOverlay message="資料載入中..." />
 *   <LoadingOverlay fullScreen={false} />
 */

import { Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'

interface LoadingOverlayProps {
    /** 顯示訊息 */
    message?: string
    /** 是否全螢幕（預設 false，佔滿父容器） */
    fullScreen?: boolean
    /** 額外 CSS class */
    className?: string
}

export function LoadingOverlay({
    message = '載入中...',
    fullScreen = false,
    className,
}: LoadingOverlayProps) {
    return (
        <div
            className={cn(
                'flex flex-col items-center justify-center gap-3',
                fullScreen
                    ? 'fixed inset-0 z-50 bg-background/80 backdrop-blur-sm'
                    : 'min-h-[200px] w-full',
                className,
            )}
        >
            <Loader2 className="h-8 w-8 animate-spin text-primary" />
            {message && (
                <p className="text-sm text-muted-foreground animate-pulse">{message}</p>
            )}
        </div>
    )
}
