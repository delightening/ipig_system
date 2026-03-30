/**
 * R20: AI 預審 API
 */
import api from './client'

import type {
    AiReviewResponse,
    RemainingCount,
    ValidationResult,
} from '@/types/aiReview'

export const aiReviewApi = {
    /** R20-2: Level 1 規則驗證 */
    validate: async (protocolId: string): Promise<ValidationResult> => {
        const res = await api.post<ValidationResult>(
            `/protocols/${protocolId}/validate`
        )
        return res.data
    },

    /** R20-6: 客戶端 AI 預審 */
    requestAiReview: async (protocolId: string): Promise<AiReviewResponse> => {
        const res = await api.post<AiReviewResponse>(
            `/protocols/${protocolId}/ai-review`
        )
        return res.data
    },

    /** R20-6: 取得最新 AI 預審結果 */
    getLatestAiReview: async (
        protocolId: string
    ): Promise<AiReviewResponse | null> => {
        const res = await api.get<AiReviewResponse | null>(
            `/protocols/${protocolId}/ai-review/latest`
        )
        return res.data
    },

    /** R20-6: 取得剩餘次數 */
    getRemainingCount: async (): Promise<RemainingCount> => {
        const res = await api.get<RemainingCount>('/ai-review/remaining')
        return res.data
    },

    /** R20-7: 執行秘書 AI 標註 */
    requestStaffReview: async (
        protocolId: string
    ): Promise<AiReviewResponse> => {
        const res = await api.post<AiReviewResponse>(
            `/protocols/${protocolId}/staff-review-assist`
        )
        return res.data
    },

    /** R20-7: 取得最新執行秘書標註 */
    getLatestStaffReview: async (
        protocolId: string
    ): Promise<AiReviewResponse | null> => {
        const res = await api.get<AiReviewResponse | null>(
            `/protocols/${protocolId}/staff-review-assist/latest`
        )
        return res.data
    },
}
