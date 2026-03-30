/**
 * R20: AI 預審與驗證相關型別
 */

// Level 1 驗證結果
export interface ValidationIssue {
    code: string
    category: string
    section: string
    message: string
    suggestion: string
}

export interface ValidationResult {
    passed: string[]
    errors: ValidationIssue[]
    warnings: ValidationIssue[]
}

// AI 預審結果
export interface AiReviewIssue {
    severity: 'error' | 'warning'
    category: string
    section: string
    message: string
    suggestion: string
}

export interface AiReviewAiResult {
    summary: string
    score?: number
    issues: AiReviewIssue[]
    passed: string[]
}

// 執行秘書標註
export interface StaffReviewFlag {
    flag_type: 'needs_attention' | 'concern' | 'suggestion'
    section: string
    message: string
    suggestion: string
}

export interface StaffAiResult {
    summary: string
    flags: StaffReviewFlag[]
}

// API Response
export interface AiReviewResponse {
    id: string
    protocol_id: string
    review_type: 'client_pre_submit' | 'staff_pre_review'
    rule_result?: ValidationResult
    ai_result?: AiReviewAiResult | StaffAiResult
    ai_model?: string
    total_errors: number
    total_warnings: number
    score?: number
    duration_ms?: number
    created_at: string
}

export interface RemainingCount {
    remaining: number
}
