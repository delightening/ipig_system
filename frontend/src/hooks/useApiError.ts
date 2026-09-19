/**
 * 統一 API 錯誤處理 Hook
 *
 * 提供一致的錯誤提示方式，避免各頁面重複撰寫 try-catch + toast 邏輯。
 *
 * 使用方式：
 *   const { handleError, withErrorHandling } = useApiError()
 *   await withErrorHandling(() => deleteResource(`/items/${id}`), '刪除成功')
 */

import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from '@/components/ui/use-toast'
import { logger } from '@/lib/logger'
import { getApiErrorMessage } from '@/lib/apiError'

interface UseApiErrorOptions {
    /** 預設的錯誤標題 */
    defaultTitle?: string
}

export function useApiError(options: UseApiErrorOptions = {}) {
    const { t } = useTranslation()
    const { defaultTitle } = options

    /**
     * 處理 API 錯誤並顯示 toast
     */
    const handleError = useCallback((error: unknown, title?: string) => {
        const message = getApiErrorMessage(error)
        toast({
            title: title || defaultTitle || t('errors.api.operationFailedTitle'),
            description: message,
            variant: 'destructive',
        })
        logger.error('[API Error]', error)
    }, [defaultTitle, t])

    /**
     * 包裝非同步操作，自動處理錯誤
     * @param fn 非同步操作
     * @param successMessage 成功時顯示的訊息（可選）
     * @param errorTitle 錯誤時的標題（可選）
     * @returns 操作結果，失敗時回傳 null
     */
    const withErrorHandling = useCallback(async <T>(
        fn: () => Promise<T>,
        successMessage?: string,
        errorTitle?: string,
    ): Promise<T | null> => {
        try {
            const result = await fn()
            if (successMessage) {
                toast({
                    title: t('common.success'),
                    description: successMessage,
                })
            }
            return result
        } catch (error) {
            handleError(error, errorTitle)
            return null
        }
    }, [handleError, t])

    return { handleError, withErrorHandling }
}
