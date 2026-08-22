/**
 * R20-3: 驗證結果面板
 *
 * 顯示 Level 1 規則引擎的驗證結果：
 * - errors（紅色）：必須修正
 * - warnings（黃色）：建議改善
 * - passed（綠色，可摺疊）：已通過
 */
import { useState } from 'react'

import { AlertCircle, AlertTriangle, CheckCircle2, ChevronDown, ChevronRight } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import type { ValidationResult } from '@/types/aiReview'

interface ValidationPanelProps {
    result: ValidationResult
    onDismiss?: () => void
    onIgnoreAndSubmit?: () => void
    hasErrors: boolean
}

export function ValidationPanel({
    result,
    onDismiss,
    onIgnoreAndSubmit,
    hasErrors,
}: ValidationPanelProps) {
    const [showPassed, setShowPassed] = useState(false)

    return (
        <Card className="border-2 border-border">
            <CardHeader className="pb-3">
                <CardTitle className="text-lg flex items-center gap-2">
                    <AlertCircle className="h-5 w-5" />
                    提交前驗證報告
                </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
                {/* Errors */}
                {result.errors.length > 0 && (
                    <div className="space-y-2">
                        <h4 className="text-sm font-semibold flex items-center gap-1.5 text-status-error-text">
                            <AlertCircle className="h-4 w-4" />
                            必須修正（{result.errors.length} 項）
                        </h4>
                        <ul className="space-y-2 ml-5">
                            {result.errors.map((issue) => (
                                <li key={issue.code} className="text-sm">
                                    <div className="font-medium text-status-error-text">
                                        [{issue.category}] {issue.message}
                                    </div>
                                    <div className="text-muted-foreground mt-0.5">
                                        {issue.suggestion}
                                    </div>
                                </li>
                            ))}
                        </ul>
                    </div>
                )}

                {/* Warnings */}
                {result.warnings.length > 0 && (
                    <div className="space-y-2">
                        <h4 className="text-sm font-semibold flex items-center gap-1.5 text-status-warning-text">
                            <AlertTriangle className="h-4 w-4" />
                            建議改善（{result.warnings.length} 項）
                        </h4>
                        <ul className="space-y-2 ml-5">
                            {result.warnings.map((issue) => (
                                <li key={issue.code} className="text-sm">
                                    <div className="font-medium text-status-warning-text">
                                        [{issue.category}] {issue.message}
                                    </div>
                                    <div className="text-muted-foreground mt-0.5">
                                        {issue.suggestion}
                                    </div>
                                </li>
                            ))}
                        </ul>
                    </div>
                )}

                {/* Passed (collapsible) */}
                {result.passed.length > 0 && (
                    <div className="space-y-1">
                        <button
                            onClick={() => setShowPassed(!showPassed)}
                            className="text-sm font-semibold flex items-center gap-1.5 text-status-success-text hover:underline"
                        >
                            {showPassed ? (
                                <ChevronDown className="h-4 w-4" />
                            ) : (
                                <ChevronRight className="h-4 w-4" />
                            )}
                            <CheckCircle2 className="h-4 w-4" />
                            通過檢查（{result.passed.length} 項）
                        </button>
                        {showPassed && (
                            <div className="ml-5 text-sm text-muted-foreground">
                                {result.passed.join('、')}
                            </div>
                        )}
                    </div>
                )}

                {/* Actions */}
                <div className="flex gap-2 pt-2 border-t">
                    {onDismiss && (
                        <Button variant="outline" size="sm" onClick={onDismiss}>
                            返回修改
                        </Button>
                    )}
                    {!hasErrors && onIgnoreAndSubmit && (
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={onIgnoreAndSubmit}
                        >
                            忽略建議，直接提交
                        </Button>
                    )}
                </div>
            </CardContent>
        </Card>
    )
}
